use std::{
    fs::File,
    io::{self, BufReader},
    path::{Path, PathBuf},
};

use age::{
    Decryptor, IdentityFile,
    armor::{ArmoredReader, ArmoredWriter, Format},
    cli_common::file_io::{InputReader, OutputFormat, OutputWriter},
};
use eyre::{Context, eyre};

fn get_age_decryptor<P: AsRef<Path>>(
    path: P,
) -> eyre::Result<Decryptor<ArmoredReader<BufReader<File>>>> {
    let file = File::open(path)?;

    let reader = ArmoredReader::new(file);
    let decryptor = Decryptor::new(reader)?;

    Ok(decryptor)
}

pub(crate) fn get_identities(
    paths: Vec<PathBuf>,
) -> eyre::Result<Vec<Box<dyn age::Identity + Send + Sync>>> {
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

pub(crate) fn decrypt<P: AsRef<Path>>(
    input_file: P,
    output_file: P,
    identities: &Vec<Box<dyn age::Identity + Send + Sync>>,
) -> eyre::Result<()> {
    let output_file_mode: u32 = 0o600;
    get_age_decryptor(input_file)?
        .decrypt(identities.iter().map(|i| i.as_ref() as &dyn age::Identity))
        .map_err(Into::into)
        .and_then(|mut plaintext_reader| {
            let output = output_file
                .as_ref()
                .to_str()
                .map(std::string::ToString::to_string);
            let mut ciphertext_writer =
                OutputWriter::new(output, true, OutputFormat::Unknown, output_file_mode, false)?;
            io::copy(&mut plaintext_reader, &mut ciphertext_writer)?;
            Ok(())
        })
}

fn parse_recipient(
    s: &str,
    recipients: &mut Vec<Box<dyn age::Recipient + Send>>,
    plugin_recipients: &mut Vec<age::plugin::Recipient>,
) -> eyre::Result<()> {
    if let Ok(pk) = s.parse::<age::x25519::Recipient>() {
        recipients.push(Box::new(pk));
        Ok(())
    } else if let Some(pk) = { s.parse::<age::ssh::Recipient>().ok().map(Box::new) } {
        recipients.push(pk);
        Ok(())
    } else if let Ok(pk) = s.parse::<age::plugin::Recipient>() {
        plugin_recipients.push(pk);
        Ok(())
    } else {
        Err(eyre!("Invalid recipient: {}", s))
    }
}

pub(crate) fn encrypt<P: AsRef<Path>>(
    input_file: P,
    output_file: P,
    public_keys: &[String],
) -> eyre::Result<()> {
    let output_file_mode: u32 = 0o644;
    let mut input = InputReader::new(input_file.as_ref().to_str().map(str::to_string))?;

    let output = OutputWriter::new(
        output_file.as_ref().to_str().map(str::to_string),
        true,
        OutputFormat::Text,
        output_file_mode,
        false,
    )?;

    let mut recipients: Vec<Box<dyn age::Recipient + Send>> = vec![];
    let mut plugin_recipients: Vec<age::plugin::Recipient> = vec![];

    for pubkey in public_keys {
        parse_recipient(pubkey, &mut recipients, &mut plugin_recipients)?;
    }

    merge_plugin_recipients_and_recipients(&mut recipients, &plugin_recipients)?;

    let recipient_refs = recipients.iter().map(|r| r.as_ref() as &dyn age::Recipient);
    let encryptor = age::Encryptor::with_recipients(recipient_refs)?;

    let mut output = encryptor
        .wrap_output(
            ArmoredWriter::wrap_output(output, Format::AsciiArmor)
                .wrap_err("Failed to wrap output with age::ArmoredWriter")?,
        )
        .map_err(|err| eyre!(err))?;

    io::copy(&mut input, &mut output)?;
    output.finish().and_then(ArmoredWriter::finish)?;

    Ok(())
}

fn merge_plugin_recipients_and_recipients(
    recipients: &mut Vec<Box<dyn age::Recipient + Send>>,
    plugin_recipients: &[age::plugin::Recipient],
) -> eyre::Result<()> {
    let mut plugin_names = plugin_recipients
        .iter()
        .map(age::plugin::Recipient::plugin)
        .collect::<Vec<_>>();
    plugin_names.sort_unstable();
    plugin_names.dedup();

    for plugin_name in plugin_names {
        recipients.push(Box::new(age::plugin::RecipientPluginV1::new(
            plugin_name,
            plugin_recipients,
            &Vec::<age::plugin::Identity>::new(),
            age::cli_common::UiCallbacks,
        )?));
    }
    Ok(())
}
