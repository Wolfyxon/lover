use std::path::Path;
use std::process::Command;
use std::process::exit;
use std::process::Stdio;
use ansi_term::Style;
use ansi_term::Color::Blue;

use crate::console::{print_err, print_warn, print_success};
use crate::files;

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