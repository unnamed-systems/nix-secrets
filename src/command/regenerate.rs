use std::{collections::HashMap, io::Write as _, path::PathBuf, process::Command};

use age::cli_common::file_io::{OutputFormat, OutputWriter};
use clap::{Parser, ValueHint};
use eyre::{Ok, OptionExt as _, bail};

use crate::{
    Result, SECRETS_EXTENSION,
    command::{Args, CommandTrait},
    manifest::Secret,
    utils::{self},
};

#[derive(Parser, PartialEq, Eq, Debug)]
/// Regenerate all or selected encrypted secrets with generators
pub struct RegenerateCommand {
    /// path to the secrets directory
    #[arg(short, long)]
    #[arg(value_hint = ValueHint::DirPath, env = "NIX_SECRETS_STORAGE_PATH")]
    storage: PathBuf,

    /// secrets to regenerate
    secrets: Vec<String>,

    /// regenerate all secrets
    #[arg(short, long)]
    all: bool,
}

impl CommandTrait for RegenerateCommand {
    fn execute(&self, root: &Args) -> Result<()> {
        if !self.storage.is_dir() {
            bail!("Invalid directory path");
        }

        if self.secrets.is_empty() != self.all {
            bail!("Provide either secret names to rekey or `--all`");
        }

        let (flake, hostname) =
            utils::parse_flake(&root.flake).ok_or_eyre("Failed to parse flake")?;

        trace!("Parsed flake: {}, hostname: {}", flake, hostname);

        let manifest = utils::eval_manifest(&flake, &hostname)?;
        let secrets: Vec<Secret> = manifest
            .secrets
            .into_iter()
            .filter(|s| (self.secrets.contains(&s.name) || self.all) && s.generator.is_some())
            .collect();

        let regenerated: HashMap<&Secret, Vec<u8>> = secrets
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

                let Some(generator) = &s.generator else {
                    bail!("Secret generator is undefined");
                };
                let output = Command::new(generator).output()?;
                trace!("Got generator output: {output:?}");
                Ok((s, output.stdout))
            })
            .collect::<eyre::Result<_>>()?;

        trace!("Rekeyed all secrets, writing");

        #[expect(clippy::iter_over_hash_type, reason = "We don't care about order here")]
        for (secret, rekey) in regenerated {
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
