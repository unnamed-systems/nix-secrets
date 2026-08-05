#[macro_use]
extern crate tracing;

use tracing::level_filters::LevelFilter;

use crate::command::Args;
use std::str::FromStr as _;

pub(crate) mod command;
pub(crate) mod manifest;
pub(crate) mod utils;

pub type Result<T> = eyre::Result<T, eyre::ErrReport>;

pub static SECRETS_DIR_D: &str = "/run/nix-secrets.d";
pub static SECRETS_FOR_USERS_DIR_D: &str = "/run/nix-secrets-for-users.d";

pub static SECRETS_DIR: &str = "/run/nix-secrets";
pub static SECRETS_FOR_USERS_DIR: &str = "/run/nix-secrets-for-users";

pub static SECRETS_EXTENSION: &str = "enc";

fn main() -> Result<()> {
    let default_level = if cfg!(debug_assertions) {
        LevelFilter::TRACE
    } else {
        LevelFilter::INFO
    };

    let filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(default_level.into())
        .from_env()
        .unwrap_or_else(|_| {
            #[expect(clippy::expect_used, reason = "Environment filter is hardcoded")]
            tracing_subscriber::EnvFilter::from_str("error,nix-secrets=info")
                .expect("Environment filter must work")
        });
    tracing_subscriber::fmt()
        .compact()
        .with_env_filter(filter)
        .init();
    trace!("Initialized tracing");

    Args::run()
}
