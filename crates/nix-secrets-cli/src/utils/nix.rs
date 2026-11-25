use std::{
    env, fs,
    process::{Command, Stdio},
};

use crate::{
    command::{Args, Source},
    utils,
};
use eyre::{Context, OptionExt, bail, eyre};
use nix::sys::utsname::uname;

use nix_secrets_core::{
    Result,
    model::{Manifest, ManifestOptions, ModuleSystem, parse_manifest},
};

const MANIFEST_NIX: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../manifest.nix"));

pub fn parse_flake_fallback(
    flake: &str,
    fallback: &str,
    canonicalize: bool,
) -> Option<(String, String)> {
    let (path, hostname) = match flake.split_once('#') {
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

    Some((res, hostname.to_owned()))
}

pub fn parse_flake(flake: &str, canonicalize: bool) -> Option<(String, String)> {
    let hostname = uname()
        .ok()
        .and_then(|v| v.nodename().to_str().map(String::from));
    let fallback = hostname.unwrap_or_else(|| String::from("default"));

    parse_flake_fallback(flake, &fallback, canonicalize)
}

pub fn eval_build_command(
    var: &str,
    default: &str,
    is_flake: bool,
    path: &str,
    module_system: Option<&ModuleSystem>,
    manifest_path: &str,
    manifest_args: &str,
    attr: Option<&str>,
    output_name: Option<&str>,
    configuration_name: &str,
) -> Result<String> {
    let version = shlex::try_quote(env!("CARGO_PKG_VERSION"))?;
    let path = shlex::try_quote(path)?;
    let module_system_string = module_system
        .map(std::string::ToString::to_string)
        .unwrap_or_default();
    let module_system = shlex::try_quote(&module_system_string)?;
    let manifest_path = shlex::try_quote(manifest_path)?;
    let attr = shlex::try_quote(attr.unwrap_or_default())?;
    let output_name = shlex::try_quote(output_name.unwrap_or_default())?;
    let configuration_name = shlex::try_quote(configuration_name)?;
    let env_cmd_str = env::var(var)
        .unwrap_or_else(|_| default.to_owned())
        .replace("{{version}}", &version)
        .replace("{{isFlake}}", &is_flake.to_string())
        .replace("{{path}}", &path)
        .replace("{{moduleSystem}}", &module_system)
        .replace("{{manifestPath}}", &manifest_path)
        .replace("{{manifestArgs}}", manifest_args)
        .replace("{{attr}}", &attr)
        .replace("{{outputName}}", &output_name)
        .replace("{{configurationName}}", &configuration_name);
    trace!("Parsed base eval command: {env_cmd_str:?}");

    let cmd = shlex::split(&env_cmd_str).ok_or_eyre("Failed to parse the nix build command")?;
    let program = cmd
        .first()
        .ok_or_eyre("No binary provided in the nix build command")?;
    let args = cmd.get(1..).unwrap_or(&[]);

    let mut eval_command = Command::new(program);

    eval_command.args(args);

    eval_command.stdout(Stdio::piped());

    let build_child = eval_command
        .spawn()
        .wrap_err_with(|| format!("Failed to spawn `{program}`"))?;

    let build_output = build_child
        .wait_with_output()
        .wrap_err_with(|| format!("Failed to run `{program}`"))?;

    match build_output.status.code() {
        Some(0_i32) => (),
        exit_code => {
            bail!(
                "Failed to build the manifest (exit code: {})",
                exit_code.unwrap_or(-1)
            );
        }
    }

    let output = String::from_utf8(build_output.stdout)
        .wrap_err("Failed to convert the build command output to a string")?;

    Ok(output)
}

pub fn eval_manifest(args: &Args, options: &ManifestOptions) -> Result<Manifest> {
    let source = args.source();
    let is_flake = matches!(source, Source::Flake(_));

    let (path, hostname) = match source {
        Source::Flake(flake) => utils::parse_flake(flake, true)
            .ok_or_else(|| eyre!("Failed to parse the flake `{}`", flake))?,
        Source::File(path) => (path.to_string(), String::new()),
    };
    let options_json =
        serde_json::to_string(&options).wrap_err("Failed to serialize manifest options")?;

    let manifest_path = args.home_manager.as_ref().map_or_else(
        || {
            args.hjem.as_ref().map_or_else(
                || String::from("security.nix-secrets.manifest"),
                |user| format!("hjem.users.{user}.security.nix-secrets.manifest"),
            )
        },
        |user| format!("home-manager.users.{user}.security.nix-secrets.manifest"),
    );

    let quoted_manifest = shlex::try_quote(MANIFEST_NIX)?;

    let build_output = eval_build_command(
        "NIX_SECRETS_NIX_BUILD_COMMAND",
        &format!(
            r"nix-build --expr {} {} --no-out-link --argstr version {{{{version}}}} --arg isFlake {{{{isFlake}}}} --argstr path {{{{path}}}} --argstr moduleSystem {{{{moduleSystem}}}} --argstr manifestPath {{{{manifestPath}}}} --argstr manifestArgs {{{{manifestArgs}}}} --argstr attr {{{{attr}}}} --argstr outputName {{{{outputName}}}} --argstr configurationName {{{{configurationName}}}}",
            quoted_manifest,
            if is_flake {
                "--extra-experimental-features flakes"
            } else {
                ""
            }
        ),
        is_flake,
        &path,
        args.module_system.as_ref(),
        &manifest_path,
        &options_json,
        args.attr.as_deref(),
        args.output_name.as_deref(),
        &hostname,
    )?;

    trace!("Evaluating manifest from: {}", build_output.trim());
    let contents =
        fs::read_to_string(build_output.trim()).wrap_err("Failed to read manifest contents")?;

    trace!("Got contents: {}", contents);

    let mut manifest = parse_manifest(&contents)?;
    trace!("Retrieved manifest for {path}#{hostname}");

    utils::save_manifest(&format!("{path}#{hostname}"), &manifest)?;
    trace!("Cached manifest");

    manifest.identity_paths.extend(args.identity.clone());
    trace!(
        "Extended manifest identities with {} extra identities",
        args.identity.len()
    );

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
        let expected = Some(("foo".to_string(), "bar".to_string()));
        assert_eq!(actual, expected);
        Ok(())
    }
    #[test]
    fn parse_flake_spaces() -> Result<()> {
        let actual = parse_flake("foo bar#buzz", false);
        let expected = Some(("foo bar".to_string(), "buzz".to_string()));
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_flake_host_sharp() -> Result<()> {
        let actual = parse_flake("foo#", false);
        let expected = Some(("foo".to_string(), get_attr_fallback()));
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_flake_host() -> Result<()> {
        let actual = parse_flake("foo", false);
        let expected = Some(("foo".to_string(), get_attr_fallback()));
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_flake_attr() -> Result<()> {
        let actual = parse_flake("#bar", false);
        let expected = Some(("".to_string(), "bar".to_string()));
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_flake_empty() -> Result<()> {
        let actual = parse_flake("", false);
        let expected = Some(("".to_string(), get_attr_fallback()));
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_flake_macos() -> Result<()> {
        let actual = parse_flake("#laptop.local", false);
        let expected = Some(("".to_string(), "laptop.local".to_string()));
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
