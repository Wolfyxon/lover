use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::process::Command;
use std::process::exit;
use std::process::ExitStatus;
use std::process::Stdio;
use ansi_term::Style;
use ansi_term::Color::Blue;
use zip::write::SimpleFileOptions;
use zip::ZipArchive;

use crate::config;
use crate::console::exit_err;
use crate::console::ProgressBar;
use crate::console::{print_err, print_warn, print_success, print_stage};
use crate::files;
use crate::files::get_file_tree;

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

    let scripts = match files::get_file_tree_of_type(root, "lua") {
        Ok(scripts) => scripts,
        Err(err) => exit_err(format!("Failed to get scripts: {}", err)) 
    };

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

pub fn create_dir(path: &Path) {
    let res = std::fs::create_dir_all(path);

    if res.is_err() {
        exit_err(format!("Failed to create '{}': {}", path.to_str().unwrap(), res.err().unwrap()));
    }
}

pub fn archive(source: &Path, output: &Path) {
    create_dir(output.parent().unwrap());

    let output_file = match File::create(output) {
        Ok(file) => file,
        Err(err) => exit_err(format!("Cannot open '{}': {}", output.to_str().unwrap(), err))
    };

    let tree = match get_file_tree(source) {
        Ok(tree) => tree,
        Err(err) => exit_err(format!("Cannot get source tree: {}", err))
    };

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

pub fn append_file(from: &Path, to: &Path) {
    let mut from_file = match File::open(from) {
        Ok(file) => file,
        Err(err) => exit_err(format!("Failed to open '{}': {}", from.to_str().unwrap(), err))
    };
    
    let to_file_res = OpenOptions::new()
        .append(true)
        .open(to);
    
    if to_file_res.is_err() {
        exit_err(format!("Failed to open '{}': {}", to.to_str().unwrap(), to_file_res.err().unwrap()));
    }

    let mut to_file = to_file_res.unwrap();

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