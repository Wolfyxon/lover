use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::process::Command;
use std::process::exit;
use std::process::Stdio;
use ansi_term::Style;
use ansi_term::Color::Blue;
use zip::write::SimpleFileOptions;

use crate::console::{print_err, print_warn, print_success};
use crate::files;
use crate::files::get_file_tree;

pub const PARSER: &str = "luac";

pub fn command_exists(command: &str) -> bool {
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

pub fn execute(command: &str, args: Vec<String>, quiet: bool) -> std::process::ExitStatus {
    if !command_exists(command) {
        print_err(format!("Can't run '{}': not found.", command));
        exit(1);
    }

    if !quiet {
        let prefix = Style::new().fg(Blue).paint("Executing >");
        println!("{} {} {}", prefix, command, args.join(" "));
    }

    let mut pre_run = Command::new(command);
    pre_run.args(args);

    if quiet {
        pre_run.stdout(Stdio::null());
    }

    let res = pre_run.status();
    
    if res.is_err() {
        print_err("Failed to execute command".to_string());    
        exit(1);
    }

    let status: std::process::ExitStatus = res.unwrap();
    let exit_code = status.code().unwrap();

    if status.success() {
        if !quiet {
            println!("");
            print_success(format!("Command completed with code: {}", exit_code));
        }
    } else {
        println!("");
        print_err(format!("Command failed with code: {}", exit_code));
        exit(1);
    }

    status
}

pub fn parse_all(root: &Path) {
    if !command_exists(PARSER) {
        print_warn(format!("'{}' not found. Skipping parse.", PARSER));
        return;
    }

    println!("Parsing Lua scripts...");

    let res = files::get_file_tree_of_type(root, "lua");

    if res.is_err() {
        print_err(format!("Failed to get scripts: {}", res.err().unwrap().to_string()));
        exit(1);
    }

    for script in res.unwrap() {
        execute(PARSER, vec!["-p".to_string(), script.to_str().unwrap().to_string()], true);
    }

    print_success("Parsing successful".to_string());
}

pub fn clean(path: &Path) {
    if !path.exists() {
        print_success("Nothing to clean.".to_string());
        return;
    }

    if path.is_file() {
        print_err(format!("'{}' is not a directory!", path.to_str().unwrap()));
        exit(1);
    }

    let res = std::fs::remove_dir_all(path);
    
    if res.is_err() {
        print_err(format!("Failed to delete '{}': {}", path.to_str().unwrap(), res.err().unwrap()));
        exit(1);
    }
}

pub fn create_dir(path: &Path) {
    let res = std::fs::create_dir_all(path);

    if res.is_err() {
        print_err(format!("Failed to create '{}': {}", path.to_str().unwrap(), res.err().unwrap()));
        exit(1);
    }
}

pub fn archive(source: &Path, output: &Path) {
    create_dir(output.parent().unwrap());
    parse_all(source);
    
    let output_file_res = File::create(output);
    let tree_res = get_file_tree(source);

    if output_file_res.is_err() {
        print_err(format!("Cannot open '{}': {}", output.to_str().unwrap(), output_file_res.err().unwrap()));
        exit(1);
    }

    if tree_res.is_err() {
        print_err(format!("Cannot get source tree: {}", tree_res.err().unwrap()));
        exit(1);
    }

    let output_file = output_file_res.unwrap();
    let options = SimpleFileOptions::default();
    let mut zip = zip::ZipWriter::new(output_file);
    let mut buffer: Vec<u8> = Vec::new();
    
    println!("Archiving {}...", output.to_str().unwrap());

    for path in tree_res.unwrap() {
        let out_path = PathBuf::from_iter(path.components().skip(1));
        let mut file = File::open(path).unwrap();
            
        file.read_to_end(&mut buffer).unwrap();
        zip.start_file_from_path(out_path, options).unwrap();
        zip.write_all(&buffer).unwrap();

        buffer.clear();
    }
}