use crate::Result;
use eyre::Context;
use nix::unistd::{Gid, Group, Uid, User};
use serde::{Deserialize, Serialize, Serializer, ser::SerializeMap};
use std::{fmt, path::PathBuf};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub storage_path: Option<PathBuf>,
    pub identity_paths: Vec<String>,
    pub secrets: Vec<Secret>,
    pub templates: Vec<Template>,
    pub storage: PathBuf,
    pub use_placeholders: bool,
    pub module_system: ModuleSystem,
    pub generations_dir: String,
    pub generations_for_users_dir: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum ManifestOptions {
    WithAllGenerators,
    WithGenerators(Vec<String>),
    #[default]
    Default,
}

impl Serialize for ManifestOptions {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ManifestOptions::WithAllGenerators => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("withAllGenerators", &true)?;
                map.end()
            }
            ManifestOptions::WithGenerators(vec) => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("withGenerators", vec)?;
                map.end()
            }
            ManifestOptions::Default => {
                let map = serializer.serialize_map(Some(0))?;
                map.end()
            }
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum ModuleSystem {
    HomeManager,
    Hjem,
    Nixos,
    NixDarwin,
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
    pub placeholder: String,
    pub generator: Option<PathBuf>,
    pub path: PathBuf,
    pub recipients: Vec<String>,
    pub template_key: String,
    pub needed_for_users: bool,
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
