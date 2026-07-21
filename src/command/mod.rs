use argh::{ArgsInfo, FromArgs};
use eyre::Ok;

use activate::ActivateCommand;
use completion::CompletionCommand;
use edit::EditCommand;
use keygen::KeygenCommand;

mod activate;
mod completion;
mod edit;
mod keygen;

#[derive(FromArgs, ArgsInfo, PartialEq, Debug)]
#[argh(help_triggers("-h", "--help", "help"))]
/// A postmodern secret manager for NixOS.
pub(crate) struct Args {
    #[argh(subcommand)]
    command: Command,

    #[argh(option, short = 'f', default = "String::from(\".\")")]
    /// path to the flake containing secrets
    flake: String,
}

#[derive(FromArgs, ArgsInfo, PartialEq, Debug)]
#[argh(subcommand)]
enum Command {
    Activate(ActivateCommand),
    Completion(CompletionCommand),
    Edit(EditCommand),
    Generate(KeygenCommand),
}

impl CommandTrait for Command {
    fn execute(&self, root: &Args) -> eyre::Result<()> {
        match &self {
            Command::Activate(cmd) => cmd.execute(root)?,
            Command::Generate(cmd) => cmd.execute(root)?,
            Command::Completion(cmd) => cmd.execute(root)?,
            Command::Edit(cmd) => cmd.execute(root)?,
        }
        Ok(())
    }
}

trait CommandTrait {
    fn execute(&self, root: &Args) -> eyre::Result<()>;
}

impl Args {
    pub(crate) fn parse(&self) -> eyre::Result<()> {
        trace!("Parsing command");
        self.command.execute(self)?;
        Ok(())
    }
}
