use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::exit;
use std::process::Stdio;
use ansi_term::Style;
use ansi_term::Color::Blue;
use crate::output::{print_err, print_warn};

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
        let prefix = Style::new().fg(Blue).paint("Running >");
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
            println!("Command exited with code: {}", exit_code);
        }
    } else {
        println!("");
        print_err(format!("Command failed with code: {}", exit_code));
        exit(1);
    }

    status
}

pub fn parse_all(root: &Path) {
    let parser = "luac";

    if !command_exists(&parser) {
        print_warn(format!("'{}' not found. Skipping parse.", parser));
        return;
    }

    println!("Parsing Lua scripts...");

    let res = get_file_tree_of_type(root, "lua");

    if res.is_err() {
        print_err(format!("Failed to get scripts: {}", res.err().unwrap().to_string()));
        exit(1);
    }

    for script in res.unwrap() {
        execute(&parser, vec!["-p".to_string(), script.to_str().unwrap().to_string()], true);
    }

    println!("Parsing completed successfully");
}

pub fn get_file_tree(root: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    let paths = std::fs::read_dir(root)?;
    let mut res: Vec<PathBuf> = Vec::new();

    for entry_res in paths {
        let entry = entry_res?;
        let path = entry.path();

        if path.is_file() {
            res.push(path);
        } else {
            let mut sub = get_file_tree(path.as_path())?;
            res.append(&mut sub);
        }
    }

    Ok(res)
}

pub fn get_file_tree_of_type(root: &Path, extension: &str) -> Result<Vec<PathBuf>, std::io::Error> {
    let tree = get_file_tree(root)?;
    let mut res: Vec<PathBuf> = Vec::new();

    for path in tree {
        let ext_res = path.extension();

        if ext_res.is_none() {
            continue;
        }

        if ext_res.unwrap() == extension {
            res.push(path);
        }
    }

    Ok(res)
}