use eyre::{Context as _, Result, eyre};
use nix::unistd::chown;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io;
use std::path::Path;

use crate::manifest::OwnerOrGroup;

pub fn hash_file<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    io::copy(&mut file, &mut hasher)?;
    Ok(hasher.finalize().to_vec())
}

pub fn set_owner_and_group(path: &Path, owner: &OwnerOrGroup, group: &OwnerOrGroup) -> Result<()> {
    let uid = owner.get_uid()?;
    let gid = group.get_gid()?;

    chown(path, Some(uid), Some(gid))
        .wrap_err_with(|| format!("Failed to set file permissions: `{}`", path.display()))?;

    Ok(())
}

pub fn parse_permissions_str(input: &str) -> eyre::Result<u32> {
    let trimmed = input.strip_prefix('0').unwrap_or(input);

    if trimmed.is_empty() || !trimmed.chars().all(|c| c.is_ascii_digit()) {
        return Err(eyre!("Failed to parse permissions: invalid numeric format"));
    }

    u32::from_str_radix(trimmed, 8).map_err(|err| eyre!("Failed to parse permissions: {:?}", err))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_permission_string() {
        for (s, r) in [("0700", "700"), ("700", "700"), ("400", "400")] {
            assert_eq!(
                parse_permissions_str(s).unwrap(),
                u32::from_str_radix(r, 8).unwrap()
            );
        }
        assert!(parse_permissions_str("33993").is_err(),);
        assert!(parse_permissions_str("0000111").is_ok(),);
        assert!(parse_permissions_str("1000119").is_err(),);
    }
}
