use std::process::{Command, Stdio};

use crate::{Result, manifest};
use eyre::bail;

use crate::manifest::Manifest;

pub(crate) fn parse_flake(flake: &str) -> Option<(String, String)> {
    let hostname = nix::unistd::gethostname().ok();
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

    Some((path.to_string(), attr.to_string()))
}

fn test_flake_support() -> Result<bool> {
    debug!("Checking for flake support");

    Ok(Command::new("nix")
        .arg("eval")
        .arg("--expr")
        .arg("builtins.getFlake")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?
        .success())
}

pub(crate) fn eval_manifest(flake: &str, hostname: &str) -> Result<Manifest> {
    let supports_flakes = test_flake_support()?;
    trace!("Supports flakes: {}", supports_flakes);

    let mut eval_command = if supports_flakes {
        Command::new("nix")
    } else {
        Command::new("nix-instantiate") // TOOD: support nix-instantiate
    };

    trace!("Evaluating flake: {}", format!("{}#{}", flake, hostname));

    eval_command
        .arg("--extra-experimental-features")
        .arg("nix-command flakes")
        .arg("eval")
        .arg("--raw")
        .arg(format!(
            "{flake}#nixosConfigurations.{hostname}.config.security.nix-secrets.manifest"
        ));

    eval_command.stdout(Stdio::piped());

    let build_child = eval_command.stdout(Stdio::piped()).spawn()?;

    let build_output = build_child.wait_with_output()?;

    match build_output.status.code() {
        Some(0) => (),
        _exit_code => {
            bail!("Failed to retrieve manifest");
        }
    }

    let data_json = String::from_utf8(build_output.stdout)?;

    trace!("Retrived manifest: {}", data_json);

    let manifest: Manifest = manifest::parse_manifest(&data_json)?;

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
