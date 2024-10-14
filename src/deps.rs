use std::{fs, path::PathBuf, process::exit};

use crate::{config, console::print_success, http};

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