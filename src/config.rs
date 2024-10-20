use std::path::{Path, PathBuf};
use serde::Deserialize;
use crate::{actions, console::exit_err};

const DKP_TOOLS: &str = "/opt/devkitpro/tools/bin/";

#[derive(Deserialize)]
pub struct Config {
    #[serde(default = "Build::default")]
    pub build: Build,

    #[serde(default = "Run::default")]
    pub run: Run,

    #[serde(default = "Software::default")]
    pub software: Software
}

impl Config {
    pub fn default() -> Self {
        Config {
            build: Build::default(),
            run: Run::default(),
            software: Software::default()
        }
    }

    pub fn parse_str(string: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(string)
    }
}

#[derive(Deserialize)]
pub struct Run {
    pub prime: bool
}

impl Run {
    pub fn default() -> Self {
        Self {
            prime: Run::default_prime()
        }
    }

    fn default_prime() -> bool {
        false
    }
}

#[derive(Deserialize)]
pub struct Build {
    #[serde(default = "Build::default_zip")]
    pub zip: bool,

    #[serde(default = "Build::default_sign")]
    pub sign: bool
}

impl Build {
    pub fn default() -> Self {
        Build {
            zip: Build::default_zip(),
            sign: Build::default_sign()
        }
    }

    fn default_zip() -> bool {
        false
    }

    fn default_sign() -> bool {
        false
    }
}

#[derive(Deserialize)]
pub struct Software {
    #[serde(default = "Software::default_love")]
    pub love: String,

    #[serde(default = "Software::default_luac")]
    pub luac: String,

    #[serde(default = "Software::default_wine")]
    pub wine: String,

    #[serde(default = "Software::default_rcedit")]
    pub rcedit: String,

    #[serde(default = "Software::default_appimagetool")]
    pub appimagetool: String,

    #[serde(default = "Software::default_smdhtool")]
    pub smdhtool: String,

    #[serde(default = "Software::default_3dsxtool")]
    pub threedsxtool: String, // 3dsxtool

    #[serde(default = "Software::default_3dslink")]
    pub threedslink: String, // 3dslink
}

impl Software {
    pub fn check_rcedit(&self) -> Result<(), String> {
        if std::env::consts::FAMILY == "unix" {
            if !actions::command_exists(&self.wine) {
                return Err(format!("Wine is not installed or could not be found at path '{}'.", &self.wine));
            }

            let rcedit = Path::new(&self.rcedit);

            if !rcedit.exists() {
                return Err(format!("RCEdit could not be found at path '{}'.", &self.rcedit));
            }
        } else {
            if !actions::command_exists(&self.rcedit) {
                return Err(format!("RCEdit is not installed or could not be found at path '{}'", &self.rcedit));
            }
        }

        Ok(())
    }

    pub fn default() -> Self {
        Software {
            love: Software::default_love(),
            luac: Software::default_luac(),
            wine: Software::default_wine(),
            rcedit: Software::default_rcedit(),
            appimagetool: Software::default_appimagetool(),
            smdhtool: Software::default_smdhtool(),
            threedsxtool: Software::default_3dsxtool(),
            threedslink: Software::default_3dslink()
        }
    }

    fn default_love() -> String {
        "love".to_string()
    }

    fn default_luac() -> String {
        "luac".to_string()
    }

    fn default_wine() -> String {
        "wine".to_string()
    }

    fn default_rcedit() -> String {
        "rcedit".to_string()
    }

    fn default_appimagetool() -> String {
        "appimagetool".to_string()
    }

    fn default_smdhtool() -> String {
        DKP_TOOLS.to_owned() + "/smdhtool"
    }

    fn default_3dsxtool() -> String {
        DKP_TOOLS.to_owned() + "/3dsxtool"
    }

    fn default_3dslink() -> String {
        DKP_TOOLS.to_owned() + "/3dslink"
    }
        
}

pub fn get_dir() -> PathBuf {
    dirs::data_dir().unwrap().join("lover")
}

pub fn get_config_path() -> PathBuf {
    get_dir().join("config.toml")
}

pub fn exists() -> bool {
    return get_config_path().exists();
}

pub fn get() -> Config {
    if !exists() {
        return Config::default();
    }

    let path: PathBuf = get_config_path();
    
    let string = match std::fs::read_to_string(&path) {
        Ok(string) => string,
        Err(err) => exit_err(format!("Failed to open config at '{}': {}", &path.display(), err))
    };

    match Config::parse_str(string.as_str()) {
        Ok(parsed) => parsed,
        Err(err) =>  exit_err(format!("Config parse error: {}", err))
    }
}