use std::process::Command;
use std::process::exit;
use std::process::Output;
use ansi_term::Style;
use ansi_term::Color::Blue;
use crate::output::print_err;

pub fn execute(command: &str, args: Vec<String>) -> Output {
    let prefix = Style::new().fg(Blue).paint("Running >");
    println!("{} {} {}", prefix, command, args.join(" "));

    let res = Command::new(command)
        .args(args)
        .output();
    
    if res.is_err() {
        print_err("Failed to execute command".to_string());    
        exit(1);
    }

    let out = res.unwrap();
    
    println!("");
    println!("Command exited with code: {}", out.status.code().unwrap());

    if !out.status.success() {
        print_err("Command failed".to_string());
        exit(1);
    }

    out
}