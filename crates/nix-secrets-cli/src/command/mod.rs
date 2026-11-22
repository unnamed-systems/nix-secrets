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
#[command(author, version, long_about = None, bin_name = "nix-secrets")]
/// A postmodern secrets manager for NixOS
pub struct Args {
    #[command(subcommand)]
    command: Command,

    #[clap(short, long, default_value = ".")]
    #[arg(add = ArgValueCompleter::new(utils::complete_flake), global = true)]
    /// Path to the flake containing the secrets
    pub(crate) flake: String,

    /// Flake output containing the host (e.g. nixosConfigurations)
    #[clap(short, long, default_value = FLAKE_CONFIGURATION_PREFIX)]
    #[arg(global = true)]
    pub(crate) module_system: String,

    /// Use the home-manager profile of the given user inside the host configuration
    #[clap(long, conflicts_with = "hjem", value_name = "user")]
    #[arg(global = true, visible_alias = "hm")]
    pub(crate) home_manager: Option<String>,

    /// Use the hjem profile of the given user inside the host configuration
    #[clap(long, value_name = "user")]
    #[arg(global = true, visible_alias = "hj")]
    pub(crate) hjem: Option<String>,

    /// Extra paths to identity files used to decrypt secrets
    #[clap(long, short, value_name = "path")]
    #[arg(global = true, value_hint = clap::ValueHint::FilePath)]
    pub(crate) identity: Vec<String>,
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
            Self::Edit(cmd) => cmd.execute(root)?,
            Self::Decrypt(cmd) => cmd.execute(root)?,
            Self::Rekey(cmd) => cmd.execute(root)?,
            Self::Regenerate(cmd) => cmd.execute(root)?,
            Self::Keygen(cmd) => cmd.execute(root)?,
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
