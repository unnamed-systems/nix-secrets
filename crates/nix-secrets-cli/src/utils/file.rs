use eyre::Result;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io;
use std::path::Path;

pub fn hash_file<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    io::copy(&mut file, &mut hasher)?;
    Ok(hasher.finalize().to_vec())
}
