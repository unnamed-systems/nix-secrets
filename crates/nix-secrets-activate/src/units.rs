use std::{
    collections::HashSet,
    fs::{self, OpenOptions},
    io::{self, BufWriter, Write as _},
    os::unix::fs::OpenOptionsExt as _,
    path::Path,
    process::{Command, Stdio},
};

use eyre::bail;
use nix_secrets_core::Result;

#[derive(Clone, Copy)]
pub enum Verb {
    TryRestart,
    TryReloadOrRestart,
}

pub fn run_systemctl<S: ::std::hash::BuildHasher>(
    verb: Verb,
    units: &HashSet<String, S>,
) -> Result<()> {
    if units.is_empty() {
        return Ok(());
    }

    let verb_str = match verb {
        Verb::TryRestart => "try-restart",
        Verb::TryReloadOrRestart => "try-reload-or-restart",
    };
    let status = Command::new("systemctl")
        .arg("--no-block")
        .arg(verb_str)
        .args(units)
        .stdout(Stdio::from(io::stderr()))
        .stderr(Stdio::inherit())
        .status()?;

    if !status.success() {
        bail!("Failed to run systemctl")
    }

    Ok(())
}

pub fn append_units<S: ::std::hash::BuildHasher>(
    units: &HashSet<String, S>,
    path: &Path,
) -> io::Result<()> {
    if units.is_empty() {
        return Ok(());
    }

    if let Some(parent) = path.parent() {
        match fs::metadata(parent) {
            Ok(_) => {}
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(()),
            Err(e) => return Err(e),
        }
    }

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .mode(0o600)
        .open(path)?;

    let mut writer = BufWriter::new(file);
    for unit in units {
        writeln!(writer, "{unit}")?;
    }
    writer.flush()
}
