use age::cli_common::file_io::{OutputFormat, OutputWriter};
use nix_secrets_core::{
    Result, SECRETS_EXTENSION,
    age::encrypt_stream,
    file::{PathBufExt as _, check_secret_name},
    model::{ManifestOptions, Secret},
};

use std::{
    collections::{HashMap, HashSet},
    io::Write as _,
    path::PathBuf,
    process::Command,
};

use clap::{Parser, ValueHint};
use clap_complete::ArgValueCompleter;
use eyre::{Context as _, Ok, OptionExt as _, bail, ensure, eyre};

use crate::{
    command::{Args, CommandTrait},
    utils::{self},
};

#[derive(Parser, PartialEq, Eq, Debug)]
/// Regenerate all or selected encrypted secrets with generators
pub struct RegenerateCommand {
    /// Path to the nix-secrets storage (overrides the manifest `storagePath`)
    #[arg(short, long)]
    #[arg(value_hint = ValueHint::DirPath, env = "NIX_SECRETS_STORAGE_PATH")]
    storage: Option<PathBuf>,

    /// Secrets to regenerate
    #[arg(add = ArgValueCompleter::new(|v: &std::ffi::OsStr| utils::complete_secrets(v, true)), conflicts_with_all = ["missing", "all"])]
    secrets: Vec<String>,

    /// Regenerate all secrets with generators
    #[arg(short, long, conflicts_with = "missing")]
    all: bool,

    /// Regenerate only secrets that don't exist yet
    #[arg(long)]
    missing: bool,

    /// Show which secrets would be regenerated without writing anything
    #[arg(long)]
    dry: bool,
}

impl CommandTrait for RegenerateCommand {
    fn execute(&self, root: &Args) -> Result<()> {
        let manifest_options = if self.all {
            ManifestOptions::WithAllGenerators
        } else {
            ManifestOptions::WithGenerators(self.secrets.clone())
        };

        let manifest = utils::eval_manifest(root, &manifest_options)?;
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

        let mut requested: HashSet<&String> = HashSet::from_iter(self.secrets.iter());

        let secrets: Vec<Secret> = manifest
            .secrets
            .into_iter()
            .filter(|s| {
                requested.remove(&s.name)
                    || s.generator.is_some()
                        && (self.all
                            || (self.missing
                                && !storage_path
                                    .join(&s.name)
                                    .with_added_extension(SECRETS_EXTENSION)
                                    .try_exists()
                                    .unwrap_or(false)))
            })
            .map(|s| {
                ensure!(
                    s.generator.is_some(),
                    "Secret `{}` has no generator",
                    s.name
                );
                Ok(s)
            })
            .collect::<eyre::Result<_>>()?;

        if let Some(missing) = requested.into_iter().next() {
            bail!("Secret `{missing}` is not present in the Nix configuration");
        }

        if secrets.is_empty() {
            bail!("No secrets to regenerate");
        }

        if self.dry {
            info!(
                "The following secrets would have been regenerated:\n{}",
                secrets
                    .into_iter()
                    .map(|s| format!(
                        "\t- `{}` ({})",
                        s.name,
                        s.generator
                            .and_then(|p| p.into_os_string().into_string().ok())
                            .unwrap_or_else(|| String::from("Secret generator is undefined"))
                    ))
                    .collect::<Vec<String>>()
                    .join("\n")
            );
            return Ok(());
        }

        let regenerated: HashMap<&Secret, Vec<u8>> = secrets
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

                let Some(generator) = &s.generator else {
                    bail!("Secret generator is undefined");
                };

                trace!("Evaluated generator. Binary: {}", generator.display());

                let output = Command::new(generator)
                    .current_dir("/")
                    .output()
                    .wrap_err_with(|| {
                        format!("Failed to spawn generator `{}`", generator.display())
                    })?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);

                    return Err(eyre!(
                        "Failed to execute generator `{}`",
                        generator.display()
                    ))
                    .wrap_err_with(|| stderr.trim().to_owned());
                }

                trace!("Got generator output");

                let mut plaintext_buffer = Vec::new();
                encrypt_stream(&*output.stdout, &mut plaintext_buffer, &s.recipients)
                    .wrap_err_with(|| format!("Failed to encrypt the secret `{}`", s.name))?;

                info!("Successfully regenerated secret `{}`", &s.name);
                Ok((s, plaintext_buffer))
            })
            .collect::<eyre::Result<_>>()?;

        trace!("Regenerated all secrets, writing");

        #[expect(clippy::iter_over_hash_type, reason = "We don't care about order here")]
        for (secret, plaintext) in regenerated {
            let resulting_path = storage_path
                .join(&secret.name)
                .append_extension(SECRETS_EXTENSION);
            let output = resulting_path.to_str().map(String::from);
            let mut writer = OutputWriter::new(output, true, OutputFormat::Text, 0o644, false)
                .wrap_err_with(|| {
                    format!("Failed to open the secret `{}` for writing", secret.name)
                })?;
            writer
                .write_all(&plaintext)
                .wrap_err_with(|| format!("Failed to write the secret `{}`", secret.name))?; // TODO: atomic
        }

        info!("Done");

        Ok(())
    }
}
