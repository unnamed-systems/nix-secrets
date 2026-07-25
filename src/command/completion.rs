use std::{env, path};

use argh::{ArgsInfo, FromArgValue, FromArgs};
use argh_complete::{Generator as _, bash, fish, nushell, zsh};
use eyre::Ok;

use crate::command::{Args, CommandTrait};

#[derive(Debug, PartialEq, Eq, FromArgValue)]
enum Shell {
    Bash,
    Zsh,
    Fish,
    Nushell,
}

#[derive(FromArgs, ArgsInfo, PartialEq, Eq, Debug)]
/// Generate shell completions
#[argh(subcommand, name = "completion")]
pub struct CompletionCommand {
    /// shell to generate completions for
    #[argh(positional)]
    shell: Shell,
}

impl CommandTrait for CompletionCommand {
    #[expect(clippy::print_stdout, reason = "We write completions code to stdout")]
    fn execute(&self, _root: &Args) -> eyre::Result<()> {
        let cmd_info = Args::get_args_info();
        let command_name = env::args().next().map_or_else(String::new, |arg0| {
            path::Path::new(&arg0)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        });
        match self.shell {
            Shell::Bash => {
                println!("{}", bash::Bash::generate(&command_name, &cmd_info));
            }
            Shell::Zsh => {
                println!("{}", zsh::Zsh::generate(&command_name, &cmd_info));
            }
            Shell::Fish => {
                println!("{}", fish::Fish::generate(&command_name, &cmd_info));
            }
            Shell::Nushell => {
                println!("{}", nushell::Nushell::generate(&command_name, &cmd_info));
            }
        }

        Ok(())
    }
}
