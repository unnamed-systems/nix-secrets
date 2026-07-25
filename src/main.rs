#[macro_use]
extern crate tracing;

use std::str::FromStr as _;

use crate::command::Args;

pub(crate) mod command;
pub(crate) mod manifest;
pub(crate) mod utils;

pub type Result<T> = eyre::Result<T, Error>;
pub type Error = eyre::ErrReport;

fn main() -> Result<()> {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        #[expect(clippy::expect_used, reason = "Environment filter is hardcoded")]
        tracing_subscriber::EnvFilter::from_str("error,nix-secrets=info")
            .expect("Environment filter must work")
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
