use std::{
    env,
    fs::{self, OpenOptions},
    io,
    os::unix::fs::OpenOptionsExt as _,
    path::{Path, PathBuf},
    process,
};

use crate::{
    Result,
    command::{Args, CommandTrait},
    utils::{self},
};
use clap::Parser;
use clap_complete::ArgValueCompleter;
use eyre::{Context as _, OptionExt as _, bail, eyre};
use nix::{errno::Errno, sys::signal::kill, unistd::Pid};
use nix_secrets_core::{
    SECRETS_EXTENSION, age::get_identities_paths, file::PathBufExt as _, model::ManifestOptions,
};
use nix_secrets_core::{
    age::{decrypt, encrypt},
    file::check_secret_name,
};
use scopeguard::defer;

fn editor_hook(path: &Path, editor: &str) -> eyre::Result<()> {
    trace!("Running editor hook with editor: {editor}");
    if utils::is_stdin(editor) {
        let mut src = io::stdin();
        let mut dst = OpenOptions::new()
            // Only the user should be able to read and write this file
            .mode(0o600)
            .create(true)
            .truncate(true)
            .write(true)
            .open(path)
            .wrap_err("Failed to open the input file")?;
        io::copy(&mut src, &mut dst)?;
    } else {
        let (editor_bin, args) = utils::split_editor(editor)?;
        let cmd = process::Command::new(&editor_bin)
            .args(args.unwrap_or_default())
            .arg(path)
            .stdin(process::Stdio::inherit())
            .stdout(process::Stdio::inherit())
            .stderr(process::Stdio::piped())
            .output()
            .wrap_err_with(|| format!("Failed to spawn editor `{editor}`"))?;

        if !cmd.status.success() {
            let stderr = String::from_utf8_lossy(&cmd.stderr);

            return Err(eyre!(
                "Editor `{}` exited with a non-zero status code",
                &editor_bin
            ))
            .wrap_err_with(|| stderr.trim().to_owned());
        }
    }

    Ok(())
}

#[derive(Parser, PartialEq, Eq, Debug)]
/// Edit an encrypted secret in $EDITOR
pub struct EditCommand {
    /// Name of the secret to edit
    #[arg(value_name = "SECRET", add = ArgValueCompleter::new(|v: &std::ffi::OsStr| utils::complete_secrets(v, false)))]
    name: String,

    /// Forcefully edit secret even if another process is using it
    #[arg(long)]
    force: bool,
}

impl CommandTrait for EditCommand {
    fn execute(&self, root: &Args) -> Result<()> {
        let editor = env::var("EDITOR").wrap_err(eyre!("$EDITOR is not set"))?;
        trace!("Using editor: {}", editor);

        let manifest = utils::eval_manifest(root, &ManifestOptions::default())?;
        let storage_path = root
            .storage
            .as_ref()
            .or(manifest.storage_path.as_ref())
            .ok_or_eyre("No storage path provided")?;

        if !storage_path.is_dir() {
            bail!(
                "Storage path `{}` is not a directory",
                storage_path.display()
            );
        }

        let secret = manifest
            .secrets
            .iter()
            .find(|s| s.name.eq(&self.name))
            .ok_or_else(|| {
                eyre!(
                    "Secret `{}` is not present in the Nix configuration",
                    self.name
                )
            })?;

        if !check_secret_name(&secret.name) {
            bail!(
                "Secret name `{}` contains illegal path components",
                secret.name
            )
        }

        let resulting_path = storage_path
            .join(&self.name)
            .append_extension(SECRETS_EXTENSION);
        let dir = env::temp_dir().join(".nix-secrets");

        trace!("Ensuring temporary directory {dir:?} exists");
        fs::create_dir_all(&dir)
            .wrap_err_with(|| eyre!("Failed to create temporary directory ({dir:?})"))?;

        let path_prefix = format!("secret_input_{}", self.name.replace('/', "_"));
        let existing_files = fs::read_dir(&dir)
            .wrap_err_with(|| eyre!("Failed to read temporary directory ({dir:?})"))?;

        for entry in existing_files {
            let path = entry?.path();

            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };

            if !path.is_file() || !name.starts_with(&path_prefix) {
                continue;
            }

            let pid = name
                .rsplit_once('_')
                .and_then(|(_, pid_str)| pid_str.parse::<i32>().ok())
                .ok_or_eyre("No PID in secret name")?;

            let is_being_edited =
                matches!(kill(Pid::from_raw(pid), None), Ok(()) | Err(Errno::EPERM));

            if is_being_edited && !self.force {
                bail!("The secret `{}` is already being edited", secret.name);
            }

            let _ = fs::remove_file(&path);
        }

        let input_path = dir.join(format!(
            "secret_input_{}_{}",
            self.name.replace('/', "_"),
            process::id()
        ));
        trace!("Calculated the input file path: {input_path:?}");

        defer! {
            #[expect(clippy::let_underscore_must_use, reason = "Cleanup")]
            let _: io::Result<()> = fs::remove_file(&input_path);
        }

        if resulting_path.exists() {
            trace!("Secret already exists, editing");
            let identity_paths: Vec<PathBuf> =
                manifest.identity_paths.iter().map(PathBuf::from).collect();
            let identities = get_identities_paths(&identity_paths)?;
            trace!(
                "Decrypting the secret at {resulting_path:?} with {} identities",
                identities.len()
            );
            decrypt(&resulting_path, &input_path, &identities)?;
            trace!("Decrypted secret successfully ({input_path:?})");

            let pre_edit_hash = utils::hash_file(&input_path).wrap_err_with(|| {
                format!(
                    "Failed to calculate file pre-edit hash for {}",
                    input_path.display()
                )
            })?;

            editor_hook(&input_path, &editor).wrap_err("Failed to set up editor hook")?;

            let post_edit_hash = utils::hash_file(&input_path).wrap_err_with(|| {
                format!(
                    "Failed to calculate file post-edit hash for {}",
                    input_path.display()
                )
            })?;

            if pre_edit_hash == post_edit_hash {
                info!(
                    "Secret at `{}` wasn't changed, skipping re-encryption",
                    resulting_path.display()
                );
                fs::remove_file(&input_path)?;
                info!("Done");
                return Ok(());
            }
        } else {
            trace!("Secret does not exist, creating");
            fs::File::create(&input_path)?;

            editor_hook(&input_path, &editor)?;
        }

        let final_parent = resulting_path.parent().ok_or_eyre("Secret has no parent")?;
        trace!("Ensuring parent directory {final_parent:?} exists");
        fs::create_dir_all(final_parent).wrap_err_with(|| {
            eyre!("Failed to create secret parent directory ({final_parent:?})")
        })?;

        encrypt(&input_path, &resulting_path, &secret.recipients)?;

        info!("Done");
        Ok(())
    }
}
