#[macro_use]
extern crate tracing;

pub mod age;
pub mod model;

pub type Result<T> = eyre::Result<T, eyre::ErrReport>;
