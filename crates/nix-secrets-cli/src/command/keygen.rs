use crate::command::{Args, CommandTrait};
use age::{cli_common::file_io, secrecy::ExposeSecret as _, x25519};
use clap::{Parser, ValueHint};
use eyre::{Context, Ok};
use std::io::Write as _;

#[derive(Parser, PartialEq, Eq, Debug)]
/// Generate an age keypair
pub struct KeygenCommand {
    /// Write the key to this file instead of stdout
    #[arg(short = 'o', long, value_name = "PATH", value_hint = ValueHint::FilePath)]
    output: Option<String>,

    /// Convert the given identity file to a recipients file instead of generating a new keypair
    #[arg(short = 'y', long = "convert", value_name = "PATH", value_hint = ValueHint::FilePath)]
    #[expect(
        clippy::option_option,
        reason = "Allows for providing just the argument"
    )]
    input: Option<Option<String>>,
}

impl CommandTrait for KeygenCommand {
    // TODO: zerocopy
    fn execute(&self, _root: &Args) -> eyre::Result<()> {
        let mut output = file_io::OutputWriter::new(
            self.output.clone(),
            false,
            file_io::OutputFormat::Text,
            // Only the user should be able to read and write this file
            0o600,
            false,
        )?;

        if let Some(input) = self.input.clone() {
            trace!("Converting an existing identity");
            let reader =
                file_io::InputReader::new(input).wrap_err("Failed to open the identity file")?;
            let file = age::IdentityFile::from_input_reader(reader)
                .wrap_err("Failed to parse the identity file")?;

            file.write_recipients_file(output)
                .wrap_err("Failed to write the recipients file")?;
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
                eprintln!("Public key: {pk}");
            }
        }
        Ok(())
    }
}
