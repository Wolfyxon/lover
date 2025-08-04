use std::{collections::HashMap, env, fs::File, io::Read, path::PathBuf, time::{Duration, SystemTime, UNIX_EPOCH}};
use serde::{Deserialize, Serialize};
use globset::{Glob, GlobSetBuilder};
use crate::{actions::Context, console::{exit_err, print_warn}, files, meta::ProjectMeta, targets};

pub const PROJECT_FILE: &str = "lover.toml";
const IGNORE_MARKER: &str = "---@lover:ignoreFile";

#[derive(Serialize, Deserialize)]
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
    pub fn new(name: impl Into<String>) -> Self {
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

    pub fn get_meta(&self) -> Result<ProjectMeta, String> {
        ProjectMeta::new(self.directories.get_source_dir())
    }

    pub fn get_meta_path(&self) -> PathBuf {
        self.directories.get_temp_dir().join("meta.toml")
    }

    pub fn get_cached_meta(&self) -> Option<ProjectMeta> {
        let path = self.get_meta_path();

        if !path.exists() {
            return None;
        }

        let mut file = files::open(path);
        let mut text = String::new();

        let read_res = file.read_to_string(&mut text);

        if read_res.is_err() {
            print_warn(format!("Failed to read meta cache: {}. Assuming it doesn't exist.", read_res.unwrap_err().to_string()));
            return None;
        }

        let parse_res = ProjectMeta::parse(text);

        if parse_res.is_err() {
            print_warn(format!("Failed to parse meta cache: {}. Assuming it doesn't exist.", parse_res.as_ref().err().unwrap().to_string()));
        }

        Some(parse_res.unwrap())
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

    pub fn get_env_map(&self, context: Context) -> HashMap<String, String> {
        let mut map: HashMap<String, String> = HashMap::new();
    
        let env = &self.env;
        let pkg = &self.package;
    
        let ctx_map = match context {
            Context::Build => &env.build,
            Context::Run => &env.run
        }.to_owned();
    
        for (k, v) in ctx_map {
            map.insert(k, v);
        }
    
        for (k, v) in &env.global {
            map.insert(k.to_owned(), v.to_owned());
        }
    
        let ctx_str = match context {
            Context::Build => "build",
            Context::Run => "run"
        }.to_string();
    
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_else(|err| {
            print_warn(format!("Error getting UNIX timestamp: {}.\nLOVER_TIMESTAMP will be equal to 0", err));
            Duration::from_secs(0)
        }).as_secs();
    
        map.insert("LOVER_CONTEXT".to_string(), ctx_str);
        map.insert("LOVER_TIMESTAMP".to_string(), timestamp.to_string());
    
        map.insert("LOVER_PKG_DISPLAY_NAME".to_string(), pkg.get_display_name());
        map.insert("LOVER_PKG_VERSION".to_string(), pkg.version.to_owned());
        map.insert("LOVER_PKG_NAME".to_string(), pkg.name.to_owned());
        map.insert("LOVER_PKG_AUTHOR".to_string(), pkg.author.to_owned());
        map.insert("LOVER_PKG_DESCRIPTION".to_string(), pkg.description.to_owned());
        
        return map;
    }
}

#[derive(Serialize, Deserialize, PartialEq)]
pub struct Package {
    pub name: String,
    pub copyright: Option<String>,
    pub display_name: Option<String>,

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
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            display_name: None,
            copyright: None,
            description: String::new(),
            author: String::new(),
            version: Self::default_version(),
            icon: Self::default_icon()
        }
    }

    pub fn get_display_name(&self) -> String {
        self.display_name.to_owned().unwrap_or(format!("{} {}", self.name, self.version))
    }

    pub fn get_rcedit_args(&self) -> Vec<String> {
        let mut res: Vec<String> = Vec::new();
        let mut map: HashMap<&str, String> = HashMap::new();

        map.insert("ProductName", self.name.to_owned());    
        map.insert("FileDescription", self.get_display_name());
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

#[derive(Serialize, Deserialize, PartialEq)]
pub struct Directories {
    #[serde(default = "Directories::default_source")]
    pub source: String,

    #[serde(default = "Directories::default_exclude")]
    pub exclude: Vec<String>,

    #[serde(default = "Directories::default_build")]
    pub build: String,
}

impl Directories {
    fn default() -> Self {
        Self {
            source: Self::default_source(),
            exclude: Self::default_exclude(),
            build: Self::default_build()
        }
    }

    fn default_source() -> String {
        "src".to_string()
    }

    fn default_exclude() -> Vec<String> {
        Vec::new()
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

    //`only_ignored == true` -> returns only the ignored files
    //`only_ignored == false` -> returns non-ignored files
    fn filter_files(&self, only_ignored: bool) -> Vec<PathBuf> {
        let src = self.get_source_dir();
        //TODO: Improve explicitly allowed.
        let allowed = ["main.lua", "conf.lua"];

        let mut builder = GlobSetBuilder::new();

        for pat in &self.exclude {
            match Glob::new(pat) {
                Ok(glob) => { builder.add(glob); }
                Err(err) => print_warn(format!("Invalid ignore pattern `{}`: {}", pat, err)),
            }
        }

        if let Ok(glob) = Glob::new("**/.git/**") {
            builder.add(glob);
        };

        let exclude_set = builder.build().expect("Building globset shouldn't fail.");

        files::get_file_tree(self.get_source_dir())
            .into_iter()
            .filter(|path| {
                //Ignore files in build directory
                if path.starts_with(&self.get_build_dir()) {
                    return only_ignored;
                }

                let rel_path = path
                    .strip_prefix(&src)
                    .expect("All paths must be under source")
                    .to_string_lossy()
                    .replace("\\", "/");
            
                let is_allowed = allowed.iter().any(|allowed| rel_path == *allowed);
                let is_ignored = exclude_set.is_match(&rel_path);
                let has_start = Self::has_ignore_marker(path);

                let ignored = is_ignored || has_start;

                if only_ignored {
                    ignored && !is_allowed
                } else {
                    !ignored || is_allowed
                }
            })
            .collect()
    }

    pub fn has_ignore_marker(path: &std::path::Path) -> bool {
        let mut buffer = [0u8; IGNORE_MARKER.len()];
        
        match File::open(path).and_then(|mut f| f.read_exact(&mut buffer)) {
            Ok(_) => std::str::from_utf8(&buffer).map(|s| s == IGNORE_MARKER).unwrap_or(false),
            Err(_) => false,
        }
    }

    pub fn get_ignored_files(&self) -> Vec<PathBuf> {
        self.filter_files(true)
    }

    pub  fn get_files(&self) -> Vec<PathBuf> {
        self.filter_files(false)
    }

    pub fn is_default(&self) -> bool {
        return self == &Self::default();
    }
}

#[derive(Serialize, Deserialize, PartialEq)]
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

#[derive(Serialize, Deserialize, PartialEq)]
pub struct Build {
    pub default: Option<Vec<String>>
}

impl Build {
    fn default() -> Self {
        Self {
            default: None
        }
    }

    pub fn get_default_targets(&self) -> Vec<String> {
        let platform_targets = vec![targets::get_platform_target_name()];

        match &self.default {
            Some(list) => {
                if list.is_empty() {
                    platform_targets
                } else {
                    list.to_owned()
                }
            }
            None => platform_targets
        }
    }
    
    pub fn is_default(&self) -> bool {
        return self == &Self::default();
    }
}

#[derive(Serialize, Deserialize, PartialEq)]
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

    let string = std::fs::read_to_string(&path).unwrap_or_else(|err| {
         exit_err(format!("Failed to open '{}': {}", path.to_str().unwrap(), err)); 
    });

    let parsed = ProjectConfig::parse_str(string.as_str()).unwrap_or_else(|err| {
        exit_err(format!("Project config parse error: {}", err))
    });

    parsed.validate();
    parsed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid() {
        let project = ProjectConfig::parse_str(include_str!("testData/projects/generic.toml")).unwrap();

        assert_eq!(project.package.name, "Some game");
        assert_eq!(project.package.version, "1.2");
    }

    #[test]
    #[should_panic]
    fn parse_syntax_error() {
        ProjectConfig::parse_str(include_str!("testData/projects/invalidSyntax.toml")).unwrap();
    }

    #[test]
    #[should_panic]
    fn parse_missing_name() {
        ProjectConfig::parse_str(include_str!("testData/projects/missingName.toml")).unwrap();
    }

    #[test]
    #[should_panic]
    fn parse_empty() {
        ProjectConfig::parse_str("").unwrap();
    }
}