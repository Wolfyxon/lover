
use std::collections::HashMap;
use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::process::Command;
use std::process::ExitStatus;
use std::process::Stdio;
use ansi_term::Style;
use ansi_term::Color::Blue;
use zip::write::SimpleFileOptions;
use zip::ZipArchive;
use crate::config;
use crate::console::exit_err;
use crate::console::ProgressBar;
use crate::console::{print_warn, print_success, print_stage};
use crate::files;
use crate::files::get_file_tree;
use crate::project_config;

pub fn command_exists(command: &str) -> bool {
    let as_path = Path::new(command);
    
    if as_path.is_file() {
        return true;
    }
    
    let path_env_res = std::env::var_os("PATH");

    if path_env_res.is_none() {
        return false;
    }

    for path_dir in std::env::split_paths(&path_env_res.unwrap()) {
        if path_dir.join(command).is_file() {
            return true;
        }
    }

    return false;
}

pub fn execute_with_env(command: &str, args: Vec<String>, env: HashMap<&str, &str>, quiet: bool) -> ExitStatus {
    if !command_exists(command) {
        exit_err(format!("Can't run '{}': not found.", command));
    }

    if !quiet {
        let prefix = Style::new().fg(Blue).paint("Executing >");
        println!("{} {} {}", prefix, command, args.join(" "));
    }

    let mut pre_run = Command::new(command);
    pre_run.args(args);
    
    for (key, value) in env {
        pre_run.env(key, value);
    }

    if quiet {
        pre_run.stdout(Stdio::null());
    }

    let status = match pre_run.status() {
        Ok(status) => status,
        Err(err) =>  exit_err(format!("Failed to execute command: {}", err)) 
    };
    
    let exit_code = status.code().unwrap();

    if status.success() {
        if !quiet {
            println!("");
            print_success(format!("Command completed with code: {}", exit_code));
        }
    } else {
        println!("");
        exit_err(format!("Command failed with code: {}", exit_code));
    }

    status
}

pub fn execute(command: &str, args: Vec<String>, quiet: bool) -> ExitStatus{
    execute_with_env(command, args, HashMap::new(), quiet)
}

pub fn execute_wine(command: &str, mut args: Vec<String>, quiet: bool) -> ExitStatus {
    if std::env::consts::FAMILY == "unix" {
        let mut all_args = vec![command.to_string()];
        all_args.append(&mut args);

        let mut env: HashMap<&str, &str> = HashMap::new();
        env.insert("WINEDEBUG", "-all");

        execute_with_env(&config::get().software.wine, all_args, env, quiet)
    } else {
        execute(command, args, quiet)
    }
}

pub fn execute_prime(command: &str, args: Vec<String>, quiet: bool) -> ExitStatus {
    let mut env = HashMap::new();

    env.insert("__NV_PRIME_RENDER_OFFLOAD", "1");
    env.insert("__GLX_VENDOR_LIBRARY_NAME", "nvidia");
    env.insert("__VK_LAYER_NV_optimus", "NVIDIA_only");
    env.insert("VK_ICD_FILENAMES", "/usr/share/vulkan/icd.d/nvidia_icd.json");
    
    execute_with_env(command, args, env, quiet)
}


pub fn parse_all(root: &Path) {
    let parser = config::get().software.luac;

    if !command_exists(&parser) {
        print_warn(format!("'{}' not found. Skipping parse.", &parser));
        return;
    }

    print_stage("Parsing Lua scripts...".to_string());

    let scripts = files::get_file_tree_of_type(root, "lua");
    
    for script in scripts {
        execute(&parser, vec!["-p".to_string(), script.to_str().unwrap().to_string()], true);
    }

    print_success("Parsing successful".to_string());
}

pub fn clean(path: &Path) {
    if !path.exists() {
        print_success("Nothing to clean.".to_string());
        return;
    }

    if path.is_file() {
        exit_err(format!("'{}' is not a directory!", path.to_str().unwrap()));
    }

    match std::fs::remove_dir_all(path) {
        Ok(()) => print_success("Clean successful".to_string()),
        Err(err) => exit_err(format!("Failed to delete '{}': {}", path.to_str().unwrap(), err))
    };
}

pub fn archive(source: &Path, output: &Path) {
    files::create_dir(output.parent().unwrap());

    let output_file = files::create(output);
    let tree = get_file_tree(source);
    let options = SimpleFileOptions::default();
    let mut zip = zip::ZipWriter::new(output_file);
    let mut buffer: Vec<u8> = Vec::new();
    let mut progress: usize = 0;
    let bar = ProgressBar::new(tree.len());

    print_stage(format!("Archiving '{}' into '{}'...", source.to_str().unwrap(), output.to_str().unwrap()));
    
    for path in tree {
        let out_path = PathBuf::from_iter(path.components().skip(1));
        let mut file = File::open(path).unwrap();
            
        file.read_to_end(&mut buffer).unwrap();
        zip.start_file_from_path(out_path, options).unwrap();
        zip.write_all(&buffer).unwrap();

        buffer.clear();

        progress += 1;
        bar.update(progress);
    }

    bar.finish();
}

pub fn extract(from_zip: &Path, to_dir: &Path) {
    files::create_dir(to_dir);

    let zip_file = files::open(from_zip);

    let mut archive = match ZipArchive::new(zip_file) {
        Ok(archive) => archive,
        Err(err) => exit_err(format!("ZIP failed for '{}': {}", from_zip.to_str().unwrap(), err))
    };

    let archive_len = archive.len();

    print_stage(format!("Extracting '{}' to '{}'...", from_zip.to_str().unwrap(), to_dir.to_str().unwrap()));

    let bar = ProgressBar::new(archive_len);
    bar.update(0);

    for i in 0..archive_len {
        let mut file = match archive.by_index(i) {
            Ok(file) => file,
            Err(err) => exit_err(format!("Failed to get file at index {} of '{}': {}", i, from_zip.to_str().unwrap(), err))
        };
        
        let path = match file.enclosed_name() {
            Some(path) => {
                PathBuf::from(path.file_name().unwrap())

                // Enable if nested files are needed.
                /*
                let mut res = path.to_owned();
                let components = path.components();
                let cmp_len = components.to_owned().count();

                if cmp_len > 1 {
                    let mut new_path = PathBuf::new();
                    let mut skipped = false;

                    for cmp in components {
                        if !skipped {
                            skipped = true;
                            continue;
                        }

                        new_path = new_path.join(cmp);
                    }

                    res = new_path;
                }

                res*/
            },
            None => exit_err(format!("Failed to get path of file at index {} of '{}'", i, from_zip.to_str().unwrap()))
        };

        let mut out_file = files::create(to_dir.join(path).as_path());
        
        loop {
            let mut buf: [u8; 1024] = [0; 1024];
            
            let bytes_read = match file.read(&mut buf) {
                Ok(bytes_read) => bytes_read,
                Err(err) =>  exit_err(format!("Read failed: {}", err))
            };
    
            if bytes_read == 0 { break; }
    
            let write_res = out_file.write_all(&buf[..bytes_read]);
    
            if write_res.is_err() {
                exit_err(format!("Write failed: {}", write_res.err().unwrap()));
            }
        }

        bar.update(i + 1);
    }

    bar.finish();
}

pub fn append_file(from: &Path, to: &Path) {
    let mut from_file = files::open(from);
    let mut to_file = files::open_append(to);

    loop {
        let mut buf: [u8; 1024] = [0; 1024];
        
        let bytes_read = match from_file.read(&mut buf) {
            Ok(bytes_read) => bytes_read,
            Err(err) =>  exit_err(format!("Read failed: {}", err))
        };

        if bytes_read == 0 { break; }

        let write_res = to_file.write_all(&buf[..bytes_read]);

        if write_res.is_err() {
            exit_err(format!("Write failed: {}", write_res.err().unwrap()));
        }
    }
}

pub fn get_env_map<'a>() -> HashMap<&'a str, String> {
    let mut map: HashMap<&str, String> = HashMap::new();

    let project_conf = project_config::get();
    let pkg = project_conf.package;

    map.insert("LOVER_VERSION", pkg.version);
    map.insert("LOVER_NAME", pkg.name);
    map.insert("LOVER_AUTHOR", pkg.author);
    map.insert("LOVER_DESCRIPTION", pkg.description);

    return map;
}