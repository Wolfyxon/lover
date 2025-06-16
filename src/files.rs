use std::{fs::{File, OpenOptions}, io::Read, path::{Path, PathBuf}};

use crate::console::exit_err;

pub fn get_file_tree(root: impl Into<PathBuf>) -> Vec<PathBuf> {
    let root = root.into();

    let paths = std::fs::read_dir(root).unwrap_or_else(|err| {
        exit_err(format!("Failed to get file tree: {}", err));
    });

    let mut res: Vec<PathBuf> = Vec::new();

    for entry_res in paths {
        let entry = entry_res.unwrap_or_else(|err| {
            exit_err(format!("File tree read error: {}", err));
        });

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

pub fn get_file_tree_of_type(root: impl Into<PathBuf>, extension: &str) -> Vec<PathBuf> {
    let tree = get_file_tree(root.into());
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

pub fn create_dir(path: impl Into<PathBuf>) {
    let path = path.into();
    
    std::fs::create_dir_all(&path).unwrap_or_else(|err| {
        exit_err(format!("Failed to create directory '{}': {}", path.display(), err));
    })
}

pub fn create(path: impl Into<PathBuf>) -> File {
    let path = path.into();

    File::create(&path).unwrap_or_else(|err| {
        exit_err(format!("Failed to create '{}': {}", path.display(), err));
    })
}

pub fn open(path: impl Into<PathBuf>) -> File {
    let path = path.into();

    File::open(&path).unwrap_or_else(|err| {
         exit_err(format!("Failed to open '{}': {}", path.display(), err));
    })
}

pub fn open_rw(path: impl Into<PathBuf>) -> File {
    let path = path.into();
    let mut options = OpenOptions::new();

    options.read(true).write(true).open(&path).unwrap_or_else(|err| {
        exit_err(format!("Failed to open '{}' with read and write: {}", path.display(), err));
    })
}

pub fn open_append(path: impl Into<PathBuf>) -> File {
    let path = path.into();
    let mut options = OpenOptions::new();

    options.append(true).open(&path).unwrap_or_else(|err| {
        exit_err(format!("Failed to open '{}' for appending: {}", path.display(), err));
    })
}

pub fn get_size(path: impl Into<PathBuf>) -> usize {
    let mut res: usize = 0;
    let mut file = open(path.into());
    
    loop {
        let mut buf: [u8; 1024] = [0; 1024];
        
        let bytes_read = file.read(&mut buf).unwrap_or_else(|err| {
            exit_err(format!("Read failed: {}", err));
        });

        if bytes_read == 0 { 
            break; 
        } else {
            res = res.saturating_add(bytes_read);
        }
    }
    
    res
}

pub fn skip_path(path: impl Into<PathBuf>, path_to_skip: impl Into<PathBuf>) -> PathBuf {
    let path: PathBuf = path.into();
    let path_to_skip: PathBuf = path_to_skip.into();
    
    if !path.starts_with(&path_to_skip) {
        return path;
    }

    PathBuf::from_iter(path.components().skip(path_to_skip.components().count()))
}

pub fn to_current_os_path(string: String) -> String {
    #[cfg(windows)]
    return string.replace("/", "\\");

    #[cfg(unix)]
    return string.replace("\\", "/");
}

pub fn compare_paths(a: impl Into<PathBuf>, b: impl Into<PathBuf>) -> bool {
    let a_str = a.into().to_str().unwrap().to_string();
    let b_str = b.into().to_str().unwrap().to_string();
    
    return to_current_os_path(a_str) == to_current_os_path(b_str);
}
