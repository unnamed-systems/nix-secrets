use crate::command::{Args, CommandTrait};
use age::{cli_common::file_io, secrecy::ExposeSecret as _, x25519};
use clap::{Parser, ValueHint};
use eyre::{Context, Ok};
use std::io::Write as _;

#[derive(Parser, PartialEq, Eq, Debug)]
/// Generate an age keypair
pub struct KeygenCommand {
    /// Path to the input file
    #[arg(value_hint = ValueHint::FilePath)]
    input: Option<String>,

    /// Path to the output directory
    #[arg(short, long)]
    #[arg(value_hint = ValueHint::DirPath)]
    output: Option<String>,

    /// Convert an identity file to a recipient
    #[arg(short = 'y')]
    convert: bool,
}

impl CommandTrait for KeygenCommand {
    // TODO: zerocopy
    fn execute(&self, _root: &Args) -> eyre::Result<()> {
        let mut output = file_io::OutputWriter::new(
            self.output.clone(),
            false,
            file_io::OutputFormat::Text,
            // We should be able to write and read
            0o600,
            false,
        )?;

        if self.convert {
            trace!("Converting an existing identity");
            let file = age::IdentityFile::from_input_reader(file_io::InputReader::new(
                self.output.clone(),
            )?)?;

            file.write_recipients_file(output)
                .wrap_err("Failed to write the recipients output")?;
        } else {
            trace!("Generating a new keypair");
            let sk = x25519::Identity::generate();
            let pk = sk.to_public();

            writeln!(
                output,
                "# created: {}",
                chrono::Local::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
            )?;
            writeln!(output, "# public key: {pk}")?;
            writeln!(output, "{}", sk.to_string().expose_secret())?;

            if !output.is_terminal() {
                println!("Public key: {pk}");
            }
        }
        Ok(())
    }
}
