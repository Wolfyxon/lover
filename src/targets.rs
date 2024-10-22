use std::fs;
use std::io::BufReader;
use std::path::Path;

use image::imageops::FilterType;
use image::{GenericImageView, ImageFormat, ImageReader};

use crate::{actions, config, files, util};
use crate::console::{exit_err, print_err, print_significant, print_stage, print_success, print_warn};
use crate::deps::Dependency;
use crate::project_config;
use crate::deps;

pub enum Arch {
    X86_64,
    X86_32
}

impl Arch {
    pub fn get_num_suffix(&self) -> String {
        match &self {
            Self::X86_64 => "64".to_string(),
            Self::X86_32 => "32".to_string()
        }
    }
}

pub struct BuildTarget<'a> {
    pub name: &'a str,
    pub description: &'a str,
    pub deps: Vec<&'a str>,
    pub previous: Vec<&'a str>,
    builder: fn()
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
        print_significant("Building target", self.name.to_string());
        (self.builder)();
        print_success(format!("Successfully built '{}'", self.name));
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
            deps: vec!["love-linux"],
            previous: vec!["love"],
            builder: build_linux
        },
        BuildTarget {
            name: "win64",
            description: "Windows x86_64 EXE",
            deps: vec!["love-win64"],
            previous: vec!["love"],
            builder: build_win64
        },
        BuildTarget {
            name: "win32",
            description: "Windows x86_32 EXE",
            deps: vec!["love-win32"],
            previous: vec!["love"],
            builder: build_win32
        }
    ]
}

// For future use
/*
pub fn get_module_code() -> String {
    let mut map: HashMap<&str, String> = HashMap::new();

    let project_conf = project_config::get();
    let pkg = project_conf.package;

    map.insert("LOVER_VERSION", pkg.version);
    map.insert("LOVER_NAME", pkg.name);
    map.insert("LOVER_AUTHOR", pkg.author);
    map.insert("LOVER_DESCRIPTION", pkg.description);
    
    let mut res = String::new();

    for (key, val) in map {
        res += format!("{} = '{}'\n", key, val).as_str();
    }

    let header = "-- Auto generated by Lover --";
    format!("{}\n{}\n{}", header, res, "-".repeat(header.len()))
}
*/

pub fn get_target_by_string<'a>(name: String) -> BuildTarget<'a> {
    for target in get_targets() {
        if target.name == name {
            return target;
        }
    }

    exit_err(format!("Unknown target '{}'", name));
} 

pub fn get_targets_by_strings<'a>(names: Vec<String>) -> Vec<BuildTarget<'a>> {
    let mut not_found: Vec<String> = Vec::new();
    let mut res: Vec<BuildTarget<'a>> = Vec::new();

    for name in names {
        let mut found = false;

        for target in get_targets() {
            if target.name == name {
                res.push(target);
                found = true;
                break;
            }
        }

        if !found {
            not_found.push(name);
        }
    }

    if not_found.len() != 0 {
        exit_err(format!("Unknown targets: {}", not_found.join(", ")));
    }

    res
} 

// for windows targets
pub fn build_windows_zip(arch: Arch) {
    let name = format!("win{}", arch.get_num_suffix());

    let conf = config::get();
    let project_conf = project_config::get();
    let pkg = project_conf.package;
    let pkg_name = &pkg.name;

    let build_dir = Path::new(&project_conf.directories.build);
    let zip_path = &deps::get_dep(("love-".to_string() + &name).as_str()).get_path();
    let path = build_dir.join(&name);

    let love = Path::new(project_conf.directories.build.as_str()).join(format!("{}.love", &pkg_name));

    actions::extract(zip_path, path.as_path());

    let exe_src = path.join("love.exe");

    if !exe_src.exists() {
        exit_err(format!("'{}' could not be found.", &exe_src.to_str().unwrap()));
    }

    print_stage("Embedding the game's code into the executable".to_string());

    actions::append_file(love.as_path(), &exe_src);

    print_success("The EXE should now be usable, even if something fails.".to_string());

    print_stage("Renaming the EXE".to_string());

    let exe_out = path.join(pkg_name.to_owned() + ".exe");
    let rename_res = fs::rename(&exe_src, &exe_out);

    if rename_res.is_err() {
        exit_err(format!("Failed to rename {}: {}", exe_src.to_str().unwrap(), rename_res.err().unwrap()));
    }

    match conf.software.check_rcedit() {
        Ok(()) => {
            let rcedit = conf.software.rcedit;
            let exe = exe_out.to_str().unwrap().to_string();

            let mut args = vec![exe];
            args.append(&mut pkg.get_rcedit_args());

            let icon_str_path = &pkg.icon;
            
            if !icon_str_path.is_empty() { // TODO: maybe unnest this abomination
                let icon_path = Path::new(icon_str_path);
                let icon_out_path = path.join("game.ico");

                if icon_path.exists() {
                    if !icon_path.is_dir() {

                        let file = files::open(icon_path);
                        let reader = BufReader::new(file);
                        
                        match ImageReader::new(reader).with_guessed_format() {
                            Ok(img_reader) => {
                                match img_reader.decode() {
                                    Ok(mut img) => {
                                        
                                        print_stage("Converting icon to the ICO format".to_string());

                                        let (mut w, mut h) = img.dimensions();

                                        w = util::clamp_u32(w, 16, 256);
                                        h = util::clamp_u32(h, 16, 256);
                                        
                                        img = img.resize(w, h, FilterType::Nearest);

                                        let save_res = img.save_with_format(&icon_out_path, ImageFormat::Ico);

                                        if save_res.is_err() {
                                            print_warn(format!("Failed to save new icon: {}", save_res.err().unwrap()));
                                        }

                                        args.append(&mut vec!["--set-icon".to_string(), icon_out_path.to_str().unwrap().to_string()]);

                                    },
                                    Err(err) => print_warn(format!("Failed to decode image: '{}': {}", icon_str_path, err)),
                                }
                            },
                            Err(err) => print_warn(format!("Failed to read image '{}': {}", icon_str_path, err))
                        };

                    } else {
                        print_warn(format!("Icon '{}' is a directory!", icon_str_path));
                    }
                } else {
                    print_warn(format!("Icon at path '{}' not found.", icon_str_path));
                }
            }

            actions::execute_wine(&rcedit, args, false);
        },
        Err(err) => print_warn(format!("EXE info could not be applied: {}\nPlease check your {}", err, config::get_config_path().to_str().unwrap())),
    };

    if conf.build.zip {
        actions::archive(&path, &build_dir.join(format!("{}_{}.zip", pkg_name, &name)));
    }
    
}

fn build_love() {
    let config = project_config::get();
    let output = Path::new(config.directories.build.as_str()).join(config.package.name + ".love");

    actions::parse_all(Path::new(&project_config::get().directories.source));
    actions::archive(Path::new(config.directories.source.as_str()), &output);
}

fn build_linux() {
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

    let love_app_img = deps::get_dep("love-linux").get_path();
    
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
    #[cfg(unix)] {
        use std::os::unix::fs::PermissionsExt;

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

fn build_win64() {
    build_windows_zip(Arch::X86_64);
}

fn build_win32() {
    build_windows_zip(Arch::X86_32);
}