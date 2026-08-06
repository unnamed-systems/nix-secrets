use std::{collections::HashMap, io::Write as _, path::PathBuf};

use age::cli_common::file_io::{InputReader, OutputFormat, OutputWriter};
use eyre::{Ok, OptionExt as _, bail};

use crate::{
    Result, SECRETS_EXTENSION,
    command::{Args, CommandTrait},
    manifest::Secret,
    utils::{self, rekey_stream},
};
use clap::{Parser, ValueHint};

#[derive(Parser, PartialEq, Eq, Debug)]
/// Rekey all or selected encrypted secrets
pub struct RekeyCommand {
    /// path to the secrets directory
    #[arg(short, long)]
    #[arg(value_hint = ValueHint::DirPath, env = "NIX_SECRETS_STORAGE_PATH")]
    storage: PathBuf,

    /// secrets to rekey. All if empty
    secrets: Vec<String>,
}

impl CommandTrait for RekeyCommand {
    fn execute(&self, root: &Args) -> Result<()> {
        if !self.storage.is_dir() {
            bail!("Invalid directory path");
        }

        let (flake, hostname) =
            utils::parse_flake(&root.flake).ok_or_eyre("Failed to parse flake")?;

        trace!("Parsed flake: {}, hostname: {}", flake, hostname);

        let manifest = utils::eval_manifest(&flake, &hostname)?;
        let secrets: Vec<Secret> = manifest
            .secrets
            .into_iter()
            .filter(|s| self.secrets.contains(&s.name) || self.secrets.is_empty())
            .collect();

        let identity_paths: Vec<PathBuf> =
            manifest.identity_paths.iter().map(PathBuf::from).collect();
        let identities = utils::get_identities(&identity_paths)?;

        let rekeyed: HashMap<&Secret, Vec<u8>> = secrets
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

                let mut plaintext_buffer = Vec::new();
                let input_reader = InputReader::new(Some(path_str))?;
                rekey_stream(
                    input_reader,
                    &mut plaintext_buffer,
                    &identities,
                    &s.recipients,
                )?;

                trace!("Successfully rekeyed secret `{}`", &s.name);
                Ok((s, plaintext_buffer))
            })
            .collect::<eyre::Result<_>>()?;

        trace!("Rekeyed all secrets, writing");

        #[expect(clippy::iter_over_hash_type, reason = "We don't care about order here")]
        for (secret, rekey) in rekeyed {
            let resulting_path = self
                .storage
                .join(&secret.name)
                .with_extension(SECRETS_EXTENSION);
            let output = resulting_path.to_str().map(String::from);
            let mut writer = OutputWriter::new(output, true, OutputFormat::Text, 0o644, false)?;
            writer.write_all(&rekey)?; // TODO: atomic
        }

        trace!("Done");

        Ok(())
    }
}
