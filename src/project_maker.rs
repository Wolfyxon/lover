use std::{fs::{self, read_dir, File}, io::Write, path::Path, process::exit};
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

    /* Template files */

    for component in get_template_files() {
        let target_path = path.join(component.path);
        let parent = target_path.parent().unwrap();

        if !parent.exists() {
            let res = fs::create_dir_all(parent);

            if res.is_err() {
                print_err(format!("Failed to create directory {}: {}", parent.to_str().unwrap(), res.err().unwrap()));
                exit(1);
            }
        }

        let file_res = File::create(&target_path);
        
        if file_res.is_err() {
            print_err(format!("Failed to create file {}: {}", &target_path.to_str().unwrap(), file_res.err().unwrap()));
            exit(1);
        }

        let mut file = file_res.unwrap();
        let write_res = file.write_all(component.buffer);

        if write_res.is_err() {
            print_err(format!("Failed to write file '{}': {}", target_path.to_str().unwrap(), write_res.err().unwrap()));
            exit(1);
        }
    }

    /* Generating project config */
}