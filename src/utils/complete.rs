use std::{
    ffi::OsStr,
    process::{Command, Stdio},
};

use clap_complete::{CompletionCandidate, PathCompleter, engine::ValueCompleter};

use crate::FLAKE_CONFIGURATION_PREFIX;
use crate::utils;

// Can't get access to other arguments in the completion function
fn get_flake_native() -> String {
    let mut args = std::env::args_os();

    while let Some(arg) = args.next() {
        if arg == "--flake" || arg == "-f" {
            return args
                .next()
                .and_then(|s| s.into_string().ok())
                .unwrap_or_else(|| ".".to_string());
        }
    }

    ".".to_string()
}

pub fn complete_secrets(current: &OsStr, regenerate: bool) -> Vec<CompletionCandidate> {
    let f = get_flake_native();
    let (flake, module_system, hostname) = utils::parse_flake(&f, true).unwrap_or_default();

    let manifest = utils::get_cached_manifest(&format!(
        "{flake}-{}-{hostname}",
        module_system.unwrap_or_else(|| "default".to_owned())
    ));

    if let Ok(m) = manifest {
        let filter = current.to_str();
        return m
            .secrets
            .into_iter()
            .filter(|v| filter.is_none_or(|f| v.name.starts_with(f)))
            .filter(|v| !regenerate || v.generator.is_some())
            .map(|s| CompletionCandidate::new(s.name))
            .collect();
    }

    vec![]
}

pub fn complete_flake(current: &OsStr) -> Vec<CompletionCandidate> {
    let cur = current.to_str().unwrap_or(".");
    let (flake, module_system, hostname) =
        utils::parse_flake_fallback(cur, "", false).unwrap_or_default();

    let resulting_module_system = module_system
        .as_deref()
        .unwrap_or(FLAKE_CONFIGURATION_PREFIX);

    let flake_str = format!("{flake}#{resulting_module_system}.");

    let output = Command::new("nix") // TODO: add ability to customize command?
        .args(["eval", &flake_str])
        .env("NIX_GET_COMPLETIONS", "2")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    if let Ok(out) = output {
        let stdout_str = String::from_utf8_lossy(&out.stdout);

        let completions: Vec<CompletionCandidate> = stdout_str
            .lines()
            .filter_map(|line| line.strip_prefix(&flake_str))
            .filter(|suffix| hostname.is_empty() || suffix.starts_with(&hostname))
            .map(|suffix| {
                let clean_suffix = suffix.replace('\t', "");
                CompletionCandidate::new(format!("{flake}#{clean_suffix}"))
            })
            .collect();

        if !completions.is_empty() {
            return completions;
        }
    }

    PathCompleter::file().complete(current)
}
