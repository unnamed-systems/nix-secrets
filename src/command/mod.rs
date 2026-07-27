use clap::{Parser, Subcommand};
use eyre::Ok;

use activate::ActivateCommand;
use edit::EditCommand;
use keygen::KeygenCommand;
use regenerate::RegenerateCommand;
use rekey::RekeyCommand;

mod activate;
mod edit;
mod keygen;
mod regenerate;
mod rekey;

#[derive(Parser, PartialEq, Debug)]
#[command(author, version, about, long_about = None)]
/// A postmodern secrets manager for NixOS.
pub struct Args {
    #[command(subcommand)]
    command: Command,

    #[clap(short, long, default_value = ".")]
    /// path to the flake containing secrets
    flake: String,
}

#[derive(Subcommand, PartialEq, Debug)]
enum Command {
    Activate(ActivateCommand),
    Edit(EditCommand),
    Rekey(RekeyCommand),
    Keygen(KeygenCommand),
    Regenerate(RegenerateCommand),
}

impl CommandTrait for Command {
    fn execute(&self, root: &Args) -> eyre::Result<()> {
        match &self {
            Self::Activate(cmd) => cmd.execute(root)?,
            Self::Keygen(cmd) => cmd.execute(root)?,
            Self::Edit(cmd) => cmd.execute(root)?,
            Self::Rekey(cmd) => cmd.execute(root)?,
            Self::Regenerate(cmd) => cmd.execute(root)?,
        }
        Ok(())
    }
}

trait CommandTrait {
    fn execute(&self, root: &Args) -> eyre::Result<()>;
}

impl Args {
    pub(crate) fn run() -> eyre::Result<()> {
        let args = Self::parse();
        trace!("Parsing command");
        args.command.execute(&args)?;
        Ok(())
    }
}
