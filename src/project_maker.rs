use std::{fs, io::Write, path::{Path, PathBuf}, process::exit};
use crate::{console::{exit_err, input, input_non_empty, print_err, print_note, print_success}, files, project_config::{self, Package, ProjectConfig}};

struct ComponentFile<'a> {
    path: &'a Path,
    buffer: &'a [u8]
} 

fn get_template_files<'a>() -> Vec<ComponentFile<'a>> {
    vec![
        ComponentFile {
            path: Path::new(".gitignore"),
            buffer: include_bytes!("lua/template/.gitignore")
        },
        ComponentFile {
            path: Path::new("src/main.lua"),
            buffer: include_bytes!("lua/template/src/main.lua")
        },
        ComponentFile {
            path: Path::new("icon.png"),
            buffer: include_bytes!("lua/template/icon.png")
        }
    ]
}

pub fn extract_template(path: &Path) {
    for component in get_template_files() {
        let target_path = path.join(component.path);
        let parent = target_path.parent().unwrap();

        if !parent.exists() {
            fs::create_dir_all(parent).unwrap_or_else(|err| {
                exit_err(format!("Failed to create directory {}: {}", parent.to_str().unwrap(), err));
            });
        }

        let mut file = files::create(&target_path);
        
        file.write_all(component.buffer).unwrap_or_else(|err| {
            exit_err(format!("Failed to write file '{}': {}", target_path.to_str().unwrap(), err));
        });
    }
}

pub fn create(name: String, path: &Path) {

    if !validate_project_dir(path.to_path_buf()) {
        exit(1);
    }
    
    extract_template(path);

    /* Generating project config */

    let config_path = path.join(project_config::PROJECT_FILE);

    let pkg = Package {
            name: name.to_owned(),
            display_name: None,
            copyright: None,
            author: "".to_string(),
            description: "".to_string(),
            version: Package::default_version(),
            icon: Package::default_icon()
    };

    
    let project = ProjectConfig::from_package(pkg);
    let project_string = toml::to_string_pretty(&project).expect("Serialization failed");
    
    fs::write(config_path, project_string).unwrap_or_else(|err| {
        exit_err(format!("Failed to create project config: {err}"));
    });

    print_success(format!("Successfully initialized new project '{}' in {}", name, path.to_str().unwrap()));

}

pub fn setup() {
    print_note(format!("All settings can be changed at any time you want in the {} file.", project_config::PROJECT_FILE));
    println!();

    setup_init();

    println!();
    println!("See https://github.com/wolfyxon/lover/wiki/project-configuration to see how you can configure your project.");
    print_success("Project successfully created");
}

pub fn setup_init() {
    let name = input_non_empty("Name of your project: ");
    let mut project = ProjectConfig::new(name.to_owned());

    // TODO: Name validation

    let path = setup_dir(&mut project);

    project.package.author = input("Your name (optional): ");
    project.package.description = input("Description (optional): ");

    let project_string = toml::to_string_pretty(&project).expect("Serialization failed");
    let config_path = path.join(project_config::PROJECT_FILE);

    fs::write(config_path, project_string).unwrap_or_else(|err| {
        exit_err(format!("Failed to create project config: {err}"));
    });

    extract_template(&path);
}

pub fn validate_project_dir(path: PathBuf) -> bool {
    if path.exists() {
        if path.is_file() {
            print_err("The given path is a file. You must either use an empty directory or a non existing one");
            return false;
        }
    
        match path.read_dir() {
            Ok(reader) => {
                if reader.count() != 0 {
                    print_err("The project directory must be empty.");
                    return false;
                }
            },
            Err(err) => {
                print_err(format!("Failed to read directory: {}. Consider changing the path", err));
                return false;
            }
        }    
    }

    true
}

pub fn setup_dir(project: &mut ProjectConfig) -> PathBuf {
    let name = &project.package.name;
    let mut entered_path = input(format!("Where should your project files be? (default: {}): ", name));
    
    if entered_path.is_empty() {
        entered_path = name.to_string();
    }

    let path = Path::new(&entered_path);

    if !validate_project_dir(path.to_path_buf()) {
        return setup_dir(project);
    }

    match fs::create_dir_all(path) {
        Ok(()) => {},
        Err(err) => {
            print_err(format!("Failed to create directories: {}. Consider changing the path", err));
            return setup_dir(project);
        }
    }

    print_success("Project directory successfully created");
    path.to_owned()
}