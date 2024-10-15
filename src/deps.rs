use std::{fs, path::PathBuf, process::exit};
use regex::Regex;
use serde::Deserialize;

use crate::{config, console::{confirm_or_exit, print_err, print_stage, print_success, ProgressBar}, http};

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
    pub browser_download_url: String,
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
    description: &'a str,
    file_name: &'a str,
    pattern: &'a str,
    repo: &'a str,
    repo_owner: &'a str
}

impl<'a> Dependency<'a> {
    pub fn get_path(&self) -> PathBuf {
        get_dir().join(self.file_name)
    }

    pub fn is_installed(&self) -> bool {
        self.get_path().exists()
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

    pub fn download(&self) {
        let asset = self.fetch_asset();
        http::download(&asset.browser_download_url, &self.get_path());
    }
}

pub fn get_deps<'a>() -> Vec<Dependency<'a>>{
    vec![
        // PC deps
        Dependency {
            name: "linux",
            description: "Linux AppImage containing Love2D binaries and required libraries.",
            file_name: "love_linux.AppImage",
            pattern: ".*x86_64.AppImage",
            repo: "love",
            repo_owner: "love2d"
        },
        Dependency {
            name: "win32",
            description: "Zipped Love2D binaries and libraries for Windows x86_32",
            file_name: "love_win32.zip",
            pattern: ".*win32.zip",
            repo: "love",
            repo_owner: "love2d"
        },
        Dependency {
            name: "win64",
            description: "Zipped Love2D binaries and libraries for Windows x86_64",
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
    get_dep_by_string(name.to_string())
}

pub fn get_dep_by_string<'a>(name: String) -> Dependency<'a> {
    for dep in get_deps() {
        if dep.name.to_lowercase() == name.to_lowercase() {
            return dep;
        }
    }

    print_err(format!("Unknown dependency '{}'", name));
    exit(1);
}

pub fn get_deps_by_strings<'a>(names: Vec<String>) -> Vec<Dependency<'a>> {
    let mut res: Vec<Dependency<'a>> = Vec::new();

    for name in names {
        res.push(get_dep_by_string(name));
    }

    res
}

pub fn install(names: Vec<String>) {
    let deps = get_deps_by_strings(names);
    let mut assets: Vec<GithubReleaseAsset> = Vec::new();

    print_stage("Fetching dependencies".to_string());

    let fetch_bar = ProgressBar::new(deps.len());
    let mut fetch_progress: usize = 0;

    fetch_bar.update(fetch_progress);

    for dep in &deps {        
        assets.push(dep.fetch_asset());

        fetch_progress += 1;
        fetch_bar.update(fetch_progress);
    }

    fetch_bar.finish();
    print_stage("The following dependencies will be installed:".to_string());

    let mut total: u32 = 0;

    for i in 0..assets.len() {
        let mut re = "";
        let dep = deps.get(i).unwrap();
        let asset = assets.get(i).unwrap();

        if dep.is_installed() {
            re = "(reinstall)";
        }
        
        total += asset.size;

        println!("  {}: {} MB {}", dep.name, asset.size as f32 / (1024 * 1024) as f32, re);
    }

    println!("\nTotal size: {} MB", total as f32 / (1024 * 1024) as f32);

    confirm_or_exit("Proceed with the installation?");

    create_dir();

    print_stage("Installing...".to_string());

    for i in 0..assets.len() {
        let dep = deps.get(i).unwrap();
        let asset = assets.get(i).unwrap();

        http::download(&asset.browser_download_url, &dep.get_path());
    }

    print_success("All dependencies successfully installed.".to_string());
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