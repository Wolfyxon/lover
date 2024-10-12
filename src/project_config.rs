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
pub struct ProjectConfig {    
    #[serde(default = "def_source")]
    pub source: String,

    #[serde(default = "def_build")]
    pub build: String,

    pub package: Package
}

impl ProjectConfig {
    pub fn parse_str(string: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(string)
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

    parse_res.unwrap()
}

fn def_source() -> String {
    "src".to_string()
}

fn def_build() -> String {
    "build".to_string()
}