use crate::Result;
use std::{
    fs,
    io::{self, BufReader, Cursor, ErrorKind, Read, Write},
    path::{Path, PathBuf},
};

use age::{
    Decryptor, IdentityFile,
    armor::{ArmoredReader, ArmoredWriter, Format},
    cli_common::{
        UiCallbacks,
        file_io::{InputReader, OutputFormat, OutputWriter},
    },
    plugin,
    secrecy::zeroize::Zeroizing,
    ssh, x25519,
};
use eyre::{Context as _, bail, eyre};

fn zeroizing_copy<R: Read, W: Write>(reader: &mut R, writer: &mut W) -> io::Result<()> {
    let mut buf = Zeroizing::new([0u8; 8192]);

    loop {
        let len = match reader.read(buf.as_mut_slice()) {
            Ok(0) => return Ok(()),
            Ok(len) => len,
            Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        };
        writer.write_all(&buf[..len])?;
    }
}

fn get_writer<W: Write>(
    output: W,
    public_keys: &[String],
) -> Result<age::stream::StreamWriter<ArmoredWriter<W>>> {
    let mut recipients: Vec<Box<dyn age::Recipient + Send>> = vec![];
    let mut plugin_recipients: Vec<plugin::Recipient> = vec![];

    for pubkey in public_keys {
        parse_recipient(pubkey, &mut recipients, &mut plugin_recipients)?;
    }

    merge_plugin_recipients_and_recipients(&mut recipients, &plugin_recipients)?;
    trace!("Merged plugin recipients");

    let recipient_refs = recipients.iter().map(|r| r.as_ref() as &dyn age::Recipient);
    let encryptor = age::Encryptor::with_recipients(recipient_refs).wrap_err(
        "Failed to initialize encryptor (Are you mixing post-quantum and classic recipient types?)",
    )?;

    let armored = ArmoredWriter::wrap_output(output, Format::AsciiArmor)
        .wrap_err("Failed to wrap output with age::ArmoredWriter")?;

    encryptor.wrap_output(armored).map_err(|err| eyre!(err))
}

fn get_age_decryptor<R>(read: R) -> Result<Decryptor<ArmoredReader<BufReader<R>>>>
where
    R: Read,
{
    let reader = ArmoredReader::new(read);
    let decryptor = Decryptor::new(reader).wrap_err("Failed to create a secret decryptor")?;

    Ok(decryptor)
}

pub fn get_identities<R>(mut read: R) -> Result<Vec<Box<dyn age::Identity + Send + Sync>>>
where
    R: io::BufRead,
{
    let mut buf = Zeroizing::new(Vec::new());
    read.read_to_end(&mut buf)?;

    let mut identities = Vec::new();
    if let Ok(file) =
        IdentityFile::from_buffer(buf.as_slice()).map(|i| i.with_callbacks(UiCallbacks))
    {
        trace!("Parsed a native age identity");
        identities.extend(
            file.into_identities()
                .wrap_err("Failed to read identities from file")?,
        );
    } else {
        trace!("Failed to parse a native identity, trying ssh");

        if let Ok(identity) = ssh::Identity::from_buffer(buf.as_slice(), None) {
            trace!("Parsed an ssh identity");
            identities.push(Box::new(identity.with_callbacks(UiCallbacks)))
        } else {
            bail!("Failed to parse the identity");
        }
    };

    Ok(identities)
}

pub fn get_identities_paths(
    paths: &[PathBuf],
) -> Result<Vec<Box<dyn age::Identity + Send + Sync>>> {
    paths
        .iter()
        .filter(|path| path.is_file())
        .filter_map(|path| fs::read(path).map(Zeroizing::new).ok())
        .try_fold(Vec::new(), |mut acc, contents| {
            acc.extend(get_identities(Cursor::new(contents.as_slice()))?);
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
    let mut plaintext_reader = get_age_decryptor(input)?
        .decrypt(identities.iter().map(|i| i.as_ref() as &dyn age::Identity))
        .wrap_err("Failed to decrypt secret")?;
    zeroizing_copy(&mut plaintext_reader, &mut output)?;
    Ok(())
}

fn parse_recipient(
    s: &str,
    recipients: &mut Vec<Box<dyn age::Recipient + Send>>,
    plugin_recipients: &mut Vec<plugin::Recipient>,
) -> Result<()> {
    if let Ok(pk) = s.parse::<x25519::Recipient>() {
        trace!("Parsed a native age recipient");
        recipients.push(Box::new(pk));
        Ok(())
    } else if let Some(pk) = { s.parse::<ssh::Recipient>().ok().map(Box::new) } {
        trace!("Parsed an ssh recipient");
        recipients.push(pk);
        Ok(())
    } else if let Ok(pk) = s.parse::<plugin::Recipient>() {
        trace!("Parsed an age plugin recipient ({})", pk.plugin());
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
    let mut output_writer = get_writer(output, public_keys)?;
    zeroizing_copy(&mut input, &mut output_writer)?;
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
    let mut plaintext_reader = get_age_decryptor(input)?
        .decrypt(identities.iter().map(|i| i.as_ref() as &dyn age::Identity))
        .wrap_err("Failed to decrypt secret")?;

    let mut output_writer = get_writer(output, recipients)?;

    zeroizing_copy(&mut plaintext_reader, &mut output_writer)
        .wrap_err("Failed to re-encrypt stream")?;

    output_writer.finish().and_then(ArmoredWriter::finish)?;
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
    let plaintext_writer = OutputWriter::new(output, true, OutputFormat::Unknown, 0o600, false)?;

    decrypt_stream(input, plaintext_writer, identities)
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
        recipients.push(Box::new(
            plugin::RecipientPluginV1::new(
                plugin_name,
                plugin_recipients,
                &Vec::<plugin::Identity>::new(),
                UiCallbacks,
            )
            .wrap_err("Failed to parse a plugin recipient")?,
        ));
    }
    Ok(())
}
