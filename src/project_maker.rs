use std::{process::exit, fs::read_dir, path::Path};
use crate::console::{print_err, print_success, print_significant};

struct ComponentFile<'a> {
    path: &'a Path,
    buffer: &'a [u8]
} 

fn get_template_files<'a>() -> Vec<ComponentFile<'a>> {
    vec![
        ComponentFile {
            path: Path::new(".gitignore"),
            buffer: include_bytes!("template/.gitignore")
        },
        ComponentFile {
            path: Path::new("src/main.lua"),
            buffer: include_bytes!("template/src/main.lua")
        }
    ]
}

pub fn create(name: String, path: &Path) {
    if path.is_file() {
        print_err(format!("'{}' already exists as a file in the current directory.", name));
        exit(1);
    }

    if !path.exists() {
        let res = std::fs::create_dir(path);

        if res.is_err() {
            print_err(format!("Failed to create directory: {}", res.err().unwrap()));
            exit(1);
        }
    }

    let dir_res = path.read_dir();

    if dir_res.is_err() {
        print_err(format!("Failed to read directory: {}", dir_res.err().unwrap()));
        exit(1);
    }

    let mut dir = dir_res.unwrap();

    if dir.next().is_some() {
        print_err(format!("Directory '{}' must be empty", name));
        exit(1);
    }
}