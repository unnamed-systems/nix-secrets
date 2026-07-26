use crate::Result;
use nix::unistd::{Gid, Group, Uid, User};
use serde::Deserialize;
use std::{fmt, path::PathBuf};

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub identity_paths: Vec<String>,
    pub secrets: Vec<Secret>,
    pub storage: PathBuf,
    pub use_placeholders: bool,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Hash)]
#[serde(rename_all = "camelCase")]
pub struct Secret {
    pub owner: OwnerOrGroup,
    pub group: OwnerOrGroup,
    pub mode: String,
    pub name: String,
    pub placeholder: PathBuf,
    pub generator: Option<String>,
    pub path: PathBuf,
    pub recipients: Vec<String>,
    pub needed_for_users: bool,
}

pub fn parse_manifest(input: &str) -> eyre::Result<Manifest> {
    let manifest: Manifest = serde_json::from_str(input)?; // TODO: validate using schema
    trace!("Parsed manifest: {:#?}", manifest);
    Ok(manifest)
}

// TODO: test
