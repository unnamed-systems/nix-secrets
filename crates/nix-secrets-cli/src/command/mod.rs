use clap::{Parser, Subcommand};
use clap_complete::ArgValueCompleter;
use eyre::Ok;

use edit::EditCommand;
use keygen::KeygenCommand;
use regenerate::RegenerateCommand;
use rekey::RekeyCommand;

use crate::{FLAKE_CONFIGURATION_PREFIX, command::decrypt::DecryptCommand, utils};

mod decrypt;
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
    #[arg(add = ArgValueCompleter::new(utils::complete_flake), global = true)]
    /// path to the flake containing secrets
    pub(crate) flake: String,

    /// module system for this host
    #[clap(short, long, default_value = FLAKE_CONFIGURATION_PREFIX)]
    #[arg(global = true)]
    pub(crate) module_system: String,

    /// whether to use the home-manager profile inside the nixos configuration
    #[clap(long, conflicts_with = "hj")]
    #[arg(global = true)]
    pub(crate) hm: Option<String>,

    /// whether to use the hjem profile inside the nixos configuration
    #[clap(long)]
    #[arg(global = true)]
    pub(crate) hj: Option<String>,
}

#[derive(Subcommand, PartialEq, Debug)]
enum Command {
    Edit(EditCommand),
    Decrypt(DecryptCommand),
    Rekey(RekeyCommand),
    Keygen(KeygenCommand),
    Regenerate(RegenerateCommand),
}

impl CommandTrait for Command {
    fn execute(&self, root: &Args) -> eyre::Result<()> {
        match self {
            Self::Keygen(cmd) => cmd.execute(root)?,
            _ => match self {
                Self::Edit(cmd) => cmd.execute(root)?,
                Self::Decrypt(cmd) => cmd.execute(root)?,
                Self::Rekey(cmd) => cmd.execute(root)?,
                Self::Regenerate(cmd) => cmd.execute(root)?,
                _ => {}
            },
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
