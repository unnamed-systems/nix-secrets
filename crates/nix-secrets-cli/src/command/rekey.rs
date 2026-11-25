use std::{
    collections::{HashMap, HashSet},
    io::Write as _,
    path::PathBuf,
};

use age::cli_common::file_io::{InputReader, OutputFormat, OutputWriter};
use clap_complete::ArgValueCompleter;
use eyre::{Context as _, Ok, OptionExt as _, bail};
use nix_secrets_core::{
    SECRETS_EXTENSION,
    age::{get_identities_paths, rekey_stream},
    file::{PathBufExt as _, check_secret_name},
    model::{ManifestOptions, Secret},
};

use crate::{
    Result,
    command::{Args, CommandTrait},
    utils::{self},
};
use clap::{Parser, ValueHint};

#[derive(Parser, PartialEq, Eq, Debug)]
/// Rekey all or selected encrypted secrets
pub struct RekeyCommand {
    /// Secrets to rekey (rekeys all secrets when empty)
    #[arg(value_name = "SECRET", add = ArgValueCompleter::new(|v: &std::ffi::OsStr| utils::complete_secrets(v, false)))]
    secrets: Vec<String>,
}

impl CommandTrait for RekeyCommand {
    fn execute(&self, root: &Args) -> Result<()> {
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

        let mut requested = self.secrets.iter().collect::<HashSet<_>>();
        let secrets: Vec<Secret> = manifest
            .secrets
            .into_iter()
            .filter(|s| requested.remove(&s.name) || self.secrets.is_empty())
            .collect();

        if let Some(missing) = self.secrets.iter().find(|name| requested.contains(name)) {
            bail!("Secret `{missing}` is not present in the Nix configuration");
        }

        let identity_paths: Vec<PathBuf> =
            manifest.identity_paths.iter().map(PathBuf::from).collect();
        let identities = get_identities_paths(&identity_paths)?;

        let rekeyed: HashMap<&Secret, Vec<u8>> = secrets
            .iter()
            .map(|s| {
                if !check_secret_name(&s.name) {
                    bail!("Secret name `{}` contains illegal path components", s.name)
                }
                let path_str = storage_path
                    .join(&s.name)
                    .append_extension(SECRETS_EXTENSION)
                    .into_os_string()
                    .into_string()
                    .map_err(|e| eyre::eyre!("Secret path contains invalid Unicode: {e:?}"))?;

                trace!("Reading secret `{}` ({})", &s.name, path_str);

                let mut plaintext_buffer = Vec::new();
                let input_reader = InputReader::new(Some(path_str))
                    .wrap_err_with(|| format!("Failed to open the secret `{}`", s.name))?;
                rekey_stream(
                    input_reader,
                    &mut plaintext_buffer,
                    &identities,
                    &s.recipients,
                )
                .wrap_err_with(|| format!("Failed to rekey the secret `{}`", s.name))?;

                info!("Successfully rekeyed secret `{}`", &s.name);
                Ok((s, plaintext_buffer))
            })
            .collect::<eyre::Result<_>>()?;

        trace!("Rekeyed all secrets, writing");

        #[expect(clippy::iter_over_hash_type, reason = "We don't care about order here")]
        for (secret, rekey) in rekeyed {
            let resulting_path = storage_path
                .join(&secret.name)
                .append_extension(SECRETS_EXTENSION);
            let output = resulting_path.to_str().map(String::from);
            let mut writer = OutputWriter::new(output, true, OutputFormat::Text, 0o644, false)
                .wrap_err_with(|| {
                    format!("Failed to open the secret `{}` for writing", secret.name)
                })?;
            writer
                .write_all(&rekey)
                .wrap_err_with(|| format!("Failed to write the secret `{}`", secret.name))?; // TODO: atomic
        }

        info!("Done");

        Ok(())
    }
}
