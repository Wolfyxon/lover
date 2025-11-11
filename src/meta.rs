use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::{
    console::{exit_err, print_warn},
    files,
};

#[derive(Serialize, Deserialize)]
pub struct ProjectMeta {
    pub files: Vec<FileEntry>,
}

impl ProjectMeta {
    pub fn parse(string: impl Into<String>) -> Result<Self, toml::de::Error> {
        toml::from_str(string.into().as_str())
    }

    pub fn new(source: impl Into<PathBuf>) -> Result<Self, String> {
        let source = source.into();

        let tree = files::get_file_tree(&source);
        let mut entries: Vec<FileEntry> = Vec::new();

        for path in tree {
            entries.push(FileEntry::new(path, &source)?);
        }

        Ok(Self { files: entries })
    }

    pub fn get_changed_files(&self, other: &Self) -> Vec<PathBuf> {
        let mut res: Vec<PathBuf> = Vec::new();

        for entry_a in &self.files {
            let mut found = false;

            for entry_b in &other.files {
                if entry_a.is_changed(entry_b) {
                    found = true;
                    res.push(entry_a.path.to_owned());
                    break;
                } else if entry_a.path_eq(entry_b) {
                    found = true;
                    break;
                }
            }

            if !found {
                res.push(entry_a.path.to_owned());
            }
        }

        for entry_b in &other.files {
            let mut found = false;

            for entry_a in &self.files {
                if entry_b.path_eq(entry_a) {
                    found = true;
                    break;
                }
            }

            if !found && !res.contains(&entry_b.path) {
                res.push(entry_b.path.to_owned());
            }
        }

        res
    }

    pub fn try_save(&self, path: impl Into<PathBuf>) {
        let text = toml::to_string(&self).unwrap_or_else(|err| {
            exit_err(format!(
                "Failed to deserialize project meta: {err}. Please report a bug."
            ));
        });

        let _ = fs::write(path.into(), text).map_err(|err| {
            print_warn(format!("Failed to save project meta: {}", err));
        });
    }
}

#[derive(Serialize, Deserialize)]
pub struct FileEntry {
    pub path: PathBuf,
    pub hash: String,
}

impl FileEntry {
    pub fn new(path: impl Into<PathBuf>, root: impl Into<PathBuf>) -> Result<Self, String> {
        let path = path.into();
        let hash = sha256::try_digest(&path);

        match hash {
            Ok(hash) => Ok(Self {
                path: files::skip_path(path, root),
                hash: hash,
            }),

            Err(err) => Err(err.to_string()),
        }
    }

    pub fn path_eq(&self, other: &Self) -> bool {
        files::compare_paths(&self.path, &other.path)
    }

    pub fn is_changed(&self, recent: &Self) -> bool {
        self.path_eq(recent) && self.hash != recent.hash
    }
}
