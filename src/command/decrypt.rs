use crate::{
    SECRETS_EXTENSION,
    command::{Args, CommandTrait},
    utils::{self, PathBufExt as _, check_secret_name},
};
use age::cli_common::file_io::InputReader;
use clap::{Parser, ValueHint};
use clap_complete::ArgValueCompleter;
use eyre::{Ok, OptionExt as _, bail};
use std::{
    io::{self},
    path::PathBuf,
};

#[derive(Parser, PartialEq, Eq, Debug)]
/// Decrypt a secret
pub struct DecryptCommand {
    /// Path to the secrets directory
    #[arg(short, long, action = clap::ArgAction::Set, env = "NIX_SECRETS_STORAGE_PATH")]
    storage: Option<PathBuf>,

    /// Secret name
    #[arg(add = ArgValueCompleter::new(|v: &std::ffi::OsStr| utils::complete_secrets(v, false)))]
    name: String,

    /// Path to the output file
    #[arg(short, long)]
    #[arg(value_hint = ValueHint::FilePath)]
    output: Option<PathBuf>,
}

impl CommandTrait for DecryptCommand {
    fn execute(&self, root: &Args) -> eyre::Result<()> {
        let (flake, module_system, hostname) =
            utils::parse_flake(&root.flake, true).ok_or_eyre("Failed to parse flake")?;

        trace!("Parsed flake: {}, hostname: {}", flake, hostname);

        let manifest = utils::eval_manifest(&flake, module_system.as_deref(), &hostname)?;
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

        if !check_secret_name(&secret.name) {
            bail!("Secret name `{}` contains illegal values", secret.name)
        }

        let resulting_path = storage_path
            .join(&secret.name)
            .append_extension(SECRETS_EXTENSION);
        trace!("Got secret path: {}", resulting_path.display());

        if !resulting_path.exists() {
            bail!("Secret {} does not exist", self.name);
        }

        let identity_paths: Vec<PathBuf> =
            manifest.identity_paths.iter().map(PathBuf::from).collect();
        let identities = utils::get_identities(&identity_paths)?;
        trace!(
            "Decrypting the secret at {resulting_path:?} with {} identities",
            identities.len()
        );

        if let Some(output) = &self.output {
            utils::decrypt(&resulting_path, output, &identities)?;
        } else {
            let stdout = io::stdout().lock();
            let input = InputReader::new(resulting_path.to_str().map(str::to_owned))?;
            utils::decrypt_stream(input, stdout, &identities)?;
        }
        trace!("Decrypted secret successfully");

        Ok(())
    }
}
