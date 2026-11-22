use crate::{
    command::{Args, CommandTrait},
    utils::{self},
};
use age::cli_common::file_io::InputReader;
use clap::{Parser, ValueHint};
use clap_complete::ArgValueCompleter;
use eyre::{Ok, OptionExt as _, bail, eyre};
use nix_secrets_core::{
    SECRETS_EXTENSION, age::get_identities_paths, file::PathBufExt as _, model::ManifestOptions,
};
use nix_secrets_core::{
    age::{decrypt, decrypt_stream},
    file::check_secret_name,
};
use std::{
    io::{self},
    path::PathBuf,
};

#[derive(Parser, PartialEq, Eq, Debug)]
/// Decrypt a secret and print it to stdout
pub struct DecryptCommand {
    /// Path to the nix-secrets storage (overrides the manifest `storagePath`)
    #[arg(short, long, action = clap::ArgAction::Set, env = "NIX_SECRETS_STORAGE_PATH")]
    storage: Option<PathBuf>,

    /// Name of the secret to decrypt
    #[arg(add = ArgValueCompleter::new(|v: &std::ffi::OsStr| utils::complete_secrets(v, false)))]
    name: String,

    /// Write the decrypted secret to this file instead of stdout
    #[arg(short, long)]
    #[arg(value_hint = ValueHint::FilePath)]
    output: Option<PathBuf>,
}

impl CommandTrait for DecryptCommand {
    fn execute(&self, root: &Args) -> eyre::Result<()> {
        let manifest = utils::eval_manifest(root, &ManifestOptions::default())?;
        let storage_path = self
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
            .join(&secret.name)
            .append_extension(SECRETS_EXTENSION);
        trace!("Got secret path: {}", resulting_path.display());

        if !resulting_path.exists() {
            bail!(
                "Secret `{}` does not exist at `{}`",
                self.name,
                resulting_path.display()
            );
        }

        let identity_paths: Vec<PathBuf> =
            manifest.identity_paths.iter().map(PathBuf::from).collect();
        let identities = get_identities_paths(&identity_paths)?;
        trace!(
            "Decrypting the secret at {resulting_path:?} with {} identities",
            identities.len()
        );

        if let Some(output) = &self.output {
            decrypt(&resulting_path, output, &identities)?;
        } else {
            let stdout = io::stdout().lock();
            let input = InputReader::new(resulting_path.to_str().map(str::to_owned))?;

            decrypt_stream(input, stdout, &identities)?;
        }
        trace!("Decrypted secret successfully");

        Ok(())
    }
}
