use std::path::PathBuf;
use serde::Deserialize;
use crate::console::exit_err;

//const DKP_TOOLS: &str = "/opt/devkitpro/tools/bin/";

#[derive(Deserialize)]
pub struct Config {
    #[serde(default = "Config::default_verbose_logging")]
    pub verbose_logging: bool,

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
            verbose_logging: Self::default_verbose_logging(),
            build: Build::default(),
            run: Run::default(),
            software: Software::default()
        }
    }

    pub fn parse_str(string: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(string)
    }

    fn default_verbose_logging() -> bool {
        false
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

    /*#[serde(default = "Build::default_sign")]
    pub sign: bool*/
}

impl Build {
    pub fn default() -> Self {
        Build {
            zip: Build::default_zip(),
            //sign: Build::default_sign()
        }
    }

    fn default_zip() -> bool {
        false
    }

    /*fn default_sign() -> bool {
        false
    }*/
}

#[derive(Deserialize)]
pub struct Software {
    #[serde(default = "Software::default_love")]
    pub love: String,

    #[serde(default = "Software::default_luac")]
    pub luac: String,

    #[serde(default = "Software::default_wine")]
    #[allow(dead_code)]
    pub wine: String,

    #[serde(default = "Software::default_rcedit")]
    pub rcedit: String,
    
    /*
    #[serde(default = "Software::default_smdhtool")]
    pub smdhtool: String,

    #[serde(default = "Software::default_3dsxtool")]
    #[serde(rename = "3dsxtool")]
    pub n3dsxtool: String, // 3dsxtool

    #[serde(default = "Software::default_3dslink")]
    #[serde(rename = "3dslink")]
    pub n3dslink: String, // 3dslink
    */
}

impl Software {
    pub fn default() -> Self {
        Software {
            love: Software::default_love(),
            luac: Software::default_luac(),
            wine: Software::default_wine(),
            rcedit: Software::default_rcedit(), /*
            smdhtool: Software::default_smdhtool(),
            n3dsxtool: Software::default_3dsxtool(),
            n3dslink: Software::default_3dslink() */
        }
    }

    fn default_love() -> String {
        #[cfg(target_family = "windows")]
        return "lovec".to_string();

        #[cfg(target_family = "unix")]
        return "love".to_string();
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

    /*
    fn default_smdhtool() -> String {
        DKP_TOOLS.to_owned() + "/smdhtool"
    }

    fn default_3dsxtool() -> String {
        DKP_TOOLS.to_owned() + "/3dsxtool"
    }

    fn default_3dslink() -> String {
        DKP_TOOLS.to_owned() + "/3dslink"
    }*/
        
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
    
    let string = std::fs::read_to_string(&path).unwrap_or_else(|err| {
        exit_err(format!("Failed to open config at '{}': {}", &path.display(), err));
    });

    Config::parse_str(string.as_str()).unwrap_or_else(|err| {
        exit_err(format!("Config parse error: {}", err));
    })
}