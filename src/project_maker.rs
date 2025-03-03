use std::{fs, io::Write, path::Path};
use crate::{console::{exit_err, print_success}, files, project_config::{self, Package, ProjectConfig}};

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
        exit_err(format!("'{}' already exists as a file in the current directory.", name));
    }

    if !path.exists() {
        let res = std::fs::create_dir(path);

        if res.is_err() {
            exit_err(format!("Failed to create directory: {}", res.err().unwrap()));
        }
    }

    let mut dir = match path.read_dir() {
        Ok(dir) => dir,
        Err(err) =>  exit_err(format!("Failed to read directory: {}", err))
    };

    if dir.next().is_some() {
        exit_err(format!("Directory '{}' must be empty", name));
    }

    /* Template files */

    for component in get_template_files() {
        let target_path = path.join(component.path);
        let parent = target_path.parent().unwrap();

        if !parent.exists() {
            let res = fs::create_dir_all(parent);

            if res.is_err() {
                exit_err(format!("Failed to create directory {}: {}", parent.to_str().unwrap(), res.err().unwrap()));
            }
        }

        let mut file = files::create(&target_path);
        let write_res = file.write_all(component.buffer);

        if write_res.is_err() {
            exit_err(format!("Failed to write file '{}': {}", target_path.to_str().unwrap(), write_res.err().unwrap()));
        }
    }

    /* Generating project config */

    let config_path = path.join(project_config::PATH);

    let pkg = Package {
            name: name.to_owned(),
            author: "".to_string(),
            description: "".to_string(),
            version: Package::default_version(),
            icon: Package::default_icon()
    };

    
    let project = ProjectConfig::from_package(pkg);
    let project_string = toml::to_string_pretty(&project).expect("Serialization failed");
    
    fs::write(config_path, project_string).map_err(|err| {
        exit_err(format!("Failed to create project config: {err}"));
    }).unwrap();

    print_success(format!("Successfully initialized new project '{}' in {}", name, path.to_str().unwrap()));

}