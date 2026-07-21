use eyre::Result;
use nix::unistd::chown;
use std::collections::hash_map::DefaultHasher;
use std::fs::File;
use std::hash::Hasher;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use crate::manifest::OwnerOrGroup;

pub(crate) fn hash_file<P: AsRef<Path>>(path: P) -> io::Result<u64> {
    let mut file = File::open(path)?;
    let mut hasher = DefaultHasher::new();

    // 8 KB is the standard buffer size (_G_BUFSIZE)
    let mut buffer = [0u8; 8192];

    while let io::Result::Ok(bytes_read) = file.read(&mut buffer) {
        if bytes_read == 0 {
            break;
        }
        hasher.write(&buffer[..bytes_read]);
    }

    Ok(hasher.finish())
}

pub(crate) fn set_owner_and_group(
    path: &PathBuf,
    owner: &OwnerOrGroup,
    group: &OwnerOrGroup,
) -> Result<()> {
    let user = owner.get_uid()?;
    let group = group.get_gid()?;

    chown(path, Some(user), Some(group))?;

    Ok(())
}

pub(crate) fn parse_permissions_str(input: &str) -> eyre::Result<u32> {
    let trimmed = input.strip_prefix('0').unwrap_or(input);

    if trimmed.is_empty() || !trimmed.chars().all(|c| c.is_ascii_digit()) {
        return Err(eyre::eyre!(
            "Failed to parse permissions: invalid numeric format"
        ));
    }

    u32::from_str_radix(trimmed, 8)
        .map_err(|err| eyre::eyre!("Failed to parse permissions: {:?}", err))
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
