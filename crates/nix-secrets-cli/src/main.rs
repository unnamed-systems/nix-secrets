#[macro_use]
extern crate tracing;

#[cfg(target_os = "linux")]
use nix_secrets_core::model::ModuleSystem;
use nix_secrets_core::{Result, log::init_logger};

use clap::CommandFactory;
use clap_complete::CompleteEnv;
use eyre::Ok;

use crate::command::Args;

pub(crate) mod command;
pub(crate) mod utils;

#[cfg(target_os = "macos")]
pub static DEFAULT_MODULE_SYSTEM: ModuleSystem = ModuleSystem::NixDarwin;

#[cfg(target_os = "linux")]
pub static DEFAULT_MODULE_SYSTEM: ModuleSystem = ModuleSystem::Nixos;

fn main() -> Result<()> {
    CompleteEnv::with_factory(Args::command).complete();

    if std::env::var_os("COMPLETE").is_none() {
        init_logger();
    }

    Args::run()?;

    Ok(())
}
