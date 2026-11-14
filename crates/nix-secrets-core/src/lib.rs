#[macro_use]
extern crate tracing;

pub mod age;
pub mod file;
pub mod log;
pub mod model;

pub type Result<T> = eyre::Result<T, eyre::ErrReport>;

pub static SECRETS_EXTENSION: &str = "enc";
