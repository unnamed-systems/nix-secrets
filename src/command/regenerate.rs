use std::{
    collections::HashMap,
    io::Write as _,
    path::{Path, PathBuf},
    process::Command,
};

use age::cli_common::file_io::{OutputFormat, OutputWriter};
use clap::{Parser, ValueHint};
use clap_complete::ArgValueCompleter;
use eyre::{Context as _, Ok, OptionExt as _, bail, eyre};

use crate::{
    Result, SECRETS_EXTENSION,
    command::{Args, CommandTrait},
    manifest::{ManifestOptions, Secret},
    utils::{self, PathBufExt as _, check_secret_name, encrypt_stream},
};

#[derive(Parser, PartialEq, Eq, Debug)]
/// Regenerate all or selected encrypted secrets with generators
pub struct RegenerateCommand {
    /// path to the secrets directory
    #[arg(short, long)]
    #[arg(value_hint = ValueHint::DirPath, env = "NIX_SECRETS_STORAGE_PATH")]
    storage: Option<PathBuf>,

    /// secrets to regenerate
    #[arg(add = ArgValueCompleter::new(|v: &std::ffi::OsStr| utils::complete_secrets(v, true)))]
    secrets: Vec<String>,

    /// regenerate all secrets
    #[arg(short, long)]
    all: bool,

    /// regenerate missing secrets
    #[arg(short, long)]
    missing: bool,
}

impl CommandTrait for RegenerateCommand {
    fn execute(&self, root: &Args) -> Result<()> {
        if self.secrets.is_empty() != self.all | self.missing {
            bail!("Provide either secret names to regenerate, `--missing` or `--all`");
        }

        let (flake, module_system, hostname) =
            utils::parse_flake(&root.flake, true).ok_or_eyre("Failed to parse flake")?;

        trace!("Parsed flake: {}, hostname: {}", flake, hostname);

        let manifest_options = if self.all {
            ManifestOptions::WithAllGenerators
        } else {
            ManifestOptions::WithGenerators(self.secrets.clone())
        };

        let manifest = utils::eval_manifest(
            &flake,
            module_system.as_deref(),
            &hostname,
            manifest_options,
        )?;
        let storage_path = self
            .storage
            .as_ref()
            .or(manifest.storage_path.as_ref())
            .ok_or_eyre("No storage path provided")?;

        if !storage_path.is_dir() {
            bail!("Invalid directory path");
        }

        let secrets: Vec<Secret> = manifest
            .secrets
            .into_iter()
            .filter(|s| {
                (self.secrets.contains(&s.name)
                    || self.all
                    || (self.missing
                        && !Path::new(
                            &storage_path
                                .join(&s.name)
                                .append_extension(SECRETS_EXTENSION)
                                .into_os_string(),
                        )
                        .exists()))
                    && s.generator.is_some()
            })
            .collect();

        if secrets.is_empty() {
            bail!("No secrets to regenerate.");
        }

        let regenerated: HashMap<&Secret, Vec<u8>> = secrets
            .iter()
            .map(|s| {
                if !check_secret_name(&s.name) {
                    bail!("Secret name `{}` contains illegal values", s.name)
                }
                let path_str = storage_path
                    .join(&s.name)
                    .append_extension(SECRETS_EXTENSION)
                    .into_os_string()
                    .into_string()
                    .map_err(|e| eyre::eyre!("Invalid unicode path provided: {e:?}"))?;

                trace!("Reading secret `{}` ({})", &s.name, path_str);

                let Some(generator) = &s.generator else {
                    bail!("Secret generator is undefined");
                };

                trace!("Evaluated generator. Binary: {}", generator.display());

                let output = Command::new(&generator)
                    .current_dir("/")
                    .output()
                    .wrap_err_with(|| {
                        format!("Failed to spawn the generator: {}", generator.display())
                    })?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);

                    return Err(eyre!("Generator execution failed: {}", generator.display()))
                        .wrap_err_with(|| stderr.trim().to_owned());
                }

                trace!("Got generator output");

                let mut plaintext_buffer = Vec::new();
                encrypt_stream(&*output.stdout, &mut plaintext_buffer, &s.recipients)?;

                info!("Successfully regenerated secret `{}`", &s.name);
                Ok((s, plaintext_buffer))
            })
            .collect::<eyre::Result<_>>()?;

        trace!("Rekeyed all secrets, writing");

        #[expect(clippy::iter_over_hash_type, reason = "We don't care about order here")]
        for (secret, rekey) in regenerated {
            let resulting_path = storage_path
                .join(&secret.name)
                .append_extension(SECRETS_EXTENSION);
            let output = resulting_path.to_str().map(String::from);
            let mut writer = OutputWriter::new(output, true, OutputFormat::Text, 0o644, false)?;
            writer.write_all(&rekey).wrap_err(format!(
                "Failed to write the secret `{}` ({})",
                secret.name,
                secret.path.display()
            ))?; // TODO: atomic
        }

        info!("Done");

        Ok(())
    }
}
