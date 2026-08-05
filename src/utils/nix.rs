use std::{
    env,
    process::{Command, Stdio},
};

use crate::{Result, manifest};
use eyre::{Context, OptionExt as _, bail};
use nix::unistd::gethostname;

use crate::manifest::Manifest;

pub fn parse_flake(flake: &str) -> Option<(String, String)> {
    let hostname = gethostname().ok();
    let fallback = hostname
        .as_ref()
        .and_then(|os| os.to_str())
        .unwrap_or("default");

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

    Some((path.to_owned(), attr.to_owned()))
}

pub fn eval_env_command(var: String, default: String, input: String) -> Result<String> {
    let env_cmd_str = env::var(var)
        .unwrap_or(default)
        .replace("{{input}}", &input);
    trace!("Parsed base eval command: {env_cmd_str:?}");

    let cmd = shlex::split(&env_cmd_str).ok_or_eyre("Failed to parse nix command")?;
    let program = cmd.first().ok_or_eyre("No binary provided")?;
    let args = cmd.get(1..).unwrap_or(&[]);

    let mut eval_command = Command::new(program);

    trace!("Evaluating command: {} {}", env_cmd_str, input);

    eval_command.args(args);

    eval_command.stdout(Stdio::piped());
    eval_command.stderr(Stdio::null());

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

pub fn eval_generator(generator: String) -> Result<String> {
    trace!("Evaluating generator: `{generator}`");

    let build_output = eval_env_command(
        "NIX_SECRETS_GENERATOR_BUILD_COMMAND".to_owned(),
        "nix-store --realise".to_owned(),
        generator,
    )?;

    trace!("Retrived generator binary: {}", build_output);

    Ok(build_output)
}

pub fn eval_manifest(flake: String, hostname: String) -> Result<Manifest> {
    trace!("Evaluating flake: {}", format!("{}#{}", flake, hostname));

    let build_output = eval_env_command(
        "NIX_SECRETS_NIX_EVAL_COMMAND".to_owned(),
        "nix --extra-experimental-features \"nix-command flakes\" eval --raw".to_owned(),
        format!("{flake}#nixosConfigurations.{hostname}.config.security.nix-secrets.manifest"),
    )?;

    trace!("Retrived manifest: {}", build_output);

    let manifest: Manifest = manifest::parse_manifest(&build_output)?;

    Ok(manifest)
}

#[cfg(test)]
mod test_flake_parsing {
    use super::*;

    fn get_attr_fallback() -> String {
        let hostname = nix::unistd::gethostname().ok();
        let fallback = hostname
            .as_ref()
            .and_then(|os| os.to_str())
            .unwrap_or("default");
        fallback.to_string()
    }

    #[test]
    fn parse_flake_both() -> Result<()> {
        let actual = parse_flake("foo#bar");
        let expected = Some(("foo".to_string(), "bar".to_string()));
        assert_eq!(actual, expected);
        Ok(())
    }
    #[test]
    fn parse_flake_spaces() -> Result<()> {
        let actual = parse_flake("foo bar#buzz");
        let expected = Some(("foo bar".to_string(), "buzz".to_string()));
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_flake_host_sharp() -> Result<()> {
        let actual = parse_flake("foo#");
        let expected = Some(("foo".to_string(), get_attr_fallback()));
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_flake_host() -> Result<()> {
        let actual = parse_flake("foo");
        let expected = Some(("foo".to_string(), get_attr_fallback()));
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_flake_attr() -> Result<()> {
        let actual = parse_flake("#bar");
        let expected = Some(("".to_string(), "bar".to_string()));
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_flake_empty() -> Result<()> {
        let actual = parse_flake("");
        let expected = Some(("".to_string(), get_attr_fallback()));
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn invalid_parse_flake_space_attr() -> Result<()> {
        let actual = parse_flake("foo# bar");
        let expected = None;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn invalid_parse_flake_sharp_attr() -> Result<()> {
        let actual = parse_flake("foo#bar#buz");
        let expected = None;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn invalid_parse_flake_escape_attr() -> Result<()> {
        let actual = parse_flake("foo#\nbar");
        let expected = None;
        assert_eq!(actual, expected);
        Ok(())
    }
}
