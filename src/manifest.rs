use crate::Result;
use eyre::Context;
use nix::unistd::{Gid, Group, Uid, User};
use serde::{Deserialize, Serialize};
use std::{fmt, path::PathBuf};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub identity_paths: Vec<String>,
    pub secrets: Vec<Secret>,
    pub templates: Vec<Template>,
    pub storage: PathBuf,
    pub use_placeholders: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum OwnerOrGroup {
    Left(String),
    Right(u32),
}

impl fmt::Display for OwnerOrGroup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Left(string) => write!(f, "{string}"),
            Self::Right(int) => write!(f, "{int}"),
        }
    }
}
impl OwnerOrGroup {
    pub(crate) fn get_uid(&self) -> Result<Uid> {
        match self {
            Self::Left(name) => {
                let uid = User::from_name(name)?.map_or_else(|| Uid::from_raw(0), |u| u.uid);
                Ok(uid)
            }
            Self::Right(id) => Ok(Uid::from_raw(*id)),
        }
    }

    pub(crate) fn get_gid(&self) -> Result<Gid> {
        match self {
            Self::Left(name) => {
                let gid = Group::from_name(name)?.map_or_else(|| Gid::from_raw(0), |g| g.gid);
                Ok(gid)
            }
            Self::Right(id) => Ok(Gid::from_raw(*id)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Hash)]
#[serde(rename_all = "camelCase")]
pub struct Secret {
    pub owner: OwnerOrGroup,
    pub group: OwnerOrGroup,
    pub mode: String,
    pub name: String,
    pub placeholder: PathBuf,
    pub generator: Option<Generator>,
    pub path: PathBuf,
    pub recipients: Vec<String>,
    pub template_key: String,
    pub needed_for_users: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Hash)]
#[serde(rename_all = "camelCase")]
pub struct Generator {
    pub derivation: String,
    pub executable: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Hash)]
#[serde(rename_all = "camelCase")]
pub struct Template {
    pub owner: OwnerOrGroup,
    pub group: OwnerOrGroup,
    pub mode: String,
    pub name: String,
    pub path: PathBuf,
    pub needed_for_users: bool,
    pub content: PathBuf,
}

pub fn parse_manifest(input: &str) -> eyre::Result<Manifest> {
    trace!("Parsing manifest: {}", input);
    let manifest: Manifest =
        serde_json::from_str(input).wrap_err("Failed to parse the manifest")?; // TODO: validate using schema
    trace!("Parsed manifest: {:#?}", manifest);
    Ok(manifest)
}

// TODO: test
