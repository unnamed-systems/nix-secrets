use std::{
    env,
    fs::{self, OpenOptions},
    io,
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
    process,
};

use argh::{ArgsInfo, FromArgs};
use eyre::{Context, Ok, OptionExt, bail, eyre};

use crate::{
    command::{Args, CommandTrait},
    utils::{self},
};

fn editor_hook(path: &Path, editor: &str) -> eyre::Result<()> {
    if utils::is_stdin(editor) {
        let mut src = io::stdin();
        let mut dst = OpenOptions::new()
            .mode(0o600)
            .create(true)
            .truncate(true)
            .write(true)
            .open(path)?;
        io::copy(&mut src, &mut dst)?;
    } else {
        let (editor, args) = utils::split_editor(editor)?;
        let cmd = process::Command::new(&editor)
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
                &editor
            ))
            .with_context(|| stderr.trim().to_string());
        }
    }

    Ok(())
}

#[derive(FromArgs, ArgsInfo, PartialEq, Debug)]
/// Edit an encrypted secret
#[argh(subcommand, name = "edit")]
pub(crate) struct EditCommand {
    /// path to the secrets directory
    #[argh(option)]
    directory: PathBuf,

    /// secret name
    #[argh(positional)]
    name: String,
}

impl CommandTrait for EditCommand {
    fn execute(&self, root: &Args) -> eyre::Result<()> {
        if !self.directory.is_dir() {
            bail!("Invalid directory path");
        }

        let (flake, hostname) =
            utils::parse_flake(&root.flake).ok_or(eyre!("Failed to parse flake"))?;

        trace!("Parsed flake: {}, hostname: {}", flake, hostname);

        let editor = env::var("EDITOR").wrap_err(eyre!("$EDITOR is not set"))?;
        let manifest = utils::eval_manifest(&flake, &hostname)?;

        let resulting_path = self.directory.join(&self.name).with_extension("enc");
        let dir = env::temp_dir();
        let input_path = dir.join(format!(
            "secret_input_{}_{}_{}",
            self.name,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
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
            fs::set_permissions(&dir, PermissionsExt::from_mode(0o700))?;

            println!("Secret data: {}", secret.name);
            fs::File::create(&input_path)?;
            editor_hook(&input_path, &editor)?;
        }

        utils::encrypt(&input_path, &resulting_path, &secret.recipients)?;
        fs::remove_file(input_path)?;

        Ok(())
    }
}
