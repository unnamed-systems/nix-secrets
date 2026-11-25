use std::path::PathBuf;

use clap::{CommandFactory as _, Parser, Subcommand, error::ErrorKind};
use clap_complete::ArgValueCompleter;
use eyre::Ok;

use edit::EditCommand;
use keygen::KeygenCommand;
use regenerate::RegenerateCommand;
use rekey::RekeyCommand;

use crate::{DEFAULT_MODULE_SYSTEM, command::decrypt::DecryptCommand, utils};

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

    /// Flake reference containing the configuration
    #[arg(short = 'F', long, global = true, value_name = "FLAKE-URI[#NAME]", add = ArgValueCompleter::new(utils::complete_flake), conflicts_with_all = ["file", "attr"])]
    pub(crate) flake: Option<String>,

    /// Path to the file containing the configuration
    #[arg(
        short = 'f',
        long,
        global = true,
        value_name = "PATH",
        conflicts_with = "flake"
    )]
    pub(crate) file: Option<String>,

    /// Module system name of the configuration
    #[arg(short = 'm', long, global = true, value_name = "SYSTEM")]
    pub(crate) module_system: Option<String>,

    /// Flake output containing the configuration (e.g. nixosConfigurations)
    #[arg(
        short = 'O',
        long,
        global = true,
        value_name = "OUTPUT-NAME",
        conflicts_with = "module_system"
    )]
    pub(crate) output_name: Option<String>,

    /// Attribute path in the file given by `--file` containing the configuration
    #[arg(short = 'A', long, global = true, value_name = "ATTR-PATH")]
    pub(crate) attr: Option<String>,

    /// Use the home-manager profile of the given user inside the host configuration
    #[arg(
        long,
        global = true,
        value_name = "USER",
        visible_alias = "hm",
        conflicts_with = "hjem"
    )]
    pub(crate) home_manager: Option<String>,

    /// Use the hjem profile of the given user inside the host configuration
    #[arg(long, global = true, value_name = "USER", visible_alias = "hj")]
    pub(crate) hjem: Option<String>,

    /// Extra paths to identity files used to decrypt secrets
    #[arg(short = 'i', long, global = true, value_name = "PATH", value_hint = clap::ValueHint::FilePath)]
    pub(crate) identity: Vec<String>,

    /// Path to the nix-secrets storage (overrides the manifest `storagePath`)
    #[arg(short = 's', long, global = true, value_name = "PATH", action = clap::ArgAction::Set, env = "NIX_SECRETS_STORAGE_PATH")]
    pub(crate) storage: Option<PathBuf>,
}

impl Args {
    pub(crate) fn source(&self) -> Source<'_> {
        match (&self.flake, &self.file) {
            (Some(flake), None) => Source::Flake(flake),
            (None, Some(file)) => Source::File(file),
            _ => unreachable!("Enforced by clap"),
        }
    }
}

pub(crate) enum Source<'a> {
    Flake(&'a str),
    File(&'a str),
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
        let mut args = Self::parse();

        // https://github.com/clap-rs/clap/issues/5234
        if args.module_system.is_none() && args.output_name.is_none() {
            args.module_system = Some(DEFAULT_MODULE_SYSTEM.to_string());
        }

        if args.flake.is_some() && args.file.is_some()
            || args.flake.is_some() && args.attr.is_some()
        {
            Args::command()
                .error(
                    ErrorKind::ArgumentConflict,
                    "the argument '--flake <FLAKE>' cannot be used with '--file <FILE>' or '--attr <ATTR>'",
                )
                .exit();
        }

        if args.flake.is_none() && args.file.is_none() {
            Args::command()
                .error(
                    ErrorKind::ArgumentConflict,
                    "the arguments '--flake <FLAKE>' or '--file <FILE>' or are required",
                )
                .exit();
        }

        trace!("Parsing command");
        args.command.execute(&args)?;
        Ok(())
    }
}
