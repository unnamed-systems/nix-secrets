use std::{
    collections::HashMap,
    env,
    fs::{self, File, OpenOptions, Permissions},
    io::{BufRead as _, BufReader, ErrorKind, Write as _},
    os::unix::fs::{self as unix_fs, PermissionsExt as _},
    path::PathBuf,
    process::{self, Command},
    result, time,
};

use crate::Result;
use crate::{
    SECRETS_DIR, SECRETS_DIR_D, SECRETS_EXTENSION, SECRETS_FOR_USERS_DIR, SECRETS_FOR_USERS_DIR_D,
    command::{Args, CommandTrait},
    manifest::{self, OwnerOrGroup, Secret, Template},
    utils,
};
use age::cli_common::file_io::InputReader;
use clap::{Parser, ValueHint};
use eyre::{Context as _, ContextCompat as _, Ok, OptionExt as _, bail, eyre};

#[derive(Parser, PartialEq, Eq, Debug)]
#[command(hide = true, hide_possible_values = true)]
/// Activate secrets for host
pub struct ActivateCommand {
    /// Path to the manifest file
    #[arg(value_hint = ValueHint::FilePath)]
    pub manifest: PathBuf,

    /// Whether or not to setup secrets before the users are created
    #[arg(long, action = clap::ArgAction::Set, value_parser = clap::value_parser!(bool))]
    pub needed_for_users: bool,
}

impl CommandTrait for ActivateCommand {
    #[allow(clippy::too_many_lines)]
    fn execute(&self, _root: &Args) -> eyre::Result<()> {
        let contents =
            fs::read_to_string(&self.manifest).wrap_err("Failed to read manifest content")?;
        let manifest = manifest::parse_manifest(&contents)?;
        let is_dry = env::var("NIXOS_ACTION").is_ok_and(|val| val == "dry-activate");

        let secrets: Vec<Secret> = manifest
            .secrets
            .into_iter()
            .filter(|i| i.needed_for_users == self.needed_for_users)
            .collect();
        trace!("Filtered secrets ({})", secrets.len());

        let templates: Vec<Template> = manifest
            .templates
            .into_iter()
            .filter(|i| i.needed_for_users == self.needed_for_users)
            .collect();
        trace!("Filtered templates ({})", templates.len());

        if secrets.is_empty() && templates.is_empty() {
            info!("Nothing to deploy");
            return Ok(());
        }

        let identity_paths: Vec<PathBuf> =
            manifest.identity_paths.iter().map(PathBuf::from).collect();
        let identities = utils::get_identities(&identity_paths)?;
        trace!("Found identities ({})", identities.len());

        let plain: HashMap<&Secret, String> = secrets
            .iter()
            .map(|s| {
                if manifest.use_placeholders {
                    trace!(
                        "Using placeholder for secret `{}` ({:?})",
                        s.name, s.placeholder
                    );
                    let placeholder = fs::read_to_string(&s.placeholder)
                        .wrap_err("Failed to read placeholder content")?;
                    return Ok((s, placeholder));
                }

                let path_str = manifest
                    .storage
                    .join(&s.name)
                    .with_extension(SECRETS_EXTENSION)
                    .into_os_string()
                    .into_string()
                    .map_err(|e| eyre!("Invalid unicode path provided: {e:?}"))?;
                trace!("Reading secret `{}` ({})", &s.name, path_str);

                let input_reader = InputReader::new(Some(path_str))
                    .wrap_err("Failed to create an input reader")?;
                let mut decrypted = Vec::new();
                utils::decrypt_stream(input_reader, &mut decrypted, &identities)?;
                trace!("Successfully decrypted secret `{}`", &s.name);

                Ok((s, String::from_utf8(decrypted)?))
            })
            .collect::<eyre::Result<_>>()?;

        let (generation_dir, cleanup) = init_generation_dir(self.needed_for_users)?;
        let secret_generation_dir = generation_dir.join("secrets");
        trace!("Initialized generation directory {generation_dir:?}");

        let resulting_secrets_dir = PathBuf::from(if self.needed_for_users {
            SECRETS_FOR_USERS_DIR
        } else {
            SECRETS_DIR
        })
        .join("secrets");
        trace!("Resulting dir for secret linking: {resulting_secrets_dir:?}");

        let mut links: HashMap<PathBuf, PathBuf> = HashMap::new();

        for secret in &secrets {
            let raw_content = plain
                .get(secret)
                .wrap_err_with(|| eyre!("Decrypted content not found"))?;

            let generation_dst_location = secret_generation_dir.join(&secret.name);
            let generation_dst_parent = generation_dst_location
                .parent()
                .ok_or_eyre("Path to secret has no parent")?;

            trace!(
                "Ensuring secret directory {generation_dst_parent:?} inside generation directory exists"
            );
            fs::create_dir_all(generation_dst_parent).wrap_err_with(|| {
                eyre!(
                    "Failed to create secret generation directory: `{}",
                    generation_dst_parent.display()
                )
            })?;

            info!(
                "Secret `{}` -> {}",
                secret.name,
                generation_dst_location.display()
            );
            let mut the_file = get_linkable_file(
                &generation_dst_location,
                &secret.name,
                &secret.mode,
                &secret.owner,
                &secret.group,
                secret.needed_for_users,
            )?;

            trace!("Writing secret content `{}`", secret.name);
            the_file
                .write_all(raw_content.as_bytes())
                .wrap_err_with(|| {
                    format!(
                        "Failed to write secret content: `{}`",
                        generation_dst_location.display()
                    )
                })?;

            let final_directory =
                get_linkable_directory(&resulting_secrets_dir, &secret.name, &secret.path)?;

            links.insert(generation_dst_location, final_directory);
        }

        let templates_generation_dir = generation_dir.join("templates");
        trace!("Initialized generation directory {generation_dir:?}");

        let hashes: HashMap<&String, String> = plain
            .into_iter()
            .map(|(k, v)| (&k.template_key, v))
            .collect();

        let resulting_templates_dir = PathBuf::from(if self.needed_for_users {
            SECRETS_FOR_USERS_DIR
        } else {
            SECRETS_DIR
        })
        .join("templates");
        trace!("Resulting dir for template linking: {resulting_templates_dir:?}");

        for template in templates {
            trace!("Reading template content from {:?}", template.content);
            let content = fs::read_to_string(&template.content).wrap_err_with(|| {
                format!(
                    "Failed to read template content: `{}`",
                    template.content.display()
                )
            })?;
            trace!("Replacing secret hashes");
            let raw_content = &hashes.iter().fold(content, |c, (key, value)| {
                c.replace(*key, value.trim_end_matches('\n')) // TODO: find a better, customizable way
            });

            let generation_dst_location = templates_generation_dir.join(&template.name);
            let generation_dst_parent = generation_dst_location
                .parent()
                .ok_or_eyre("Path to secret has no parent")?;

            trace!(
                "Ensuring template directory {generation_dst_parent:?} inside generation directory exists"
            );
            fs::create_dir_all(generation_dst_parent).wrap_err_with(|| {
                eyre!("Failed to create secret generation directory ({generation_dst_parent:?})")
            })?;

            info!(
                "Template `{}` -> {}",
                template.name,
                generation_dst_location.display()
            );
            let mut the_file = get_linkable_file(
                &generation_dst_location,
                &template.name,
                &template.mode,
                &template.owner,
                &template.group,
                template.needed_for_users,
            )?;

            trace!("Writing template content `{}`", template.name);
            the_file
                .write_all(raw_content.as_bytes())
                .wrap_err_with(|| {
                    format!(
                        "Failed to write template content: `{}`",
                        generation_dst_location.display()
                    )
                })?;

            let final_directory =
                get_linkable_directory(&resulting_templates_dir, &template.name, &template.path)?;

            links.insert(generation_dst_location, final_directory);
        }

        info!("Finished linkable creation, linking");

        #[expect(clippy::iter_over_hash_type, reason = "We don't care about order here")]
        for (from, to) in links {
            if is_dry {
                trace!("Dry, skipping linking ({from:?} -> {to:?})");
                continue;
            }
            deploy_linkable(&from, &to)?;
        }
        info!("Linked everything, cleaning up");

        for directory in cleanup {
            trace!(
                "{} old generation ({directory:?})",
                if is_dry {
                    "Would delete (dry)"
                } else {
                    "Deleting"
                }
            );
            if !is_dry {
                fs::remove_dir_all(&directory).wrap_err_with(|| {
                    format!(
                        "Failed to remove old generation directory: `{}`",
                        directory.display()
                    )
                })?;
            }
        }

        Ok(())
    }
}

fn get_linkable_directory(base: &PathBuf, name: &String, path: &PathBuf) -> Result<PathBuf> {
    if path == &base.join(name) {
        trace!("Using default path for linkable `{}`", &name);
        Ok(base.join(name))
    } else {
        if path.starts_with(base) {
            warn!(
                "Extraction inside the decrypted directory detected. It is recommend to specify `name` instead of `path`"
            );
        }
        trace!(
            "Using specified path for linkable `{}` ({})",
            name,
            path.display()
        );
        Ok(path.clone()) // TODO: attempt zerocopy
    }
}

fn deploy_linkable(from: &PathBuf, to: &PathBuf) -> Result<()> {
    trace!("Creating a temporary directory for linkable");
    let parent = to.parent().ok_or_eyre("Path to secret is a directory")?;
    let temp_name = format!(
        ".tmp_symlink_{}_{}",
        process::id(),
        time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .wrap_err("Current system time is before the unix epoch")?
            .as_nanos()
    );
    let temp_path = parent.join(temp_name);

    trace!("Ensuring parent directory {parent:?} exists");
    fs::create_dir_all(parent)
        .wrap_err_with(|| eyre!("Failed to create secret parent directory ({parent:?})"))?;

    trace!("Symlinking {from:?} -> {to:?} ({temp_path:?})");
    unix_fs::symlink(from, &temp_path).wrap_err_with(|| {
        format!(
            "Failed to symlink linkable ({} -> {})",
            from.display(),
            temp_path.display()
        )
    })?;
    fs::rename(&temp_path, to).wrap_err_with(|| {
        format!(
            "Failed to move temp linkable to the final path ({} -> {})",
            temp_path.display(),
            to.display()
        )
    })?;

    trace!("Successfully deployed linkable");

    Ok(())
}

fn get_linkable_file(
    dst: &PathBuf,
    name: &str,
    mode: &str,
    owner: &OwnerOrGroup,
    group: &OwnerOrGroup,
    needed_for_users: bool,
) -> Result<File> {
    let parsed_mode = utils::parse_permissions_str(mode).wrap_err("Failed to parse permissions")?;
    let permissions = Permissions::from_mode(parsed_mode);

    trace!(
        "Applying file permissions for linkable `{}`: {:o} ({})",
        name,
        permissions.mode(),
        dst.display()
    );
    let file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(dst)
        .wrap_err_with(|| format!("Failed to open file: `{}`", dst.display()))?;
    file.set_permissions(permissions)
        .wrap_err_with(|| format!("Failed to set file permissions: `{}`", dst.display()))?;

    if !needed_for_users {
        trace!(
            "Applying ownership for linkable `{}`: {}:{}",
            name, owner, group
        );
        utils::set_owner_and_group(dst, owner, group)?;
    }

    Ok(file)
}

#[cfg(target_os = "macos")]
fn mount_secret_fs(mountpoint: &PathBuf) -> Result<()> {
    trace!("Creating mount point `{}`", mountpoint.display());
    fs::create_dir_all(&mountpoint).wrap_err_with(|| {
        format!(
            "Failed to create the decrypted mountpoint `{}`",
            mountpoint.display()
        )
    })?;

    let anchor = mountpoint.join("nix-secrets-anchor");
    if anchor.exists() {
        return Ok(());
    }

    let mb = 64;
    let size = mb * 1024 * 1024 / 512;
    let path = format!("ram://{}", size);

    let hdutil = Command::new("hdutil")
        .arg("attach")
        .arg("-nomount")
        .arg(path)
        .output()?;

    if !hdutil.status.success() {
        let stderr = String::from_utf8_lossy(&hdutil.stderr);

        return Err(eyre!("Failed to create a temporary filesystem"))
            .wrap_err_with(|| stderr.trim().to_owned());
    }

    let diskpath = String::from_utf8_lossy(&hdutil.stdout);
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
fn mount_secret_fs(mountpoint: &PathBuf) -> Result<()> {
    use rustix::mount::{MountFlags, mount};

    let supports_ramfs = File::open("/proc/filesystems")
        .map(|file| {
            BufReader::new(file)
                .lines()
                .filter_map(|line| line.ok())
                .any(|line| {
                    let mut parts = line.split_whitespace();

                    match (parts.next(), parts.next()) {
                        (Some("nodev"), Some(fs)) => fs == "ramfs",
                        (Some(fs), None) => fs == "ramfs",
                        _ => false,
                    }
                })
        })
        .unwrap_or(false);

    if !supports_ramfs {
        bail!("ramfs not supported! Refusing extract secret since it will write to disk");
    }
    trace!("Creating mount point `{}`", mountpoint.display());
    fs::create_dir_all(&mountpoint).wrap_err_with(|| {
        format!(
            "Failed to create the decrypted mountpoint `{}`",
            mountpoint.display()
        )
    })?;
    trace!("Creating ramfs");

    mount(
        "ramfs",
        mountpoint,
        "ramfs",
        MountFlags::NOSUID | MountFlags::RELATIME,
        Some(c"mode=751"),
    )
    .wrap_err("Ramfs creation failed")?;

    Ok(())
}

fn init_generation_dir(needed_for_users: bool) -> eyre::Result<(PathBuf, Vec<PathBuf>)> {
    let mut max = 0;

    let gen_dir = PathBuf::from(if needed_for_users {
        SECRETS_FOR_USERS_DIR_D
    } else {
        SECRETS_DIR_D
    });

    let mut cleanup: Vec<PathBuf> = Vec::new();

    trace!("Base generation directory: `{}`", gen_dir.display());

    let res = match fs::read_dir(&gen_dir) {
        Err(e) if e.kind() == ErrorKind::NotFound => {
            trace!("Base generation directory not found, creating ramfs");
            mount_secret_fs(&gen_dir)?;
            Ok(())
        }
        Err(e) => Err(e).wrap_err("Failed to read mountpoint")?,
        result::Result::Ok(_) => {
            trace!("Base generation directory exists, creating new generation");

            fs::read_dir(&gen_dir)
                .wrap_err_with(|| eyre!("Failed to read base generation directory: {gen_dir:?}"))
                .and_then(|mut o| {
                    o.try_for_each(|en| {
                        let d = en.wrap_err_with(|| {
                            eyre!("Failed to enter generation directory subdir")
                        })?;
                        match str::parse::<usize>(
                            d.file_name().to_string_lossy().to_string().as_str(),
                        ) {
                            Err(e) => Err(eyre!("Failed to parse generation: {e}")),
                            result::Result::Ok(res) => {
                                trace!("Found existing generation {res} in {gen_dir:?}");
                                if res >= max {
                                    max = res.checked_add(1).unwrap_or_default();
                                }

                                if max.checked_sub(res).is_some_and(|d| d > 1) {
                                    cleanup.push(d.path());
                                }
                                Ok(())
                            }
                        }
                    })
                })
        }
    };
    trace!("Calculated new generation id: {}", max);
    trace!("{} old generation directories to remove", cleanup.len());

    res.map(|()| (gen_dir.join(max.to_string()), cleanup))
}
