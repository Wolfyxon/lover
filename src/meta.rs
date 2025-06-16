use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::files;

#[derive(Serialize, Deserialize)]
pub struct ProjectMeta {
    pub files: Vec<FileEntry>
}

impl ProjectMeta {
    pub fn parse(string: impl Into<String>) -> Result<Self, toml::de::Error> {
        toml::from_str(string.into().as_str())
    }
}

#[derive(Serialize, Deserialize)]
pub struct FileEntry {
    path: PathBuf,
    hash: String
}

impl FileEntry {
    pub fn new(path: impl Into<PathBuf>, root: impl Into<PathBuf>) -> Result<Self, String> {
        let path = path.into();
        let hash = sha256::try_digest(&path);
        
        match hash {
            Ok(hash) => Ok(Self {
                path: files::skip_path(path, root),
                hash: hash
            }),

            Err(err) => Err(err.to_string())
        }
    }
}
