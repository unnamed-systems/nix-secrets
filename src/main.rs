#[macro_use]
extern crate tracing;

use clap::CommandFactory;
use clap_complete::CompleteEnv;
use eyre::Ok;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{
    Layer, Registry, fmt, layer::SubscriberExt as _, util::SubscriberInitExt,
};

use crate::command::Args;

pub(crate) mod command;
pub(crate) mod manifest;
pub(crate) mod utils;

pub type Result<T> = eyre::Result<T, eyre::ErrReport>;

pub static SECRETS_EXTENSION: &str = "enc";

#[cfg(target_os = "macos")]
pub static FLAKE_CONFIGURATION_PREFIX: &str = "darwinConfigurations";

#[cfg(target_os = "linux")]
pub static FLAKE_CONFIGURATION_PREFIX: &str = "nixosConfigurations";

fn main() -> Result<()> {
    CompleteEnv::with_factory(Args::command).complete();

    if std::env::var_os("COMPLETE").is_none() {
        init_logger();
    }

    if let Err(err) = Args::run() {
        error!("{}", err);
        std::process::exit(1);
    }

    Ok(())
}

pub fn init_logger() {
    let _ = simple_eyre::install();
    let is_debug = cfg!(debug_assertions);
    let default_level = if is_debug {
        LevelFilter::TRACE
    } else {
        LevelFilter::INFO
    };
    let filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(default_level.into())
        .from_env_lossy();

    let fmt_layer = fmt::layer()
        .compact()
        .with_target(is_debug)
        .with_writer(std::io::stderr)
        .with_ansi(std::io::IsTerminal::is_terminal(&std::io::stderr()))
        .with_timer(tracing_subscriber::fmt::time::Uptime::default());

    let fmt_layer_final = if is_debug {
        fmt_layer.boxed()
    } else {
        fmt_layer.without_time().boxed()
    };

    Registry::default()
        .with(filter)
        .with(fmt_layer_final)
        .init();

    trace!("Initialized tracing");
}
