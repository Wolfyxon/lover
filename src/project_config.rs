use std::{collections::HashMap, path::{Path, PathBuf}};
use serde::{Deserialize, Serialize};
use crate::console::exit_err;

pub const PATH: &str = "lover.toml";

#[derive(Deserialize)]
#[derive(Serialize)]
pub struct ProjectConfig {
    pub package: Package,

    #[serde(default = "Directories::default")]
    #[serde(skip_serializing_if = "Directories::is_default")]
    pub directories: Directories,

    #[serde(default = "Build::default")]
    #[serde(skip_serializing_if = "Build::is_default")]
    pub build: Build,

    #[serde(default = "Run::default")]
    #[serde(skip_serializing_if = "Run::is_default")]
    pub run: Run,

    #[serde(default = "Env::default")]
    #[serde(skip_serializing_if = "Env::is_default")]
    pub env: Env
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

    pub fn from_package(pkg: Package) -> Self {
        Self {
            package: pkg,
            env: Env::default(),
            directories: Directories::default(),
            run: Run::default(),
            build: Build::default()
        }
    }
}

#[derive(Deserialize)]
#[derive(Serialize)]
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

    pub fn default_version() -> String {
        "1.0".to_string()
    }

    pub fn default_icon() -> String {
        "icon.png".to_string()
    }

}

#[derive(Deserialize)]
#[derive(Serialize)]
#[derive(PartialEq)]
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

    pub fn get_build_dir(&self) -> &Path {
        Path::new(&self.build)
    }

    pub fn get_temp_dir(&self) -> PathBuf {
        self.get_build_dir().join("temp")
    }

    pub fn is_default(&self) -> bool {
        return self == &Self::default();
    }
}

#[derive(Deserialize)]
#[derive(Serialize)]
#[derive(PartialEq)]
pub struct Run {
    #[serde(default = "Run::default_default_run_args")]
    pub default_args: Vec<String>
}

impl Run {
    pub fn default() -> Self {
        Self {
            default_args: Self::default_default_run_args()
        }
    }

    fn default_default_run_args() -> Vec<String> {
        Vec::new()
    }

    pub fn is_default(&self) -> bool {
        return self == &Self::default();
    }
}

#[derive(Deserialize)]
#[derive(Serialize)]
#[derive(PartialEq)]
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

    pub fn is_default(&self) -> bool {
        return self == &Self::default();
    }
}

#[derive(Deserialize)]
#[derive(Serialize)]
#[derive(PartialEq)]
pub struct Env {
    #[serde(default = "Env::default_any_env")]
    pub global: HashMap<String, String>,

    #[serde(default = "Env::default_any_env")]
    pub run: HashMap<String, String>,

    #[serde(default = "Env::default_any_env")]
    pub build: HashMap<String, String>
}

impl Env {
    pub fn default() -> Self {
        Self {
            global: Self::default_any_env(),
            run: Self::default_any_env(),
            build: Self::default_any_env()
        }
    }

    pub fn default_any_env() -> HashMap<String, String> {
        HashMap::new()
    }

    pub fn is_default(&self) -> bool {
        return self == &Self::default();
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