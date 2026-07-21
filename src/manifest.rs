use crate::Result;
use nix::unistd::{Gid, Group, Uid, User};
use serde::Deserialize;
use std::{fmt, path::PathBuf};

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Manifest {
    pub(crate) identity_paths: Vec<String>,
    pub(crate) secrets: Vec<Secret>,
    pub(crate) storage: PathBuf,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub(crate) enum OwnerOrGroup {
    Left(String),
    Right(u32),
}

impl fmt::Display for OwnerOrGroup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OwnerOrGroup::Left(string) => write!(f, "{string}"),
            OwnerOrGroup::Right(int) => write!(f, "{int}"),
        }
    }
}
impl OwnerOrGroup {
    pub(crate) fn get_uid(&self) -> Result<Uid> {
        match self {
            OwnerOrGroup::Left(name) => {
                let uid = User::from_name(name)?.map_or_else(|| Uid::from_raw(0), |u| u.uid);
                Ok(uid)
            }
            OwnerOrGroup::Right(id) => Ok(Uid::from_raw(*id)),
        }
    }

    pub(crate) fn get_gid(&self) -> Result<Gid> {
        match self {
            OwnerOrGroup::Left(name) => {
                let gid = Group::from_name(name)?.map_or_else(|| Gid::from_raw(0), |g| g.gid);
                Ok(gid)
            }
            OwnerOrGroup::Right(id) => Ok(Gid::from_raw(*id)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Secret {
    pub(crate) owner: OwnerOrGroup,
    pub(crate) group: OwnerOrGroup,
    pub(crate) mode: String,
    pub(crate) name: String,
    pub(crate) path: PathBuf,
    pub(crate) recipients: Vec<String>,
    pub(crate) needed_for_users: bool,
}

pub(crate) fn parse_manifest(input: &str) -> eyre::Result<Manifest> {
    let manifest: Manifest = serde_json::from_str(input)?; // TODO: validate using schema
    trace!("Parsed manifest: {:#?}", manifest);
    Ok(manifest)
}

// TODO: test
