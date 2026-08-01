use crate::Result;
use std::{
    fs::File,
    io::{self, BufReader, Read, Write},
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

fn get_age_decryptor<R>(read: R) -> Result<Decryptor<ArmoredReader<BufReader<R>>>>
where
    R: Read,
{
    let reader = ArmoredReader::new(read);
    let decryptor = Decryptor::new(reader)?;

    Ok(decryptor)
}

pub fn get_identities(paths: &[PathBuf]) -> Result<Vec<Box<dyn age::Identity + Send + Sync>>> {
    paths
        .iter()
        .filter(|path| path.is_file())
        .filter_map(|path| File::open(path).map(BufReader::new).ok())
        .try_fold(Vec::new(), |mut acc, mut reader| {
            // Parses both age native identities and plugins
            if let Ok(file) = IdentityFile::from_buffer(&mut reader) {
                acc.extend(file.into_identities()?);
                trace!("Parsed a native age keypair")
            } else {
                use std::io::Seek;
                reader.rewind()?;

                match ssh::Identity::from_buffer(reader, None) {
                    Ok(identity) => acc.push(Box::new(identity)),
                    _ => trace!("Failed to parse ssh identity"),
                }
            }
            Ok(acc)
        })
}

pub fn decrypt_stream<R, W>(
    input: R,
    mut output: W,
    identities: &[Box<dyn age::Identity + Send + Sync>],
) -> Result<()>
where
    R: Read,
    W: Write,
{
    #[expect(
        clippy::as_conversions,
        reason = "dyn Identity + Send + Sync -> dyn Identity"
    )]
    let mut plaintext_reader = get_age_decryptor(input)?
        .decrypt(identities.iter().map(|i| i.as_ref() as &dyn age::Identity))?;
    io::copy(&mut plaintext_reader, &mut output)?;
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

pub fn encrypt_stream<R, W>(mut input: R, output: W, public_keys: &[String]) -> Result<()>
where
    R: Read,
    W: Write,
{
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

pub fn rekey_stream<R, W>(
    input: R,
    output: W,
    identities: &[Box<dyn age::Identity + Send + Sync>],
    recipients: &[String],
) -> Result<()>
where
    R: Read,
    W: Write,
{
    let mut plaintext_buffer = Vec::new();

    decrypt_stream(input, &mut plaintext_buffer, identities)
        .wrap_err("Failed to decrypt stream")?;

    encrypt_stream(&*plaintext_buffer, output, recipients)
        .wrap_err("Failed to encrypt stream with new recipients")?;

    Ok(())
}

pub fn decrypt<P>(
    input_file: P,
    output_file: P,
    identities: &[Box<dyn age::Identity + Send + Sync>],
) -> Result<()>
where
    P: AsRef<Path>,
{
    let input = InputReader::new(input_file.as_ref().to_str().map(str::to_owned))?;

    let output = output_file.as_ref().to_str().map(str::to_owned);
    let ciphertext_writer = OutputWriter::new(output, true, OutputFormat::Unknown, 0o600, false)?;

    decrypt_stream(input, ciphertext_writer, identities)
}

pub fn encrypt<P>(input_file: P, output_file: P, recipients: &[String]) -> Result<()>
where
    P: AsRef<Path>,
{
    let input = InputReader::new(input_file.as_ref().to_str().map(str::to_owned))?;

    let output = output_file.as_ref().to_str().map(str::to_owned);
    let ciphertext_writer = OutputWriter::new(output, true, OutputFormat::Text, 0o644, false)?;

    encrypt_stream(input, ciphertext_writer, recipients)
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
