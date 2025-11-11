use serde::Serialize;

// https://lovebrew.org/bundler/getting-started/configuration

#[derive(Serialize)]
pub struct BundlerConfig {
    pub metadata: Metadata,
    pub build: Build,
}

#[derive(Serialize)]
pub struct Metadata {
    pub title: String,
    pub author: String,
    pub description: String,
    pub version: String,
}

#[derive(Serialize)]
pub struct Build {
    pub targets: Vec<String>,
    pub source: String,
    pub packaged: bool,
}
