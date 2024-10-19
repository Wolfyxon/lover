use std::path::{PathBuf, Path};

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