use crate::command::{Args, CommandTrait};
use age::{cli_common::file_io, secrecy::ExposeSecret as _, x25519};
use argh::{ArgsInfo, FromArgs};
use eyre::Ok;
use std::io::Write as _;

#[derive(FromArgs, ArgsInfo, PartialEq, Eq, Debug)]
/// Generate an age keypair
#[argh(subcommand, name = "keygen")]
pub struct KeygenCommand {
    /// path to the input file
    #[argh(positional)]
    input: Option<String>,

    /// path to the output directory
    #[argh(option, short = 'o')]
    output: Option<String>,

    /// convert an identity file to a recipient
    #[argh(switch, short = 'y')]
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

            file.write_recipients_file(output)?;
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
