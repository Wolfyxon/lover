use std::{fs::{File, OpenOptions}, path::{Path, PathBuf}};

use crate::console::exit_err;

pub fn get_file_tree(root: &Path) -> Vec<PathBuf> {
    let paths = match std::fs::read_dir(root) {
        Ok(paths) => paths,
        Err(err) => exit_err(format!("Failed to get file tree: {}", err))
    };

    let mut res: Vec<PathBuf> = Vec::new();

    for entry_res in paths {
        let entry = match entry_res {
            Ok(entry) => entry,
            Err(err) => exit_err(format!("File tree read error: {}", err))
        };

        let path = entry.path();

        if path.is_file() {
            res.push(path);
        } else {
            let mut sub = get_file_tree(path.as_path());
            res.append(&mut sub);
        }
    }

    res
}

pub fn get_file_tree_of_type(root: &Path, extension: &str) -> Vec<PathBuf> {
    let tree = get_file_tree(root);
    let mut res: Vec<PathBuf> = Vec::new();

    for path in tree {
        let ext_res = path.extension();

        if ext_res.is_none() {
            continue;
        }

        if ext_res.unwrap() == extension {
            res.push(path);
        }
    }

    res
}

pub fn create_dir(path: &Path) {
    let res = std::fs::create_dir_all(path);

    if res.is_err() {
        exit_err(format!("Failed to create directory '{}': {}", path.to_str().unwrap(), res.err().unwrap()));
    }
}

pub fn create(path: &Path) -> File {
    match File::create(path) {
        Ok(file) => file,
        Err(err) => exit_err(format!("Failed to create '{}': {}", path.to_str().unwrap(), err))
    }
}

pub fn open(path: &Path) -> File {
    match File::open(path) {
        Ok(file) => file,
        Err(err) => exit_err(format!("Failed to open '{}': {}", path.to_str().unwrap(), err))
    }
}

pub fn open_rw(path: &Path) -> File {
    let mut options = OpenOptions::new();

    match options.read(true).write(true).open(path) {
        Ok(file) => file,
        Err(err) => exit_err(format!("Failed to open '{}' with read and write: {}", path.to_str().unwrap(), err))
    }
}

pub fn open_append(path: &Path) -> File {
    let mut options = OpenOptions::new();

    match options.append(true).open(path) {
        Ok(file) => file,
        Err(err) => exit_err(format!("Failed to open '{}' for appending: {}", path.to_str().unwrap(), err))
    }
}

pub fn to_unix_path(string: String) -> String {
    string.replace("\\", "/")
}

pub fn to_windows_path(string: String) -> String {
    string.replace("/", "\\")
}

pub fn to_current_os_path(string: String) -> String {
    #[cfg(windows)]
    return to_windows_path(string);

    #[cfg(unix)]
    return to_unix_path(string);
}