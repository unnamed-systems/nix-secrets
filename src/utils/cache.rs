use std::{fs, path::PathBuf};

use eyre::{OptionExt, bail};
use sha2::{Digest as _, Sha256};

use crate::{
    Result,
    manifest::{self, Manifest},
};

fn get_cached_manifest_file_location(flake: &str) -> Result<PathBuf> {
    let state_directory = dirs::state_dir().ok_or_eyre("Failed to get state directory")?;

    let mut hasher = Sha256::new();
    hasher.update(flake);
    let hash_result = hasher.finalize();
    let hashed_flake = format!("{hash_result:x}");

    let manifest_location = state_directory
        .join("nix-secrets")
        .join(hashed_flake)
        .with_extension("json");
    trace!(
        "Got manifest file location: {}",
        manifest_location.display()
    );

    Ok(manifest_location)
}

pub fn get_cached_manifest(flake: &str) -> Result<Manifest> {
    let file_location = get_cached_manifest_file_location(flake)?;

    if file_location.is_file() {
        let content = fs::read_to_string(file_location)?;
        let manifest = manifest::parse_manifest(&content)?;
        return Ok(manifest);
    }

    bail!("Manifest not found");
}

pub fn save_manifest(flake: &str, manifest: &Manifest) -> Result<()> {
    let file_location = get_cached_manifest_file_location(flake)?;
    let parent = file_location
        .parent()
        .ok_or_eyre("Manifest cached location has no parent")?;
    let serialized = serde_json::to_string_pretty(manifest)?;

    fs::create_dir_all(parent)?; // TODO: more atomic?
    fs::write(file_location, serialized)?;
    Ok(())
}
