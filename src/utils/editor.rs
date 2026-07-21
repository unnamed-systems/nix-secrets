use crate::Result;

use eyre::eyre;

pub(crate) fn is_stdin(editor: &str) -> bool {
    split_editor(editor).is_ok_and(|(program, args)| program == "-" && args.is_none())
}

pub(crate) fn split_editor(editor: &str) -> Result<(String, Option<Vec<String>>)> {
    let mut splitted: Vec<String> =
        shlex::split(editor).ok_or_else(|| eyre!("Could not parse editor"))?;

    if splitted.is_empty() {
        Err(eyre!("Editor is empty"))
    } else {
        let binary = splitted.remove(0);
        let args = (!splitted.is_empty()).then_some(splitted);
        Ok((binary, args))
    }
}

#[cfg(test)]
mod test_split_editor {
    use super::*;

    #[test]
    fn parse_editor_no_args() -> Result<()> {
        let actual = split_editor("vim")?;
        let expected = (String::from("vim"), None);
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_editor_one_arg() -> Result<()> {
        let actual = split_editor("vim -R")?;
        let expected = (String::from("vim"), Some(vec![String::from("-R")]));
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_editor_complex_1() -> Result<()> {
        let actual = split_editor(r#"sed -i "s/.*/ x  /""#)?;
        let expected = (
            String::from("sed"),
            Some(vec![String::from("-i"), String::from("s/.*/ x  /")]),
        );
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_editor_complex_2() -> Result<()> {
        let actual = split_editor(r"sed -i 's/.*/ x  /'")?;
        let expected = (
            String::from("sed"),
            Some(vec![String::from("-i"), String::from("s/.*/ x  /")]),
        );
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_editor_stdin() -> Result<()> {
        let actual = split_editor(r" - ")?;
        let expected = (String::from("-"), None);
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn err_for_empty_editor() {
        let result = split_editor("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Editor is empty");
    }
}
