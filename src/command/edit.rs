use std::{
    env,
    fs::{self, OpenOptions},
    io,
    os::unix::fs::{OpenOptionsExt as _, PermissionsExt},
    path::{Path, PathBuf},
    process,
    time::SystemTime,
};

use argh::{ArgsInfo, FromArgs};
use eyre::{Context as _, Ok, OptionExt as _, bail, eyre};

use crate::{
    Result, SECRETS_EXTENSION,
    command::{Args, CommandTrait},
    utils::{self},
};

fn editor_hook(path: &Path, editor: &str) -> eyre::Result<()> {
    if utils::is_stdin(editor) {
        let mut src = io::stdin();
        let mut dst = OpenOptions::new()
            // Stdin should be able to write and read
            .mode(0o600)
            .create(true)
            .truncate(true)
            .write(true)
            .open(path)?;
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
            .wrap_err_with(|| format!("Failed to spawn editor '{editor_bin}'"))?;

        if !cmd.status.success() {
            let stderr = String::from_utf8_lossy(&cmd.stderr);

            return Err(eyre!(
                "Editor '{}' exited with non-zero status code",
                &editor_bin
            ))
            .with_context(|| stderr.trim().to_owned());
        }
    }

    Ok(())
}

#[derive(FromArgs, ArgsInfo, PartialEq, Eq, Debug)]
/// Edit an encrypted secret
#[argh(subcommand, name = "edit")]
pub struct EditCommand {
    /// path to the secrets directory
    #[argh(option)]
    directory: PathBuf,

    /// secret name
    #[argh(positional)]
    name: String,
}

impl CommandTrait for EditCommand {
    fn execute(&self, root: &Args) -> Result<()> {
        if !self.directory.is_dir() {
            bail!("Invalid directory path");
        }

        let (flake, hostname) =
            utils::parse_flake(&root.flake).ok_or_eyre("Failed to parse flake")?;

        trace!("Parsed flake: {}, hostname: {}", flake, hostname);

        let editor = env::var("EDITOR").wrap_err(eyre!("$EDITOR is not set"))?;
        trace!("Using editor: {}", editor);

        let manifest = utils::eval_manifest(&flake, &hostname)?;

        let resulting_path = self
            .directory
            .join(&self.name)
            .with_extension(SECRETS_EXTENSION);
        let dir = env::temp_dir();
        let input_path = dir.join(format!(
            "secret_input_{}_{}_{}",
            self.name,
            process::id(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_nanos()
        ));

        let secret = manifest
            .secrets
            .iter()
            .find(|s| s.name.eq(&self.name))
            .ok_or_eyre("Secret not present in nix config.")?;

        if resulting_path.exists() {
            let identities =
                utils::get_identities(manifest.identity_paths.iter().map(PathBuf::from).collect())?;
            utils::decrypt(&resulting_path, &input_path, &identities)?;

            let pre_edit_hash = utils::hash_file(&input_path)?;

            editor_hook(&input_path, &editor)?;

            let post_edit_hash = utils::hash_file(&input_path)?;

            if pre_edit_hash == post_edit_hash {
                info!(
                    "{} wasn't changed, skipping re-encryption.",
                    resulting_path.display()
                );
                fs::remove_file(input_path)?;
                return Ok(());
            }
        } else {
            // We should be able to write and read
            fs::set_permissions(&dir, PermissionsExt::from_mode(0o600))?;

            fs::File::create(&input_path)?;
            editor_hook(&input_path, &editor)?;
        }

        utils::encrypt(&input_path, &resulting_path, &secret.recipients)?;
        fs::remove_file(input_path)?;

        Ok(())
    }
}
