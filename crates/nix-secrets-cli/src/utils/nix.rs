use std::{
    env, fs,
    process::{Command, Stdio},
};

use crate::utils;
use eyre::{Context, OptionExt, bail};
use nix::sys::utsname::uname;

use crate::FLAKE_CONFIGURATION_PREFIX;
use nix_secrets_core::{
    Result,
    model::{Manifest, ManifestOptions, parse_manifest},
};

static MODULE_SYSTEMS: &[&str] = &[
    "nixosConfigurations",
    "darwinConfigurations",
    "homeConfigurations",
    "hjemConfigurations",
];

pub fn parse_flake_fallback(
    flake: &str,
    fallback: &str,
    canonicalize: bool,
) -> Option<(String, Option<String>, String)> {
    let (path, attr) = match flake.split_once('#') {
        Some((p, "")) => (p, fallback),
        Some((p, a)) => {
            if a.contains('#') || a.contains(char::is_whitespace) {
                return None;
            }
            (p, a)
        }
        None => (flake, fallback),
    };

    let res = if canonicalize {
        std::fs::canonicalize(path)
            .ok()
            .and_then(|p| p.into_os_string().into_string().ok())
            .unwrap_or_else(|| path.to_string())
    } else {
        path.to_string()
    };

    let (module_system, hostname) = match attr.split_once('.') {
        Some((module_system, hostname))
            if !module_system.is_empty()
                && !hostname.is_empty()
                && MODULE_SYSTEMS.contains(&module_system) =>
        {
            (Some(module_system.to_string()), hostname.to_string())
        }
        _ => (None, attr.to_string()),
    };

    Some((res, module_system, hostname))
}

pub fn parse_flake(flake: &str, canonicalize: bool) -> Option<(String, Option<String>, String)> {
    let hostname = uname()
        .ok()
        .and_then(|v| v.nodename().to_str().map(String::from));
    let fallback = hostname.unwrap_or(String::from("default"));

    parse_flake_fallback(flake, &fallback, canonicalize)
}

pub fn eval_build_command(
    var: &str,
    default: &str,
    flake_path: &str,
    output_name: &str,
    configuration_name: &str,
    manifest_args: &str,
    manifest_name: &str,
) -> Result<String> {
    let flake = shlex::try_quote(flake_path)?;
    let output = shlex::try_quote(output_name)?;
    let configuration = shlex::try_quote(configuration_name)?;
    let manifest = shlex::try_quote(manifest_args)?;
    let env_cmd_str = env::var(var)
        .unwrap_or_else(|_| default.to_owned())
        .replace("{{flakePath}}", &flake)
        .replace("{{outputName}}", &output)
        .replace("{{manifestName}}", &manifest_name)
        .replace("{{configurationName}}", &configuration)
        .replace("{{manifestArgs}}", &manifest);
    trace!("Parsed base eval command: {env_cmd_str:?}");

    let cmd = shlex::split(&env_cmd_str).ok_or_eyre("Failed to parse nix command")?;
    let program = cmd.first().ok_or_eyre("No binary provided")?;
    let args = cmd.get(1..).unwrap_or(&[]);

    let mut eval_command = Command::new(program);

    eval_command.args(args);

    eval_command.stdout(Stdio::piped());

    let build_child = eval_command.spawn()?;

    let build_output = build_child.wait_with_output()?;

    match build_output.status.code() {
        Some(0_i32) => (),
        _exit_code => {
            bail!("Failed to retrieve manifest");
        }
    }

    let output = String::from_utf8(build_output.stdout)
        .wrap_err("Failed to convert output stdout to a string")?;

    Ok(output)
}

pub fn eval_manifest(
    flake: &str,
    module_system: Option<&str>,
    hostname: &str,
    options: ManifestOptions,
) -> Result<Manifest> {
    info!("Evaluating flake: {}", format!("{}#{}", flake, hostname));
    let options_json =
        serde_json::to_string(&options).wrap_err("Failed to serialize manifest options")?;

    let build_output = eval_build_command(
        "NIX_SECRETS_NIX_BUILD_COMMAND",
        "nix-build --extra-experimental-features 'flakes' --no-out-link --expr '{flakePath, outputName, configurationName, manifestArgs}: (builtins.getFlake flakePath).${outputName}.${configurationName}.config.security.nix-secrets.manifest (builtins.fromJSON manifestArgs)' --argstr flakePath {{flakePath}} --argstr outputName {{outputName}} --argstr configurationName {{configurationName}} --argstr manifestArgs {{manifestArgs}}",
        &flake,
        &module_system.unwrap_or(FLAKE_CONFIGURATION_PREFIX),
        &hostname,
        &options_json,
        "",
    )?;

    trace!("Evaluating manifest from: {}", build_output.trim());
    let contents =
        fs::read_to_string(build_output.trim()).wrap_err("Failed to read manifest contents")?;

    trace!("Got contents: {}", contents);

    let manifest = parse_manifest(&contents)?;
    trace!("Retrived manifest for {flake}#{hostname}");

    utils::save_manifest(&format!("{flake}#{hostname}"), &manifest)?;
    trace!("Cached manifest");

    Ok(manifest)
}

#[cfg(test)]
mod test_flake_parsing {
    use super::*;

    fn get_attr_fallback() -> String {
        let hostname = uname()
            .ok()
            .and_then(|v| v.nodename().to_str().map(String::from));
        let fallback = hostname.unwrap_or(String::from("default"));

        fallback
    }

    #[test]
    fn parse_flake_both() -> Result<()> {
        let actual = parse_flake("foo#bar", false);
        let expected = Some(("foo".to_string(), None, "bar".to_string()));
        assert_eq!(actual, expected);
        Ok(())
    }
    #[test]
    fn parse_flake_spaces() -> Result<()> {
        let actual = parse_flake("foo bar#buzz", false);
        let expected = Some(("foo bar".to_string(), None, "buzz".to_string()));
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_flake_host_sharp() -> Result<()> {
        let actual = parse_flake("foo#", false);
        let expected = Some(("foo".to_string(), None, get_attr_fallback()));
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_flake_host() -> Result<()> {
        let actual = parse_flake("foo", false);
        let expected = Some(("foo".to_string(), None, get_attr_fallback()));
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_flake_attr() -> Result<()> {
        let actual = parse_flake("#bar", false);
        let expected = Some(("".to_string(), None, "bar".to_string()));
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_flake_empty() -> Result<()> {
        let actual = parse_flake("", false);
        let expected = Some(("".to_string(), None, get_attr_fallback()));
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_flake_macos() -> Result<()> {
        let actual = parse_flake("#laptop.local", false);
        let expected = Some(("".to_string(), None, "laptop.local".to_string()));
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn invalid_parse_flake_space_attr() -> Result<()> {
        let actual = parse_flake("foo# bar", false);
        let expected = None;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn invalid_parse_flake_sharp_attr() -> Result<()> {
        let actual = parse_flake("foo#bar#buz", false);
        let expected = None;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn invalid_parse_flake_escape_attr() -> Result<()> {
        let actual = parse_flake("foo#\nbar", false);
        let expected = None;
        assert_eq!(actual, expected);
        Ok(())
    }
}
