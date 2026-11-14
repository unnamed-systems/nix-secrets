use std::{
    ffi::{OsStr, OsString},
    path::PathBuf,
};

pub trait PathBufExt {
    fn append_extension(self, ext: impl AsRef<OsStr>) -> Self;
}

impl PathBufExt for PathBuf {
    fn append_extension(self, ext: impl AsRef<OsStr>) -> Self {
        let mut os_string: OsString = self.into();
        os_string.push(".");
        os_string.push(ext.as_ref());
        os_string.into()
    }
}

pub fn check_secret_name(name: &str) -> bool {
    !name
        .split('/')
        .any(|c| c == ".." || c == "." || c.is_empty())
}
