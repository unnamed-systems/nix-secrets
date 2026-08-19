use std::{
    env,
    fs::{self, OpenOptions},
    io,
    os::unix::fs::OpenOptionsExt as _,
    path::{Path, PathBuf},
    process,
};

use crate::{
    Result, SECRETS_EXTENSION,
    command::{Args, CommandTrait},
    utils::{self, PathBufExt as _},
};
use clap::Parser;
use clap_complete::ArgValueCompleter;
use eyre::{Context as _, OptionExt as _, bail, eyre};
use scopeguard::defer;

fn editor_hook(path: &Path, editor: &str) -> eyre::Result<()> {
    trace!("Running editor hook with editor: {editor}");
    if utils::is_stdin(editor) {
        let mut src = io::stdin();
        let mut dst = OpenOptions::new()
            // Stdin should be able to write and read
            .mode(0o600)
            .create(true)
            .truncate(true)
            .write(true)
            .open(path)
            .wrap_err("Failed to open stdin")?;
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
            .wrap_err_with(|| format!("Failed to spawn editor '{editor}'"))?;

        if !cmd.status.success() {
            let stderr = String::from_utf8_lossy(&cmd.stderr);

            return Err(eyre!(
                "Editor '{}' exited with non-zero status code",
                &editor_bin
            ))
            .wrap_err_with(|| stderr.trim().to_owned());
        }
    }

    Ok(())
}

#[derive(Parser, PartialEq, Eq, Debug)]
/// Edit an encrypted secret
pub struct EditCommand {
    /// Path to the secrets directory
    #[arg(short, long, action = clap::ArgAction::Set, env = "NIX_SECRETS_STORAGE_PATH")]
    storage: Option<PathBuf>,

    /// Secret name
    #[arg(add = ArgValueCompleter::new(|v: &std::ffi::OsStr| utils::complete_secrets(v, false)))]
    name: String,
}

impl CommandTrait for EditCommand {
    fn execute(&self, root: &Args) -> Result<()> {
        let (flake, hostname) =
            utils::parse_flake(&root.flake, true).ok_or_eyre("Failed to parse flake")?;

        trace!("Parsed flake: {}, hostname: {}", flake, hostname);

        let editor = env::var("EDITOR").wrap_err(eyre!("$EDITOR is not set"))?;
        trace!("Using editor: {}", editor);

        let manifest = utils::eval_manifest(&flake, &hostname)?;
        let storage_path = self
            .storage
            .as_ref()
            .or(manifest.storage_path.as_ref())
            .ok_or_eyre("No storage path provided")?;

        if !storage_path.is_dir() {
            bail!("Invalid directory path");
        }

        let secret = manifest
            .secrets
            .iter()
            .find(|s| s.name.eq(&self.name))
            .ok_or_eyre("Secret not present in nix config.")?;

        let resulting_path = storage_path
            .join(&self.name)
            .append_extension(SECRETS_EXTENSION);
        let dir = env::temp_dir().join(".nix-secrets");

        trace!("Ensuring temporary directory {dir:?} exists");
        fs::create_dir_all(&dir)
            .wrap_err_with(|| eyre!("Failed to create temporary directory ({dir:?})"))?;

        let input_path = dir.join(format!("secret_input_{}", self.name.replace('/', "_")));
        trace!("Calculated the input file path: {input_path:?}");

        if input_path.exists() {
            bail!("The secret `{}` is already being edited", secret.name);
        }

        defer! {
            #[expect(clippy::let_underscore_must_use, reason = "Cleanup")]
            let _: io::Result<()> = fs::remove_file(&input_path);
        }

        if resulting_path.exists() {
            trace!("Secret already exists, editing");
            let identity_paths: Vec<PathBuf> =
                manifest.identity_paths.iter().map(PathBuf::from).collect();
            let identities = utils::get_identities(&identity_paths)?;
            trace!(
                "Decrypting the secret at {resulting_path:?} with {} identities",
                identities.len()
            );
            utils::decrypt(&resulting_path, &input_path, &identities)?;
            trace!("Decrypted secret successfully");

            let pre_edit_hash =
                utils::hash_file(&input_path).wrap_err("Failed to calculate file pre-edit hash")?;

            editor_hook(&input_path, &editor).wrap_err("Failed to set up editor hook")?;

            let post_edit_hash = utils::hash_file(&input_path)
                .wrap_err("Failed to calculate file post-edit hash")?;

            if pre_edit_hash == post_edit_hash {
                info!(
                    "Secret {} wasn't changed, skipping re-encryption",
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

        utils::encrypt(&input_path, &resulting_path, &secret.recipients)?;

        info!("Done");
        Ok(())
    }
}
