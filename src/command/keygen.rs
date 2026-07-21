use crate::command::{Args, CommandTrait};
use age::{cli_common::file_io, secrecy::ExposeSecret};
use argh::{ArgsInfo, FromArgs};
use eyre::Ok;
use std::io::Write;

#[derive(FromArgs, ArgsInfo, PartialEq, Debug)]
/// Generate an age keypair
#[argh(subcommand, name = "keygen")]
pub(crate) struct KeygenCommand {
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
        let output = file_io::OutputWriter::new(
            self.output.clone(),
            false,
            file_io::OutputFormat::Text,
            0o600,
            false,
        )?;

        if self.convert {
            convert(self.input.clone(), output)?;
        } else {
            generate(output)?;
        }
        Ok(())
    }
}

fn generate(mut output: file_io::OutputWriter) -> eyre::Result<()> {
    trace!("Generating a new keypair");
    let sk = age::x25519::Identity::generate();
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

    Ok(())
}

fn convert(filename: Option<String>, output: file_io::OutputWriter) -> eyre::Result<()> {
    trace!("Converting an existing identity");
    let file = age::IdentityFile::from_input_reader(file_io::InputReader::new(filename)?)?;

    file.write_recipients_file(output)?;
    Ok(())
}
