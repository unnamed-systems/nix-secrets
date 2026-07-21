#![warn(clippy::pedantic)]
#![allow(clippy::too_many_lines)] // Thanks, I know how to write code

#[macro_use]
extern crate tracing;

use std::str::FromStr;

use crate::command::Args;

mod command;
mod manifest;
mod utils;

pub(crate) type Result<T> = eyre::Result<T, Error>;
pub(crate) type Error = eyre::ErrReport;

fn main() -> Result<()> {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        tracing_subscriber::EnvFilter::from_str("error,nix-secrets=info").unwrap()
    });
    tracing_subscriber::fmt()
        .compact()
        .with_env_filter(filter)
        .init();
    trace!("Initialized tracing");

    simple_eyre::install()?;

    let args: Args = argh::from_env();
    args.parse()
}
