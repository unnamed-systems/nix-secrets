use std::{
    collections::HashMap, env, fs::{self, OpenOptions, Permissions}, io::{ErrorKind, Read as _, Write as _}, os::unix::fs::{self as unix_fs, PermissionsExt as _}, path::PathBuf, process, time,
};

use crate::{
    SECRETS_DIR, SECRETS_DIR_D, SECRETS_EXTENSION, SECRETS_FOR_USERS_DIR, SECRETS_FOR_USERS_DIR_D, command::{Args, CommandTrait}, manifest::{self, Secret}, utils,
};
use age::{Decryptor, armor::ArmoredReader, cli_common::file_io::InputReader};
use argh::{ArgsInfo, FromArgs};
use eyre::{Context as _, ContextCompat as _, Ok, OptionExt as _, bail, eyre};
use sys_mount::{Mount, MountFlags, SupportedFilesystems};

#[derive(FromArgs, ArgsInfo, PartialEq, Eq, Debug)]
/// Activate secrets for host
#[argh(subcommand, name = "activate")]
pub struct ActivateCommand {
    #[argh(positional)]
    pub manifest: PathBuf,

    /// whether or not to setup secrets before the users are created
    #[argh(option)]
    pub needed_for_users: bool,
}

impl CommandTrait for ActivateCommand {
    fn execute(&self, _root: &Args) -> eyre::Result<()> {
        let contents = fs::read_to_string(&self.manifest)?;
        let manifest = manifest::parse_manifest(&contents)?;
        let is_dry = env::var("NIXOS_ACTION").is_ok_and(|val| val == "dry-activate");

        let secrets: Vec<Secret> = manifest
            .secrets
            .into_iter()
            .filter(|i| i.needed_for_users == self.needed_for_users)
            .collect();
        
        trace!("Filtered secrets ({})", secrets.len());

        if secrets.is_empty() {
            info!("Nothing to deploy");
            return Ok(());
        }

        let identity_paths: Vec<PathBuf> = manifest.identity_paths.iter().map(PathBuf::from).collect();
        let identities =
            utils::get_identities(&identity_paths)?;
        trace!("Found identities ({})", identities.len());

        let plain: HashMap<&str, String> = secrets
            .iter()
            .map(|s| {
                let path_str = manifest
                    .storage
                    .join(&s.name)
                    .with_extension(SECRETS_EXTENSION)
                    .into_os_string()
                    .into_string()
                    .map_err(|e| eyre::eyre!("Invalid unicode path provided: {e:?}"))?;

                trace!("Reading secret `{}` ({})", &s.name, path_str);

                let input_reader = InputReader::new(Some(path_str))?;
                let decryptor = Decryptor::new(ArmoredReader::new(input_reader))?;
                let mut decrypted_string = String::new();

                #[expect(clippy::as_conversions, reason = "dyn Identity + Send -> dyn Identity")]
                decryptor
                    .decrypt(identities.iter().map(|i| i.as_ref() as &dyn age::Identity))?
                    .read_to_string(&mut decrypted_string)?;

                trace!("Successfully decrypted secret `{}`", &s.name);
                Ok((s.name.as_str(), decrypted_string))
            })
            .collect::<eyre::Result<_>>()?;

        let (generation_dir, cleanup) = init_generation_dir(self.needed_for_users, is_dry)?;
        trace!("Initialized generation directory {generation_dir:?}");

        let resulting_dir = PathBuf::from(if self.needed_for_users {
            SECRETS_FOR_USERS_DIR
        } else {
            SECRETS_DIR
        });
        trace!("Resulting dir for secret extraction: {resulting_dir:?}");

        secrets
            .iter()
            .map(|s| {
                let raw_content = plain
                    .get(s.name.as_str())
                    .wrap_err_with(|| eyre!("Decrypted content not found"))?;

                let generation_dst_location = generation_dir.join(&s.name);
                let generation_dst_parent = generation_dst_location.parent().ok_or_eyre("Path to secret is a directory")?;
                
                trace!("Ensuring secret directory {generation_dst_parent:?} inside generation directory exists");
                fs::create_dir_all(generation_dst_parent)
                .wrap_err_with(|| {
                    eyre!("Failed to create secret generation directory ({generation_dst_parent:?})")
                })?;

                let dst = if s.path == resulting_dir.join(&s.name) {
                    trace!("Using default path for secret `{}`", s.name);
                    &resulting_dir.join(&s.name)
                } else {
                    if s.path.starts_with(&resulting_dir) {
                        warn!("Extraction inside the decrypted directory detected. It is recommend to specify `name` instead of `path`.");
                    }
                    trace!("Using specified path for secret `{}` ({})", s.name, s.path.display());
                    &s.path
                };

                info!("Secret `{}` -> {}", s.name, generation_dst_location.display());

                let mut the_file = {
                    let mode = utils::parse_permissions_str(&s.mode)
                        .map_err(|e| eyre!("Failed to parse permissions: {}", e))?;
                    let permissions = Permissions::from_mode(mode);

                    trace!(
                        "Applying file permissions for secret `{}`: {:o} ({})",
                        s.name,
                        permissions.mode(),
                        generation_dst_location.display()
                    );
                    let file = OpenOptions::new()
                        .create(true)
                        .truncate(true)
                        .write(true)
                        .open(&generation_dst_location)?;
                    file.set_permissions(permissions)?;

                    trace!(
                        "Applying ownership for secret `{}`: {}:{}",
                        s.name, s.owner, s.group
                    );
                    utils::set_owner_and_group(
                        &generation_dst_location,
                        &s.owner,
                        &s.group,
                    )?;

                    file
                };

                trace!("Writing secret content `{}`", s.name);
                the_file.write_all(raw_content.as_bytes())?;

                if is_dry {
                    info!("Skipping symlinking (dry)");
                    return Ok(());
                }

                trace!("Creating a temporary directory for secret `{}`", s.name);
                let parent = dst.parent().ok_or_eyre("Path to secret is a directory")?;
                let temp_name = format!(
                    ".tmp_symlink_{}_{}",
                    process::id(),
                    time::SystemTime::now()
                        .duration_since(time::UNIX_EPOCH)?
                        .as_nanos()
                );
                let temp_path = parent.join(temp_name);

                trace!("Ensuring parent directory {parent:?} exists");
                fs::create_dir_all(parent)
                .wrap_err_with(|| {
                    eyre!("Failed to create secret parent directory ({parent:?})")
                })?;

                trace!("Symlinking {generation_dst_location:?} -> {dst:?} ({temp_path:?})");
                unix_fs::symlink(&generation_dst_location, &temp_path)?;
                fs::rename(&temp_path, dst)?;

                trace!("Successfully deployed secret `{}`", s.name);
                Ok(())
            })
            .for_each(|res| {
                if let Err(e) = res {
                    error!("{e}");
                }
            });

        info!("Finished secrets deployment, cleaning up");

        for directory in cleanup {
            trace!("Removing old directory: {directory:?}");
            fs::remove_dir_all(directory)?;
        }

        Ok(())
    }
}

pub fn init_generation_dir(needed_for_users: bool, is_dry: bool) -> eyre::Result<(PathBuf, Vec<PathBuf>)> {
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
            let support_ramfs = SupportedFilesystems::new().map(|fss| fss.is_supported("ramfs"));
            if !support_ramfs? {
                bail!("ramfs not supported! Refusing extract secret since it will write to disk");
            }
            trace!("Creating mount point `{}`", gen_dir.display());
            fs::create_dir_all(&gen_dir).wrap_err_with(|| {
                format!(
                    "Creating decrypted mountpoint `{}` failed",
                    gen_dir.display()
                )
            })?;
            trace!("Creating ramfs");
            Mount::builder()
                .fstype("ramfs")
                .flags(MountFlags::NOSUID)
                .data("relatime")
                .data("mode=751")
                .mount(String::default(), &gen_dir)
                .wrap_err(eyre!("Failed to mount ramfs"))?;
            Ok(())
        }
        Err(e) => {
            error!("{e}");
            Err(e).wrap_err(eyre!("Failed to read mountpoint"))?
        }
        Result::Ok(_) => {
            trace!("Base generation directory exists, creating new generation");

            fs::read_dir(&gen_dir)
                .wrap_err_with(|| eyre!("Failed to read base generation directory: {gen_dir:?}"))
                .and_then(|mut o| {
                    o.try_for_each(|en| {
                        let d = en.wrap_err_with(|| eyre!("Failed to enter generation directory subdir."))?;
                        match str::parse::<usize>(
                                                            d.file_name().to_string_lossy().to_string().as_str(),
                                                        ) {
                                                            Err(e) => Err(eyre!("Failed to parse generation: {e}")),
                                                            Result::Ok(res) => {
                                                                trace!("Found existing generation {res} in {gen_dir:?}");
                                                                if res >= max {
                                                                    max = res + 1;
                                                                }

                                                                if max - res > 1 {
                                                                    trace!(
                                                                        "Found old generation ({}). Deleting{}.",
                                                                        res,
                                                                        if is_dry { " (dry)" } else { "" }
                                                                    );
                                                                    if !is_dry {
                                                                        cleanup.push(d.path());
                                                                    }
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

    res.map(|()| {
        (gen_dir.join(max.to_string()), cleanup)
    })
}
