use clap_complete::CompletionCandidate;

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

pub fn complete_secrets(current: &std::ffi::OsStr, regenerate: bool) -> Vec<CompletionCandidate> {
    let f = get_flake_native();
    let (flake, hostname) = utils::parse_flake(&f).unwrap_or_default();
    let flake = format!("{flake}#{hostname}");

    if let Ok(Some(manifest)) = utils::get_cached_manifest(&flake) {
        let filter = current.to_str();
        return manifest
            .secrets
            .into_iter()
            .filter(|v| filter.is_none_or(|f| v.name.starts_with(f)))
            .filter(|v| !regenerate || v.generator.is_some())
            .map(|s| CompletionCandidate::new(s.name))
            .collect();
    }

    vec![]
}
