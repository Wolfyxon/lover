use serde::Deserialize;

const DKP_TOOLS: &str = "/opt/devkitpro/tools/bin/";

#[derive(Deserialize)]
pub struct Config {
    #[serde(default = "Build::default")]
    build: Build,

    #[serde(default = "Software::default")]
    software: Software
}

impl Config {
    pub fn default() -> Self {
        Config {
            build: Build::default(),
            software: Software::default()
        }
    }

    pub fn parse_str(string: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(string)
    }
}

#[derive(Deserialize)]
pub struct Build {
    #[serde(default = "Build::default_zip")]
    zip: bool,

    #[serde(default = "Build::default_sign")]
    sign: bool
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
    #[serde(default = "Software::default_luac")]
    luac: String,

    #[serde(default = "Software::default_appimagetool")]
    appimagetool: String,

    #[serde(default = "Software::default_wine")]
    wine: String,

    #[serde(default = "Software::default_rcedit")]
    rcedit: String,

    #[serde(default = "Software::default_smdhtool")]
    smdhtool: String,

    #[serde(default = "Software::default_3dsxtool")]
    threedsxtool: String, // 3dsxtool

    #[serde(default = "Software::default_3dslink")]
    threedslink: String, // 3dslink
}

impl Software {
    pub fn default() -> Self {
        Software {
            luac: Software::default_luac(),
            appimagetool: Software::default_appimagetool(),
            wine: Software::default_wine(),
            rcedit: Software::default_rcedit(),
            smdhtool: Software::default_smdhtool(),
            threedsxtool: Software::default_3dsxtool(),
            threedslink: Software::default_3dslink()
        }
    }

    fn default_luac() -> String {
        "luac".to_string()
    }

    fn default_appimagetool() -> String {
        "appimagetool".to_string()
    }

    fn default_wine() -> String {
        "wine".to_string()
    }

    fn default_rcedit() -> String {
        "rcedit".to_string()
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