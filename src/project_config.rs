use std::{collections::HashMap, env, path::PathBuf};
use serde::{Deserialize, Serialize};
use crate::console::{exit_err, print_warn};

pub const PROJECT_FILE: &str = "lover.toml";

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
    pub fn new(name: String) -> Self {
        Self {
            package: Package::new(name),
            directories: Directories::default(),
            build: Build::default(),
            run: Run::default(),
            env: Env::default()
        }
    }
    
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
#[derive(PartialEq)]
pub struct Package {
    pub name: String,
    pub copyright: Option<String>,

    #[serde(default)]
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    
    #[serde(default)]
    #[serde(skip_serializing_if = "String::is_empty")]
    pub author: String,
    
    #[serde(default = "Package::default_version")]
    pub version: String,

    #[serde(default = "Package::default_icon")]
    #[serde(skip_serializing_if = "Package::is_default_icon")]
    pub icon: String
}

impl Package {
    pub fn new(name: String) -> Self {
        Self {
            name: name,
            copyright: None,
            description: String::new(),
            author: String::new(),
            version: Self::default_version(),
            icon: Self::default_icon()
        }
    }

    pub fn get_rcedit_args(&self) -> Vec<String> {
        let mut res: Vec<String> = Vec::new();
        let mut map: HashMap<&str, String> = HashMap::new();

        map.insert("ProductName", self.name.to_owned());    
        map.insert("FileDescription", self.description.to_owned());
        map.insert("CompanyName", self.author.to_owned());    
        map.insert("ProductVersion", self.version.to_owned());
        
        self.copyright.to_owned().map(|c| {
            map.insert("LegalCopyright", c);
        });

        for (k, v) in map {
            res.append(&mut vec![
                "--set-version-string".to_string(),
                k.to_string(),
                v
            ]);
        }

        // `--set-version-string FileVersion` doesn't work
        res.push("--set-file-version".to_string());
        res.push(self.version.to_owned());

        res
    }

    pub fn default_version() -> String {
        "1.0".to_string()
    }

    pub fn default_icon() -> String {
        "icon.png".to_string()
    }

    pub fn is_default_icon(icon: &String) -> bool {
        return &Self::default_icon() == icon;
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

    pub fn get_root_dir(&self) -> PathBuf {
        find_project_dir().unwrap_or_else(|| {
            exit_err("Failed to find project directory");
        })
    }

    pub fn get_build_dir(&self) -> PathBuf {
        self.get_root_dir().join(&self.build)
    }

    pub fn get_temp_dir(&self) -> PathBuf {
        self.get_build_dir().join("temp")
    }

    pub fn get_source_dir(&self) -> PathBuf {
        self.get_root_dir().join(&self.source)    
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

pub fn find_project_config() -> Option<PathBuf> {
    let mut current = env::current_dir().unwrap();

    loop {
        let current_str = current.to_str().unwrap();

        let dir = current.read_dir().unwrap_or_else(|err| {
            exit_err(format!("Failed to read {}: {}", &current_str, err));
        });

        for entry_res in dir {
            match entry_res {
                Ok(entry) => {
                    let path = entry.path();

                    if entry.file_name() == PROJECT_FILE && path.is_file() {
                        return Some(path);
                    }
                },
                Err(err) => {
                    print_warn(format!("Failed to read entry in {}: {}", current_str, err));
                }
            }
        }

        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => return None
        }
    }
}

pub fn find_project_dir() -> Option<PathBuf> {
    match find_project_config() {
        Some(path) => {
            match path.parent() {
                Some(parent) => Some(parent.to_path_buf()),
                None => None
            }
        },
        None => None
    }
}

pub fn get() -> ProjectConfig {
    let path = find_project_config().unwrap_or_else(|| {
        exit_err(format!("Could not find {} in the current or parent directories.", PROJECT_FILE));
    });

    let string = match std::fs::read_to_string(&path) {
        Ok(string) => string,
        Err(err) => exit_err(format!("Failed to open '{}': {}", path.to_str().unwrap(), err)) 
    };

    let parsed = match ProjectConfig::parse_str(string.as_str()) {
        Ok(parsed) => parsed,
        Err(err) => exit_err(format!("Project config parse error: {}", err)) 
    };

    parsed.validate();
    parsed
}