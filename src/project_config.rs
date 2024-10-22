use std::{collections::HashMap, path::Path};
use serde::Deserialize;
use crate::console::exit_err;

pub const PATH: &str = "lover.toml";

#[derive(Deserialize)]
pub struct Package {
    pub name: String,

    #[serde(default)]
    pub description: String,
    
    #[serde(default)]
    pub author: String,
    
    #[serde(default = "Package::default_version")]
    pub version: String,

    #[serde(default = "Package::default_icon")]
    pub icon: String
}

impl Package {
    pub fn get_rcedit_args(&self) -> Vec<String> {
        let mut res: Vec<String> = Vec::new();
        let mut map: HashMap<&str, String> = HashMap::new();

        map.insert("ProductName", self.name.to_owned());    
        map.insert("FileDescription", self.description.to_owned());
        map.insert("CompanyName", self.author.to_owned());    
        map.insert("FileVersion", self.version.to_owned());
        map.insert("ProductVersion", self.version.to_owned());
        

        for (k, v) in map {
            res.append(&mut vec![
                "--set-version-string".to_string(),
                k.to_string(),
                v
            ]);
        }

        res
    }

    fn default_version() -> String {
        "1.0".to_string()
    }

    fn default_icon() -> String {
        "icon.png".to_string()
    }

}

#[derive(Deserialize)]
pub struct Directories {
    #[serde(default = "Directories::default_source")]
    pub source: String,

    #[serde(default = "Directories::default_build")]
    pub build: String,
}

impl Directories {
    fn default() -> Self {
        Self {
            source: Self::default_source(),
            build: Self::default_build()
        }
    }

    fn default_source() -> String {
        "src".to_string()
    }

    fn default_build() -> String {
        "build".to_string()
    }
}

#[derive(Deserialize)]
pub struct Build {
    #[serde(default = "Build::default_default")]
    pub default: Vec<String>
}

impl Build {
    fn default() -> Self {
        Self {
            default: Self::default_default()
        }
    }

    fn default_default() -> Vec<String> {
        vec!["love".to_string()]
    }
}

#[derive(Deserialize)]
pub struct ProjectConfig {
    pub package: Package,

    #[serde(default = "Directories::default")]
    pub directories: Directories,

    #[serde(default = "Build::default")]
    pub build: Build
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
            exit_err(format!("Invalid project configuration: \n{}", errors.join("\n")));
        }
    }
}

pub fn get() -> ProjectConfig {
    let path = Path::new(PATH);

    if !path.exists() {
        exit_err(format!("Project config '{}' doesn't exist in the current directory.", path.display()));
    }

    let string = match std::fs::read_to_string(path) {
        Ok(string) => string,
        Err(err) => exit_err(format!("Failed to open '{}': {}", path.display(), err)) 
    };

    let parsed = match ProjectConfig::parse_str(string.as_str()) {
        Ok(parsed) => parsed,
        Err(err) => exit_err(format!("Project config parse error: {}", err)) 
    };

    parsed.validate();
    parsed
}