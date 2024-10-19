use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use zip::ZipArchive;

use crate::{actions, config, files};
use crate::console::{exit_err, print_err, print_stage, print_warn};
use crate::deps::Dependency;
use crate::project_config;
use crate::deps;

pub struct BuildTarget<'a> {
    pub name: &'a str,
    pub description: &'a str,
    pub deps: Vec<&'a str>,
    pub previous: Vec<&'a str>,
    builder: fn(target: &BuildTarget)
}

impl<'a> BuildTarget<'a> {
    pub fn get_all_dep_names(&self) -> Vec<String> {
        let mut res:Vec<String> = Vec::new();

        for name in &self.deps {
            let dep = deps::get_dep(name);

            res.push(dep.name.to_string());
        }

        res
    }

    pub fn get_all_deps(&self) -> Vec<Dependency> {
        deps::get_deps_by_strings(self.get_all_dep_names())
    }

    pub fn build(&self) {
        (self.builder)(&self);
    }
}

pub fn get_targets<'a>() -> Vec<BuildTarget<'a>> {
    vec![
        BuildTarget {
            name: "love",
            description: "Game's code packaged in the Love format.",
            deps: Vec::new(),
            previous: Vec::new(),
            builder: build_love
        },
        BuildTarget {
            name: "linux",
            description: "Linux AppImage",
            deps: vec!["linux"],
            previous: vec!["love"],
            builder: build_linux
        },
        BuildTarget {
            name: "win64",
            description: "Windows x86_64 EXE",
            deps: vec!["win64"],
            previous: vec!["love"],
            builder: build_win64
        }
    ]
}

pub fn get_target_by_string<'a>(name: String) -> Option<BuildTarget<'a>> {
    for target in get_targets() {
        if target.name == name {
            return Some(target);
        }
    }

    None
}

// for windows targets
pub fn build_windows_zip(name: &str) {
    let project_conf = project_config::get();
    let pkg_name = project_conf.package.name;

    let build_dir = Path::new(&project_conf.directories.build);
    let zip_path = &deps::get_dep(name).get_path();
    let path = build_dir.join(name);

    let love = Path::new(project_conf.directories.build.as_str()).join(format!("{}.love", &pkg_name));

    actions::extract(zip_path, path.as_path());

    let exe_src = path.join("love.exe");

    if !exe_src.exists() {
        exit_err(format!("'{}' could not be found.", &exe_src.to_str().unwrap()));
    }

    print_stage("Embedding the game's code into the executable".to_string());

    actions::append_file(love.as_path(), &exe_src);
}

fn build_love(_target: &BuildTarget) {
    let config = project_config::get();
    let output = Path::new(config.directories.build.as_str()).join(config.package.name + ".love");

    actions::parse_all(Path::new(&project_config::get().directories.source));
    actions::archive(Path::new(config.directories.source.as_str()), &output);
}

fn build_linux(_target: &BuildTarget) {
    let project_conf = project_config::get();
    let conf = config::get();

    let pkg_name = project_conf.package.name;

    // Paths
    let current_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(err) => exit_err(format!("Failed to get current working directory: {}", err))
    };

    let build_dir = Path::new(&project_conf.directories.build);
    let love = Path::new(project_conf.directories.build.as_str()).join(format!("{}.love", &pkg_name));

    let love_app_img = deps::get_dep("linux").get_path();
    
    let squashfs_root = build_dir.join("squashfs-root");
    let love_bin = squashfs_root.join("bin/love");

    // Path checks

    if squashfs_root.exists() {
        print_warn("squashfs-root already exists and will be re-extracted.".to_string());

        let res = std::fs::remove_dir_all(&squashfs_root);
        
        if res.is_err() {
            print_err(format!("Failed to delete '{}': {}", &squashfs_root.to_str().unwrap(), res.err().unwrap()));
        }
    }

    // cd into the build directory
    // (AppImages always unpacks to the current directory and seems like this can't be changed)

    let cd_res = std::env::set_current_dir(&build_dir);

    if cd_res.is_err() {
        exit_err(format!("Failed to change directory to '{}': {}", &build_dir.to_str().unwrap(), cd_res.err().unwrap()));
    }

    // Adding execution perms
    if std::env::consts::FAMILY == "unix" {
        print_stage("Applying execution permission to the Love AppImage".to_string());

        let meta = match std::fs::metadata(&love_app_img) {
            Ok(meta) => meta,
            Err(err) => exit_err(format!("Failed to get file metadata: {}", err))
        };

        let mut perms = meta.permissions();
        perms.set_mode(perms.mode() | 0o755);

        let update_res = std::fs::set_permissions(&love_app_img, perms);

        if update_res.is_err() {
            exit_err(format!("Failed to update permissions: {}", update_res.err().unwrap()));
        }
    }

    // Extracting squashfs-root
    print_stage("Extracting Love2D AppImage contents".to_string());

    actions::execute(love_app_img.to_str().unwrap(), vec!["--appimage-extract".to_string()], true);

    print_stage("Embedding the game's code into the executable".to_string());

    // Reverting the directory change

    let cd_back_res = std::env::set_current_dir(&current_dir);

    if cd_back_res.is_err() {
        exit_err(format!("Failed to revert directory to '{}': {}", &build_dir.to_str().unwrap(), cd_res.err().unwrap()));
    }

    // Appending .love to the love binary
    actions::append_file(love.as_path(), love_bin.as_path());

    // Building .AppImage from squashfs-root

    print_stage("Building .AppImage".to_string());

    let appimage_path = build_dir.join(format!("{}.AppImage", &pkg_name));

    actions::execute(conf.software.appimagetool.as_str(), vec![
        squashfs_root.to_str().unwrap().to_string(), 
        appimage_path.to_str().unwrap().to_string()
    ], false);
}

fn build_win64(_target: &BuildTarget) {
    build_windows_zip("win64");
}