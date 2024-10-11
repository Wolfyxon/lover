use std::process::Command;
use std::process::exit;
use std::process::Output;
use crate::output::{print_err};

pub fn execute(command: &str, args: Vec<String>) -> Output {
    let res = Command::new(command)
        .args(args)
        .output();
    
    if res.is_err() {
        print_err("Failed to execute command".to_string());    
        exit(1);
    }

    let out = res.unwrap();
    
    if !out.status.success() {
        print_err(format!("Command failed with code: {}", out.status.code().unwrap()));
        exit(1);
    }

    out
}