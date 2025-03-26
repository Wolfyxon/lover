use std::{fs, path::PathBuf};
use regex::Regex;
use serde::Deserialize;

use crate::{config, console::{self, confirm_or_exit, exit_err, print_note, print_step, print_success, print_warn, ProgressBar}, http::{self, Downloadable}};

pub enum RepoDownload<'a> {
    LatestRelease(&'a str), // file pattern
    Source(&'a str) // branch
}

pub enum DependencyInstance<'a> {
    LatestRelease(ReleaseDependency<'a>),
    Source(SourceDependency<'a>)
}

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
    pub name: String
}

impl GithubReleaseAsset {
    pub fn matches_pattern(&self, regex: Regex) -> bool {
        regex.is_match(&self.name)
    }
}

pub struct Dependency<'a> {
    pub name: &'a str,
    pub description: &'a str,
    pub file_name: &'a str,
    pub mode: RepoDownload<'a>,
    pub repo: &'a str,
    pub repo_owner: &'a str
}

impl<'a> Dependency<'a> {
    pub fn get_path(&self) -> PathBuf {
        get_dir().join(self.file_name)
    }

    pub fn is_installed(&self) -> bool {
        self.get_path().exists()
    }

    pub fn get_repo_url(&self) -> String {
        format!("https://github.com/{}/{}", self.repo_owner, self.repo)
    }

    pub fn get_instance(&'a self) -> DependencyInstance<'a> {
        match &self.mode {
            RepoDownload::LatestRelease(pattern) => {
                DependencyInstance::LatestRelease(
                    ReleaseDependency {
                        base: &self,
                        pattern: &pattern
                    }
                )
            },

            RepoDownload::Source(branch) => {
                DependencyInstance::Source(
                    SourceDependency {
                        base: &self,
                        branch: &branch
                    }
                )
            }
        }
    }

    pub fn fetch_download_url(&self) -> String {
        match &self.get_instance() {
            DependencyInstance::LatestRelease(dep) => {
                dep.fetch_asset().browser_download_url
            },
            DependencyInstance::Source(dep) => {
                dep.get_download_url()
            }
        }
    }

    pub fn fetch_downloadable(&self) -> Downloadable {
        Downloadable::request(self.fetch_download_url())
    }
}

pub struct ReleaseDependency<'a> {
    pub base: &'a Dependency<'a>,
    pub pattern: &'a str
}

impl<'a> ReleaseDependency<'a> {
    pub fn fetch_release(&self) -> GitHubRelease {
        fetch_gh_latest_release(&self.base.repo_owner, &self.base.repo)
    }

    pub fn get_asset_from_release(&self, release: &GitHubRelease) -> GithubReleaseAsset {
        let asset_res = release.get_asset_matching(&self.pattern);

        if asset_res.is_none() {
            exit_err(format!("No file matches pattern '{}' in release. This is a bug!", &self.pattern));
        }

        asset_res.unwrap().clone()
    }

    pub fn fetch_asset(&self) -> GithubReleaseAsset {
        self.get_asset_from_release(&self.fetch_release())
    }
}

pub struct SourceDependency<'a> {
    pub base: &'a Dependency<'a>,
    pub branch: &'a str
}

impl<'a> SourceDependency<'a> {
    pub fn get_download_url(&self) -> String {
        format!("https://github.com/{}/{}/archive/refs/heads/{}.zip", self.base.repo_owner, self.base.repo, self.branch)
    }
}

pub fn get_deps<'a>() -> Vec<Dependency<'a>>{
    vec![
        // PC deps
        
        Dependency {
            name: "love-linux",
            description: "Linux AppImage with embedded Love2D binaries and required libraries.",
            file_name: "love_linux.AppImage",
            mode: RepoDownload::LatestRelease(".*x86_64.AppImage"),
            repo: "love",
            repo_owner: "love2d"
        },

        Dependency {
            name: "love-win32",
            description: "Zipped Love2D binaries and libraries for Windows x86_32",
            file_name: "love_win32.zip",
            mode: RepoDownload::LatestRelease(".*win32.zip"),
            repo: "love",
            repo_owner: "love2d"
        },

        Dependency {
            name: "love-win64",
            description: "Zipped Love2D binaries and libraries for Windows x86_64",
            file_name: "love_win64.zip",
            mode: RepoDownload::LatestRelease(".*win64.zip"),
            repo: "love",
            repo_owner: "love2d"
        },

        Dependency {
            name: "rcedit",
            description: "Command line tool for editing EXE binaries",
            file_name: "rcedit.exe",
            mode: RepoDownload::LatestRelease(".*rcedit-x86.exe"),
            repo: "rcedit",
            repo_owner: "electron"
        },

        // Console deps
        
        Dependency {
            name: "lovepotion-3ds",
            file_name: "lovepotion_3ds.zip",
            description: "LovePotion binaries for the 3DS.",
            mode: RepoDownload::LatestRelease(r"Nintendo\.3DS.*.zip"),
            repo: "lovepotion",
            repo_owner: "lovebrew"
        },

        Dependency {
            name: "lovepotion-assets",
            description: "Love2D code and assets for various consoles.",
            file_name: "lovepotion_assets.zip",
            mode: RepoDownload::LatestRelease("resources.zip"),
            repo: "bundler",
            repo_owner: "lovebrew"
        },

        Dependency {
            name: "nest",
            description: "LovePotion console compatibility layer",
            file_name: "nest.zip",
            mode: RepoDownload::Source("master"),
            repo: "nest",
            repo_owner: "lovebrew"
        }
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

    exit_err(format!("Unknown dependency '{}'", name));
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
    let mut downloads: Vec<Downloadable> = Vec::new();

    let mut fetch_bar = ProgressBar::new(deps.len());
    fetch_bar.set_prefix(format!("{} Fetching dependencies", console::get_step_prefix()));

    let mut fetch_progress: usize = 0;

    fetch_bar.update(fetch_progress);

    for dep in &deps {
        downloads.push(dep.fetch_downloadable());

        fetch_progress += 1;
        fetch_bar.update(fetch_progress);
    }

    fetch_bar.finish();
    print_step("The following dependencies will be installed:");

    let mut total: u32 = 0;
    let mut has_unknown = false;

    for i in 0..downloads.len() {
        let mut re = "";
        let dep = deps.get(i).unwrap();
        let download = downloads.get(i).unwrap();

        let len_res = download.len();
        let mut len_text = "unknown".to_string();

        if dep.is_installed() {
            re = "(reinstall)";
        }
        
        match len_res {
            Some(len) => {
                len_text = (len as f32 / (1024 * 1024) as f32).to_string();
                total += len as u32;
            },
            None => {
                has_unknown = true;
            }
        };
        
        println!("  {}: {} MB {}", dep.name, len_text, re);
    }

    println!("\nTotal size: {} MB", total as f32 / (1024 * 1024) as f32);

    if has_unknown {
        print_warn("Size may not be accurate. Failed to retrieve size of some dependencies.");
    }
    
    confirm_or_exit("Proceed with the installation?");

    create_dir();

    print_step("Installing...");

    for i in 0..downloads.len() {
        let dep = deps.get(i).unwrap();
        let download = downloads.get_mut(i).unwrap();

        download.download(&dep.get_path(), dep.name);
    }

    print_success("All dependencies successfully installed.");
    print_note(format!("Dependencies are stored in: {}", get_dir().to_str().unwrap()));
}

pub fn get_dir() -> PathBuf {
    config::get_dir().join("deps")
}

pub fn create_dir() {
    let dir = get_dir();

    if !dir.exists() {
        let err = fs::create_dir_all(&dir);

        if err.is_err() {
            exit_err(format!("Can't create dependency directory at '{}': {}", &dir.to_str().unwrap(), err.err().unwrap()));
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