use std::{fs, io::Write, path::{Path, PathBuf}};
use crate::{console::{exit_err, input, input_non_empty, print_err, print_note, print_significant, print_success}, files, project_config::{self, Package, ProjectConfig}};

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
    if path.is_file() {
        exit_err(format!("'{}' already exists as a file in the current directory.", name));
    }

    if !path.exists() {
        std::fs::create_dir(path).unwrap_or_else(|err| {
            exit_err(format!("Failed to create directory: {}", err));
        });
    }

    let mut dir = match path.read_dir() {
        Ok(dir) => dir,
        Err(err) =>  exit_err(format!("Failed to read directory: {}", err))
    };

    if dir.next().is_some() {
        exit_err(format!("Directory '{}' must be empty", name));
    }

    extract_template(path);

    /* Generating project config */

    let config_path = path.join(project_config::PROJECT_FILE);

    let pkg = Package {
            name: name.to_owned(),
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
    print_significant("Welcome to the Lover project creator", "");
    println!("You'll be asked a few questions for your project settings and then everything will be set up for you.");
    println!("If a question has '(default: ...)' or '(optional)', just press enter enter if you don't want to change it.");
    println!("Use ^C to abort (press Ctr+C in your terminal)");
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

pub fn setup_dir(project: &mut ProjectConfig) -> PathBuf {
    let name = &project.package.name;
    let mut entered_path = input(format!("Where should your project files be? (default: {}): ", name));
    
    if entered_path.is_empty() {
        entered_path = name.to_string();
    }

    let path = Path::new(&entered_path);

    if path.exists() {
        if path.is_file() {
            print_err("The given path is a file. You must either use an empty directory or a non existing one");
            return setup_dir(project);
        }

        match path.read_dir() {
            Ok(reader) => {
                if reader.count() != 0 {
                    print_err("The project directory must be empty.");
                    return setup_dir(project);
                }
            },
            Err(err) => {
                print_err(format!("Failed to read directory: {}. Consider changing the path", err));
                return setup_dir(project);
            }
        }
    } else {
        match fs::create_dir_all(path) {
            Ok(()) => {},
            Err(err) => {
                print_err(format!("Failed to create directories: {}. Consider changing the path", err));
                return setup_dir(project);
            }
        }
    }

    print_success("Project directory successfully created");
    path.to_owned()
}