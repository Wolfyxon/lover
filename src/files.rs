use std::path::{PathBuf, Path};

pub fn get_file_tree(root: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    let paths = std::fs::read_dir(root)?;
    let mut res: Vec<PathBuf> = Vec::new();

    for entry_res in paths {
        let entry = entry_res?;
        let path = entry.path();

        if path.is_file() {
            res.push(path);
        } else {
            let mut sub = get_file_tree(path.as_path())?;
            res.append(&mut sub);
        }
    }

    Ok(res)
}

pub fn get_file_tree_of_type(root: &Path, extension: &str) -> Result<Vec<PathBuf>, std::io::Error> {
    let tree = get_file_tree(root)?;
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

    Ok(res)
}