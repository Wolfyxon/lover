use std::path::Path;
use serde::{Deserialize};
use std::process::exit;
use crate::console::{print_err};

pub const PATH: &str = "lover.toml";

#[derive(Deserialize)]
pub struct Package {
    pub name: String,

    #[serde(default)]
    pub description: String,
    
    #[serde(default)]
    pub author: String,
    
    #[serde(default)]
    pub version: String
}

#[derive(Deserialize)]
pub struct Directories {
    #[serde(default = "def_directories_source")]
    pub source: String,

    #[serde(default = "def_directories_build")]
    pub build: String,
}

#[derive(Deserialize)]
pub struct ProjectConfig {
    pub package: Package,

    #[serde(default = "def_directories")]
    pub directories: Directories    
}

impl ProjectConfig {
    pub fn parse_str(string: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(string)
    }

    pub fn validate(&self) {
        let mut errors: Vec<&str> = Vec::new();

        if self.directories.source == self.directories.build {
            errors.push("Do not attempt to use the same directory for build and source files!");
        }

        if !errors.is_empty() {
            print_err(format!("Invalid project configuration: \n{}", errors.join("\n")));
            exit(1);
        }
    }
}


pub fn exists() -> bool {
    let path = Path::new(PATH);
    return path.exists();
}

pub fn get() -> ProjectConfig {
    let path = Path::new(PATH);

    if !path.exists() {
        print_err(format!("Project config '{}' doesn't exist in the current directory.", path.display()));
        exit(1);
    }

    let string_res = std::fs::read_to_string(path);

    if string_res.is_err() {
        print_err(format!("Failed to open '{}': {}", path.display(), string_res.as_ref().err().unwrap().to_string() ));
        exit(1);
    }

    let parse_res = ProjectConfig::parse_str(string_res.unwrap().as_str());

    if parse_res.is_err() {
        print_err(format!("Config parse error: {}", parse_res.as_ref().err().unwrap().to_string() ));
        exit(1);
    }

    let conf = parse_res.unwrap();
    conf.validate();

    conf
}

fn def_directories() -> Directories {
    Directories {
        source: def_directories_source(),
        build: def_directories_build()
    }
}

fn def_directories_source() -> String {
    "src".to_string()
}

fn def_directories_build() -> String {
    "build".to_string()
}