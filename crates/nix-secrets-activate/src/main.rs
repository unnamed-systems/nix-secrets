#[macro_use]
extern crate tracing;

pub mod utils;

#[cfg(target_os = "linux")]
use std::io::{BufRead as _, BufReader};
use std::{
    collections::HashMap,
    fs::{self, File, OpenOptions},
    io::{ErrorKind, Write as _},
    os::{
        fd::AsFd,
        unix::fs::{self as unix_fs, OpenOptionsExt},
    },
    path::{Path, PathBuf},
    process::{self},
    result, time,
};

use age::{Identity, cli_common::file_io::InputReader, secrecy::zeroize::Zeroizing};
use aho_corasick::{AhoCorasick, MatchKind};
use clap::{Parser, ValueHint};
use eyre::{Context as _, ContextCompat as _, OptionExt as _, eyre};
use nix::fcntl::AT_FDCWD;
use nix_secrets_core::{
    SECRETS_EXTENSION, age::get_identities_paths, file::PathBufExt as _, log::init_logger,
    model::Manifest,
};

#[cfg(target_os = "linux")]
use nix::sys::statfs::FsType;
use nix_secrets_core::{
    Result,
    age::{decrypt_stream, get_identities},
    model::{ModuleSystem, OwnerOrGroup, Secret, parse_manifest},
};

use crate::utils::set_owner_and_group;

#[cfg(target_os = "linux")]
const RAMFS_MAGIC: FsType = FsType(0x8584_58f6);

#[cfg(target_os = "macos")]
const DARWIN_ANCHOR_NAME: &str = "nix-secrets-anchor";

#[derive(Parser, PartialEq, Eq, Debug)]
/// Activate nix-secrets configuration for the host
pub struct Args {
    /// Perform a dry activation without deploying secrets and templates
    #[arg(long, action = clap::ArgAction::SetTrue, value_parser = clap::value_parser!(bool))]
    pub dry: bool,

    /// Path to the manifest file
    #[arg(value_hint = ValueHint::FilePath)]
    pub manifest: PathBuf,

    /// Only activate secrets and templates with `neededForUsers` matching this value
    #[arg(long, action = clap::ArgAction::Set, value_parser = clap::value_parser!(bool))]
    pub needed_for_users: bool,
}

enum ActivationStage<'a> {
    Initialize,
    ParseIdentitySecrets {
        previous: usize,
        remaining: Vec<&'a Secret>,
    },
    InitializeGenerationDir,
    DecryptSecrets {
        generation_dir: PathBuf,
    },
    BuildTemplates {
        generation_dir: PathBuf,
        hashes: HashMap<&'a str, Zeroizing<String>>,
    },
    LinkMainDirectory,
    LinkIndividualLinkables,
    DeleteOldGenerations,
    Exit,
}

#[allow(clippy::too_many_lines)]
fn main() -> Result<()> {
    init_logger();

    trace!("Initiating activation");
    let args = Args::parse();

    trace!("Manifest argument: {}", args.manifest.display());
    let contents =
        fs::read_to_string(&args.manifest).wrap_err("Failed to read manifest content")?;
    let mut manifest: Manifest = parse_manifest(&contents).wrap_err("Failed to parse manifest")?;
    let mut identities = Vec::new();
    let mut links: HashMap<PathBuf, (PathBuf, bool)> = HashMap::new();
    let mut cleanup: Vec<PathBuf> = Vec::new();

    let mut stage = ActivationStage::Initialize;

    while !matches!(stage, ActivationStage::Exit) {
        match stage {
            ActivationStage::Initialize => {
                let identity_paths: Vec<PathBuf> =
                    manifest.identity_paths.iter().map(PathBuf::from).collect();

                identities = get_identities_paths(&identity_paths)
                    .wrap_err("Failed to parse initial identities")?;

                let remaining_identities: Vec<&Secret> =
                    manifest.secrets.iter().filter(|s| s.is_identity).collect();

                stage = ActivationStage::ParseIdentitySecrets {
                    previous: identities.len(),
                    remaining: remaining_identities,
                }
            }
            ActivationStage::ParseIdentitySecrets {
                previous,
                mut remaining,
            } => {
                let mut res: Result<()> = Ok(());
                remaining.retain_mut(|s| {
                    let mut step = || -> Result<bool> {
                        let decrypted = decrypt_secret(s, &manifest.storage, &identities)?;
                        let new_identities = get_identities(decrypted.as_slice())
                            .wrap_err("Failed to parse extra identities")?;

                        drop(decrypted);

                        identities.extend(new_identities);
                        Ok(false)
                    };

                    match step() {
                        Ok(keep) => keep,
                        Err(err) => {
                            res = Err(err);
                            true
                        }
                    }
                });

                if identities.len() == previous {
                    res.wrap_err("Failed to parse identity secrets")?;

                    manifest
                        .secrets
                        .retain(|s| s.needed_for_users == args.needed_for_users);
                    manifest
                        .templates
                        .retain(|t| t.needed_for_users == args.needed_for_users);

                    if manifest.secrets.len() + manifest.templates.len() == 0 {
                        info!("Nothing to deploy");
                        stage = ActivationStage::Exit;
                    } else {
                        stage = ActivationStage::InitializeGenerationDir;
                    }
                } else {
                    stage = ActivationStage::ParseIdentitySecrets {
                        previous: identities.len(),
                        remaining,
                    };
                }
            }
            ActivationStage::InitializeGenerationDir => {
                let gen_dir = get_generation_directory(
                    &manifest.module_system,
                    if args.needed_for_users {
                        &manifest.generations_for_users_dir
                    } else {
                        &manifest.generations_dir
                    },
                )
                .wrap_err("Failed to get generation directory")?;

                trace!("Base generation directory: `{}`", gen_dir.display());

                let secret_dir_exists = check_generation_dir_existence(&gen_dir)
                    .wrap_err("Failed to check for secret generation existence")?;

                if !secret_dir_exists {
                    mount_secret_fs(&gen_dir, &manifest.module_system)
                        .wrap_err("Failed to create generation directory")?;
                }

                trace!("Base generation directory exists, creating new generation");

                let generation_dir = fs::read_dir(&gen_dir)
                    .wrap_err_with(|| {
                        format!(
                            "Failed to read base generation directory: {}",
                            gen_dir.display()
                        )
                    })?
                    .map(|entry| {
                        let entry =
                            entry.wrap_err("Failed to read a generation directory entry")?;
                        let file_name = entry.file_name();
                        let name = file_name.to_str().ok_or_else(|| {
                            eyre!("Invalid UTF-8 in directory entry: {file_name:?}")
                        })?;

                        let generation: usize = name.parse().wrap_err_with(|| {
                            format!("Failed to parse generation from '{name}'")
                        })?;

                        trace!(
                            "Found existing generation {generation} in {}",
                            gen_dir.display()
                        );
                        Ok((generation, entry.path()))
                    })
                    .try_fold(
                        (0, Vec::new()),
                        |(next, mut entries), res: Result<(usize, PathBuf)>| {
                            let (generation, path) = res?;
                            entries.push((generation, path));
                            Ok::<_, eyre::Report>((next.max(generation.saturating_add(1)), entries))
                        },
                    )
                    .map(|(num, entries)| {
                        cleanup.extend(
                            entries
                                .into_iter()
                                .filter(|(generation, _)| num.saturating_sub(*generation) > 1)
                                .map(|(_, path)| path),
                        );
                        gen_dir.join(num.to_string())
                    })
                    .wrap_err("Failed to read the generation directory")?;

                stage = ActivationStage::DecryptSecrets { generation_dir }
            }
            ActivationStage::DecryptSecrets { generation_dir } => {
                let secret_generation_dir = generation_dir.join("secrets");
                let mut hashes = HashMap::new();

                for secret in &manifest.secrets {
                    let decrypted = decrypt_secret(secret, &manifest.storage, &identities)?;

                    if !secret.template_key.is_empty() {
                        let as_string = Zeroizing::new(String::from_utf8(decrypted.to_vec())?);
                        hashes.insert(secret.template_key.as_str(), as_string);
                    }

                    let generation_dst_location = deploy_linkable_ramfs(
                        &secret_generation_dir,
                        decrypted,
                        &secret.name,
                        &secret.mode,
                        &secret.owner,
                        &secret.group,
                        secret.needed_for_users,
                    )?;

                    links.insert(
                        generation_dst_location,
                        (secret.path.clone(), secret.is_symlink),
                    );
                }

                stage = ActivationStage::BuildTemplates {
                    generation_dir,
                    hashes,
                }
            }
            ActivationStage::BuildTemplates {
                generation_dir,
                hashes,
            } => {
                let templates_generation_dir = generation_dir.join("templates");

                for template in &manifest.templates {
                    trace!("Reading template content from {:?}", template.content);
                    let content = fs::read_to_string(&template.content).wrap_err_with(|| {
                        format!(
                            "Failed to read template content: `{}`",
                            template.content.display()
                        )
                    })?;
                    trace!("Replacing secret hashes");
                    let raw_content = replace_template_keys(&hashes, &content)?;

                    let generation_dst_location = deploy_linkable_ramfs(
                        &templates_generation_dir,
                        raw_content,
                        &template.name,
                        &template.mode,
                        &template.owner,
                        &template.group,
                        template.needed_for_users,
                    )?;

                    links.insert(
                        generation_dst_location,
                        (template.path.clone(), template.is_symlink),
                    );
                }

                if args.dry {
                    stage = ActivationStage::Exit;
                } else {
                    stage = ActivationStage::LinkMainDirectory;
                }
            }
            ActivationStage::LinkMainDirectory => {
                let base_dir = PathBuf::from(if args.needed_for_users {
                    &manifest.base_for_users_dir
                } else {
                    &manifest.base_dir
                });
                let main_links =
                    links.extract_if(|_from, (to, _is_symlink)| to.starts_with(&base_dir));

                let atomic_dir = base_dir
                    .parent()
                    .map(|p| {
                        p.join(if args.needed_for_users {
                            ".nix-secrets-for-users"
                        } else {
                            ".nix-secrets"
                        })
                    })
                    .wrap_err("Failed to create atomic linking path")?;

                fs::create_dir_all(&atomic_dir).wrap_err_with(|| {
                    eyre!("Failed to atomic linkable directory ({atomic_dir:?})")
                })?;

                for (from, (to, is_symlink)) in main_links {
                    let atomic_path = Path::new(&atomic_dir).join(to.strip_prefix(&base_dir)?);
                    deploy_linkable(&from, &atomic_path, is_symlink)?;
                }

                info!(
                    "Atomically moving temporary directory to target ({atomic_dir:?} -> {base_dir:?})"
                );
                let res = rename_exchange(AT_FDCWD, &atomic_dir, AT_FDCWD, &base_dir);

                if let Err(cleanup_err) = std::fs::remove_dir_all(&atomic_dir)
                    && cleanup_err.kind() != std::io::ErrorKind::NotFound
                {
                    error!(
                        "Warning: failed to clean up {}: {cleanup_err}",
                        atomic_dir.display()
                    );
                }

                res.wrap_err("Failed to atomically replace base directory")?;

                info!("Linked everything in the main directory");

                stage = ActivationStage::LinkIndividualLinkables;
            }
            ActivationStage::LinkIndividualLinkables => {
                for (from, (to, is_symlink)) in &links {
                    deploy_linkable(from, to, *is_symlink)?;
                }
                info!("Linked everything in the different directories");

                stage = ActivationStage::DeleteOldGenerations;
            }
            ActivationStage::DeleteOldGenerations => {
                for directory in &cleanup {
                    trace!("Deleting old generation ({directory:?})",);
                    fs::remove_dir_all(directory).wrap_err_with(|| {
                        format!(
                            "Failed to remove old generation directory: `{}`",
                            directory.display()
                        )
                    })?;
                }

                stage = ActivationStage::Exit;
            }
            ActivationStage::Exit => {}
        }
    }

    Ok(())
}

pub fn rename_exchange<Fd: AsFd>(
    old_dirfd: Fd,
    old_path: &Path,
    new_dirfd: Fd,
    new_path: &Path,
) -> nix::Result<()> {
    #[cfg(target_os = "linux")]
    {
        use nix::fcntl::{RenameFlags, renameat2};
        renameat2(
            old_dirfd,
            old_path,
            new_dirfd,
            new_path,
            RenameFlags::RENAME_EXCHANGE,
        )
    }

    #[cfg(target_os = "macos")]
    {
        use nix::NixPath;
        use std::os::fd::AsRawFd as _;
        let res = old_path.with_nix_path(|c_old| {
            new_path.with_nix_path(|c_new| unsafe {
                libc::renameatx_np(
                    old_dirfd.as_fd().as_raw_fd(),
                    c_old.as_ptr(),
                    new_dirfd.as_fd().as_raw_fd(),
                    c_new.as_ptr(),
                    libc::RENAME_SWAP,
                )
            })
        })??;

        nix::errno::Errno::result(res)?;
        Ok(())
    }
}

fn decrypt_secret(
    secret: &Secret,
    storage: &Path,
    identities: &[Box<dyn Identity + Send + Sync>],
) -> Result<Zeroizing<Vec<u8>>> {
    let path_str = storage
        .join(&secret.name)
        .append_extension(SECRETS_EXTENSION)
        .into_os_string()
        .into_string()
        .map_err(|e| eyre!("Identity secret path contains invalid Unicode: {e:?}"))?;
    trace!("Reading secret `{}` ({})", &secret.name, path_str);

    let input_reader = InputReader::new(Some(path_str))
        .wrap_err_with(|| format!("Failed to open the identity secret `{}`", secret.name))?;
    let mut decrypted = Zeroizing::new(Vec::new());
    decrypt_stream(input_reader, &mut *decrypted, identities)
        .wrap_err_with(|| format!("Failed to decrypt the identity secret `{}`", secret.name))?;

    Ok(decrypted)
}

#[cfg(target_os = "linux")]
fn runtime_directory() -> Result<String> {
    use std::env;

    env::var("XDG_RUNTIME_DIR").wrap_err("$XDG_RUNTIME_DIR is not set")
}

#[cfg(target_os = "macos")]
fn runtime_directory() -> Result<String> {
    use std::process::Command;

    let getconf = Command::new("getconf")
        .arg("DARWIN_USER_TEMP_DIR")
        .output()
        .wrap_err("Failed to execute getconf")?;

    if !getconf.status.success() {
        let stderr = String::from_utf8_lossy(&getconf.stderr);
        return Err(eyre!("Failed to execute getconf")).wrap_err_with(|| stderr.trim().to_owned());
    }

    let output = String::from_utf8(getconf.stdout)
        .wrap_err("Runtime directory contains invalid characters")?;

    let result: &str = output.trim_end_matches(|c: char| c == ' ' || c == '\t' || c == '\n');

    Ok(result.to_string())
}

fn get_generation_directory(module_system: &ModuleSystem, base_dir: &str) -> Result<PathBuf> {
    match module_system {
        ModuleSystem::HomeManager | ModuleSystem::Hjem => {
            let runtime_dir = runtime_directory()?;

            let result = PathBuf::from(base_dir.replace("{{RUNTIME_DIR}}", &runtime_dir));
            Ok(result)
        }
        ModuleSystem::Nixos | ModuleSystem::NixDarwin => Ok(PathBuf::from(base_dir)),
    }
}

fn replace_template_keys(
    hashes: &HashMap<&str, Zeroizing<String>>,
    content: &str,
) -> Result<Zeroizing<Vec<u8>>> {
    let keys: Vec<&&str> = hashes.keys().collect();
    let mut result = Zeroizing::new(Vec::with_capacity(content.len()));

    let ac = AhoCorasick::builder()
        .match_kind(MatchKind::LeftmostFirst)
        .build(&keys)?;

    ac.replace_all_with_bytes(content.as_bytes(), &mut result, |mat, _, dst| {
        let pat = mat.pattern().as_usize();
        keys.get(pat)
            .and_then(|key| hashes.get(*key))
            .map(|value| dst.extend_from_slice(value.trim_end_matches('\n').as_bytes()))
            .is_some()
    });

    Ok(result)
}

fn deploy_linkable_ramfs(
    generation_dir: &Path,
    raw_content: Zeroizing<Vec<u8>>,
    name: &str,
    mode: &str,
    owner: &OwnerOrGroup,
    group: &OwnerOrGroup,
    needed_for_users: bool,
) -> Result<PathBuf> {
    let generation_dst_location = generation_dir.join(name);
    let generation_dst_parent = generation_dst_location
        .parent()
        .ok_or_eyre("Path to linkable has no parent")?;

    trace!(
        "Ensuring linkable directory {generation_dst_parent:?} inside generation directory exists"
    );
    fs::create_dir_all(generation_dst_parent).wrap_err_with(|| {
        eyre!("Failed to create linkable generation directory ({generation_dst_parent:?})")
    })?;

    info!(
        "Linkable `{}` -> {}",
        name,
        generation_dst_location.display()
    );
    let mut file = get_linkable_file(
        &generation_dst_location,
        name,
        mode,
        owner,
        group,
        needed_for_users,
    )?;

    trace!("Writing linkable content `{}`", name);
    file.write_all(&raw_content).wrap_err_with(|| {
        format!(
            "Failed to write template content: `{}`",
            generation_dst_location.display()
        )
    })?;
    drop(raw_content);

    Ok(generation_dst_location)
}

fn deploy_linkable(from: &Path, to: &Path, link: bool) -> Result<()> {
    info!("Creating a temporary directory for linkable");
    let parent = to
        .parent()
        .ok_or_eyre("Path to the linkable has no parent")?;
    let temp_name = format!(
        ".tmp_symlink_{}_{}",
        process::id(),
        time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .wrap_err("Current system time is before the unix epoch")?
            .as_nanos()
    );
    let temp_path = parent.join(temp_name);

    info!("Ensuring parent directory {parent:?} exists");
    fs::create_dir_all(parent)
        .wrap_err_with(|| eyre!("Failed to create secret parent directory ({parent:?})"))?;

    info!("Linking {from:?} -> {to:?} ({temp_path:?})");
    if link {
        unix_fs::symlink(from, &temp_path).wrap_err_with(|| {
            format!(
                "Failed to symlink linkable ({} -> {})",
                from.display(),
                temp_path.display()
            )
        })?;
    } else {
        std::fs::copy(from, &temp_path).wrap_err_with(|| {
            format!(
                "Failed to copy file contents ({} -> {})",
                from.display(),
                temp_path.display()
            )
        })?;
    }
    fs::rename(&temp_path, to).wrap_err_with(|| {
        format!(
            "Failed to move temp linkable to the final path ({} -> {})",
            temp_path.display(),
            to.display()
        )
    })?;

    info!("Successfully deployed linkable");

    Ok(())
}

fn get_linkable_file(
    dst: &Path,
    name: &str,
    mode: &str,
    owner: &OwnerOrGroup,
    group: &OwnerOrGroup,
    needed_for_users: bool,
) -> Result<File> {
    let parsed_mode = utils::parse_permissions_str(mode).wrap_err("Failed to parse permissions")?;

    trace!(
        "Applying file permissions for linkable `{}`: {:o} ({})",
        name,
        parsed_mode,
        dst.display()
    );
    let file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .mode(parsed_mode)
        .open(dst)
        .wrap_err_with(|| format!("Failed to create file: `{}`", dst.display()))?;

    if !needed_for_users {
        trace!(
            "Applying ownership for linkable `{}`: {}:{}",
            name, owner, group
        );
        set_owner_and_group(dst, owner, group)?;
    }

    Ok(file)
}

#[cfg(target_os = "macos")]
fn mount_secret_fs(mountpoint: &Path, _module_system: &ModuleSystem) -> Result<()> {
    use std::process::Command;

    trace!("Creating mount point `{}`", mountpoint.display());
    fs::create_dir_all(&mountpoint).wrap_err_with(|| {
        format!(
            "Failed to create the decrypted mountpoint `{}`",
            mountpoint.display()
        )
    })?;

    let anchor = mountpoint.join(DARWIN_ANCHOR_NAME);
    if anchor.exists() {
        return Ok(());
    }

    let mb = 64;
    let size = mb * 1024 * 1024 / 512;
    let path = format!("ram://{}", size);

    let hdiutil = Command::new("hdiutil")
        .arg("attach")
        .arg("-nomount")
        .arg(path)
        .output()?;

    if !hdiutil.status.success() {
        let stderr = String::from_utf8_lossy(&hdiutil.stderr);

        return Err(eyre!("Failed to create a temporary filesystem"))
            .wrap_err_with(|| stderr.trim().to_owned());
    }

    let diskpath = String::from_utf8_lossy(&hdiutil.stdout);
    let diskpath_trimmed = diskpath.trim();

    let newfs = Command::new("newfs_hfs")
        .arg("-s")
        .arg(diskpath_trimmed)
        .output()?;

    if !newfs.status.success() {
        let stderr = String::from_utf8_lossy(&newfs.stderr);

        return Err(eyre!("Failed to format a new filesystem"))
            .wrap_err_with(|| stderr.trim().to_owned());
    }

    let mount = Command::new("mount")
        .arg("-t")
        .arg("hfs")
        .arg("-o")
        .arg("nobrowse,nodev,nosuid,-m=0751")
        .arg(diskpath_trimmed)
        .arg(mountpoint)
        .output()?;

    if !mount.status.success() {
        let stderr = String::from_utf8_lossy(&mount.stderr);

        return Err(eyre!("Failed to mount the filesystem"))
            .wrap_err_with(|| stderr.trim().to_owned());
    }

    fs::File::create(&anchor)?;

    Ok(())
}

#[cfg(target_os = "linux")]
fn mount_secret_fs(mountpoint: &Path, module_system: &ModuleSystem) -> Result<()> {
    use eyre::bail;
    use nix::mount::{MsFlags, mount};

    trace!("Creating mount point `{}`", mountpoint.display());
    fs::create_dir_all(mountpoint).wrap_err_with(|| {
        format!(
            "Failed to create the decrypted mountpoint `{}`",
            mountpoint.display()
        )
    })?;

    if matches!(
        module_system,
        ModuleSystem::HomeManager | ModuleSystem::Hjem
    ) {
        trace!("ramfs not supported on home-manager and hjem. Skipping."); // TODO: maybe tmpfs?
        return Ok(());
    }

    trace!("Creating ramfs");
    let supports_ramfs = File::open("/proc/filesystems").is_ok_and(|file| {
        BufReader::new(file)
            .lines()
            .map_while(std::result::Result::ok)
            .any(|line| {
                let mut parts = line.split_whitespace();

                match (parts.next(), parts.next()) {
                    (Some("nodev"), Some(fs)) | (Some(fs), None) => fs == "ramfs",
                    _ => false,
                }
            })
    });

    if !supports_ramfs {
        bail!("ramfs not supported! Refusing to extract secrets since it would write to disk");
    }

    mount(
        Some("ramfs"),
        mountpoint,
        Some("ramfs"),
        MsFlags::MS_NOSUID | MsFlags::MS_RELATIME,
        Some("mode=751"),
    )
    .wrap_err("Ramfs creation failed")?;

    Ok(())
}

fn check_generation_dir_existence(mountpoint: &Path) -> Result<bool> {
    match fs::read_dir(mountpoint) {
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e).wrap_err("Failed to read mountpoint")?,
        result::Result::Ok(_) => {
            #[cfg(target_os = "linux")]
            {
                use nix::sys::statfs::statfs;

                let buf =
                    statfs(mountpoint).wrap_err("Failed to get statfs for secret mountpoint")?;
                let magic = buf.filesystem_type();
                Ok(magic == RAMFS_MAGIC)
            }

            #[cfg(target_os = "macos")]
            {
                let anchor = Path::new(mountpoint).join(DARWIN_ANCHOR_NAME);
                return Ok(anchor.exists());
            }
        }
    }
}
