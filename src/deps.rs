use std::{fs, path::PathBuf, process::exit};
use regex::Regex;
use serde::Deserialize;

use crate::{config, console::{print_err, print_success}, http};

#[derive(Deserialize)]
pub struct GitHubRelease { 
    // Not all fields are needed. Add only those that are necessary.
    pub name: String,
    pub tag_name: String,
    pub html_url: String,
    pub assets: Vec<GithubReleaseAsset>,
    
}

impl GitHubRelease {
    pub fn get_asset_matching(&self, pattern: &str) -> Option<&GithubReleaseAsset> {
        let pattern = Regex::new(pattern).expect("Invalid Regex pattern");

        for asset in &self.assets {
            if asset.matches_pattern(pattern.to_owned()) {
                return Some(&asset);
            }
        }

        None
    }
}

#[derive(Deserialize)]
#[derive(Clone)]
pub struct GithubReleaseAsset {
    pub url: String,
    pub name: String,
    pub size: u32
}

impl GithubReleaseAsset {
    pub fn matches_pattern(&self, regex: Regex) -> bool {
        regex.is_match(&self.name)
    }
}

pub struct Dependency<'a> {
    name: &'a str,
    file_name: &'a str,
    pattern: &'a str,
    repo: &'a str,
    repo_owner: &'a str
}

impl<'a> Dependency<'a> {
    pub fn is_installed(&self) -> bool {
        get_dir().join(self.file_name).exists()
    }

    pub fn fetch_release(&self) -> GitHubRelease {
        fetch_gh_latest_release(&self.repo_owner, &self.repo)
    }

    pub fn get_asset_from_release(&self, release: &GitHubRelease) -> GithubReleaseAsset {
        let asset_res = release.get_asset_matching(&self.pattern);

        if asset_res.is_none() {
            print_err(format!("No file matches pattern '{}' in release. This is a bug!", &self.pattern));
            exit(1);
        }

        asset_res.unwrap().clone()
    }

    pub fn fetch_asset(&self) -> GithubReleaseAsset {
        self.get_asset_from_release(&self.fetch_release())
    }
}

pub fn get_deps<'a>() -> Vec<Dependency<'a>>{
    vec![
        // PC deps
        Dependency {
            name: "linux",
            file_name: "love_linux.AppImage",
            pattern: ".*x86_64.AppImage",
            repo: "love",
            repo_owner: "love2d"
        },
        Dependency {
            name: "win32",
            file_name: "love_win32.zip",
            pattern: ".*win32.zip",
            repo: "love",
            repo_owner: "love2d"
        },
        Dependency {
            name: "win64",
            file_name: "love_win64.zip",
            pattern: ".*win64.zip",
            repo: "love",
            repo_owner: "love2d"
        },

        // Console deps
        /*
        Dependency {
            name: "3ds",
            file_name: "lovepotion_3ds.zip",
            pattern: r"Nintendo\.3DS.*.zip",
            repo: "lovepotion",
            repo_owner: "lovebrew"
        },

        Dependency {
            name: "lovepotion_assets",
            file_name: "lovepotion_assets.zip",
            pattern: "resources.zip",
            repo: "bundler",
            repo_owner: "lovebrew"
        }
        */
    ]
}

pub fn get_dep(name: &str) -> Dependency {
    for dep in get_deps() {
        if dep.name.to_lowercase() == name.to_lowercase() {
            return dep;
        }
    }

    print_err(format!("Unknown dependency '{}'", name));
    exit(1);
}

pub fn get_dir() -> PathBuf {
    config::get_dir().join("deps")
}

pub fn create_dir() {
    let dir = get_dir();

    if !dir.exists() {
        let err = fs::create_dir_all(&dir);

        if err.is_err() {
            println!("Can't create dependency directory at '{}': {}", &dir.to_str().unwrap(), err.err().unwrap());
            exit(1);
        }
    }
}

pub fn fetch_gh_release(owner: &str, repo: &str, release: &str) -> GitHubRelease {
    let url = format!("https://api.github.com/repos/{}/{}/releases/{}", owner, repo, release);
    
    http::fetch_struct(url.as_str())
}

pub fn fetch_gh_latest_release(owner: &str, repo: &str) -> GitHubRelease {
    fetch_gh_release(owner, repo, "latest")
}