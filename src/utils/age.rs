use crate::Result;
use std::{
    fs::File,
    io::{self, BufReader},
    path::{Path, PathBuf},
};

use age::{
    Decryptor, IdentityFile,
    armor::{ArmoredReader, ArmoredWriter, Format},
    cli_common::{
        UiCallbacks,
        file_io::{InputReader, OutputFormat, OutputWriter},
    },
    plugin, ssh, x25519,
};
use eyre::{Context as _, eyre};

fn get_age_decryptor<P>(path: P) -> Result<Decryptor<ArmoredReader<BufReader<File>>>>
where
    P: AsRef<Path>,
{
    let file = File::open(path)?;

    let reader = ArmoredReader::new(file);
    let decryptor = Decryptor::new(reader)?;

    Ok(decryptor)
}

pub fn get_identities(paths: Vec<PathBuf>) -> Result<Vec<Box<dyn age::Identity + Send + Sync>>> {
    paths
        .into_iter()
        .filter(|path| path.is_file())
        .filter_map(|path| File::open(path).map(BufReader::new).ok())
        .filter_map(|reader| IdentityFile::from_buffer(reader).ok())
        .try_fold(Vec::new(), |mut acc, file| {
            acc.extend(file.into_identities()?);
            Ok(acc)
        })
}

pub fn decrypt<P>(
    input_file: P,
    output_file: P,
    identities: &Vec<Box<dyn age::Identity + Send + Sync>>,
) -> Result<()>
where
    P: AsRef<Path>,
{
    let output_file_mode: u32 = 0o600;
    #[expect(
        clippy::as_conversions,
        reason = "dyn Recipient + Send -> dyn Recipient"
    )]
    let mut plaintext_reader = get_age_decryptor(input_file)?
        .decrypt(identities.iter().map(|i| i.as_ref() as &dyn age::Identity))?;
    let output = output_file.as_ref().to_str().map(str::to_owned);
    let mut ciphertext_writer =
        OutputWriter::new(output, true, OutputFormat::Unknown, output_file_mode, false)?;
    io::copy(&mut plaintext_reader, &mut ciphertext_writer)?;
    Ok(())
}

fn parse_recipient(
    s: &str,
    recipients: &mut Vec<Box<dyn age::Recipient + Send>>,
    plugin_recipients: &mut Vec<plugin::Recipient>,
) -> Result<()> {
    if let Ok(pk) = s.parse::<x25519::Recipient>() {
        recipients.push(Box::new(pk));
        Ok(())
    } else if let Some(pk) = { s.parse::<ssh::Recipient>().ok().map(Box::new) } {
        recipients.push(pk);
        Ok(())
    } else if let Ok(pk) = s.parse::<plugin::Recipient>() {
        plugin_recipients.push(pk);
        Ok(())
    } else {
        Err(eyre!("Invalid recipient: {}", s))
    }
}

pub fn encrypt<P>(input_file: P, output_file: P, public_keys: &[String]) -> Result<()>
where
    P: AsRef<Path>,
{
    let output_file_mode: u32 = 0o644;
    let mut input = InputReader::new(input_file.as_ref().to_str().map(str::to_owned))?;

    let output = OutputWriter::new(
        output_file.as_ref().to_str().map(str::to_owned),
        true,
        OutputFormat::Text,
        output_file_mode,
        false,
    )?;

    let mut recipients: Vec<Box<dyn age::Recipient + Send>> = vec![];
    let mut plugin_recipients: Vec<plugin::Recipient> = vec![];

    for pubkey in public_keys {
        parse_recipient(pubkey, &mut recipients, &mut plugin_recipients)?;
    }

    merge_plugin_recipients_and_recipients(&mut recipients, &plugin_recipients)?;

    #[expect(
        clippy::as_conversions,
        reason = "dyn Recipient + Send -> dyn Recipient"
    )]
    let recipient_refs = recipients.iter().map(|r| r.as_ref() as &dyn age::Recipient);
    let encryptor = age::Encryptor::with_recipients(recipient_refs)?;

    let mut output_writer = encryptor
        .wrap_output(
            ArmoredWriter::wrap_output(output, Format::AsciiArmor)
                .wrap_err("Failed to wrap output with age::ArmoredWriter")?,
        )
        .map_err(|err| eyre!(err))?;

    io::copy(&mut input, &mut output_writer)?;
    output_writer.finish().and_then(ArmoredWriter::finish)?;

    Ok(())
}

fn merge_plugin_recipients_and_recipients(
    recipients: &mut Vec<Box<dyn age::Recipient + Send>>,
    plugin_recipients: &[plugin::Recipient],
) -> Result<()> {
    let mut plugin_names = plugin_recipients
        .iter()
        .map(plugin::Recipient::plugin)
        .collect::<Vec<_>>();
    plugin_names.sort_unstable();
    plugin_names.dedup();

    for plugin_name in plugin_names {
        recipients.push(Box::new(plugin::RecipientPluginV1::new(
            plugin_name,
            plugin_recipients,
            &Vec::<plugin::Identity>::new(),
            UiCallbacks,
        )?));
    }
    Ok(())
}
