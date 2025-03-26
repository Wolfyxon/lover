use std::io::{stdin, stdout, BufRead, Read, Write};
use std::process::exit;
use ansi_term::Style;
use ansi_term::Color::{Red, Yellow, Green, Cyan, Blue, Purple};

use crate::config;

pub struct CommandLineSettings {
    pub args: Vec<String>,
    pub flags: Vec<String>,
    pub verbose: bool,
}

impl CommandLineSettings {
    pub fn get_command_alias(&self) -> Option<&String> {
        return self.args.get(0);
    }

    pub fn has_flag(&self, flag: &str) -> bool {
        return self.flags.contains(&flag.to_string());
    }
}

pub struct ProgressBar {
    pub max: usize,
    pub prefix: Option<String>
}

impl ProgressBar {
    pub fn new(max: usize) -> Self {
        ProgressBar {
            max: max,
            prefix: None
        }
    }

    pub fn set_prefix(&mut self, prefix: impl Into<String>) {
        self.prefix = Some(prefix.into());
    }

    pub fn update(&self, progress: usize) {
        let width = 32.0;

        let mut amt: f32 = 0.0;

        if progress != 0 {
            amt = progress as f32 / self.max as f32;
        }

        let fill = "=".repeat( (amt * width) as usize );
        let spaces = " ".repeat( (width - amt * width) as usize );

        print!("\r{} [{}{}] {}/{}", self.prefix.clone().unwrap_or("".to_string()), fill, spaces, progress, self.max);

        let flush_res = stdout().flush();

        if flush_res.is_err() {
            print_warn(format!("Failed to flush stdout: {}", flush_res.err().unwrap()));
        }
    }

    pub fn finish(&self) {
        println!();
    }
}

pub fn get_command_line_settings() -> CommandLineSettings {
    let conf = config::get();

    let mut args: Vec<String> = Vec::new();
    let mut flags: Vec<String> = Vec::new();

    let mut first = false;

    for i in std::env::args() {
        if !first {
            first = true;
            continue;
        }

        if i.starts_with("--") {
            flags.push(i.replace("--", ""));
        } else {
            args.push(i);
        }
    }

    let verbose_flag = (&flags).contains(&"--verbose".to_string());

    CommandLineSettings {
        args: args,
        flags: flags,
        verbose: conf.verbose_logging || verbose_flag
    }
}

pub fn flush() {
    stdout().flush().expect("Failed to flush stdout.");
}

pub fn confirm(message: impl Into<String>) -> bool {
    print!("{} [Y/N]: ", message.into());
    flush();
    
    let mut handle = stdin().lock();  
    let mut ch = [0_u8];

    handle.read_exact(&mut ch).expect("Failed to read stdin");

    String::from_utf8_lossy(&ch) == "y"
}

pub fn input(message: impl Into<String>) -> String {
    print!("{}", message.into());
    flush();

    let stdin = std::io::stdin();
    let mut res = String::new();

    stdin.lock().read_line(&mut res).expect("Failed to read stdin");
    
    res.trim().to_string()
}

pub fn input_non_empty(message: impl Into<String>) -> String {
    let msg: String = message.into();
    let res = input(msg.to_owned());

    if res.is_empty() {
        print_err("Value cannot be empty, try again");
        return input_non_empty(msg);
    }

    res
}

pub fn confirm_or_exit(message: &str) {
    if !confirm(message) {
        exit(1);
    }
}

pub fn print_err(message: impl Into<String>) {
    eprintln!("{} {}", Style::new().fg(Red).bold().paint("Error:"), message.into());
}

pub fn exit_err(message: impl Into<String>) -> ! {
    println!();
    print_err(message);
    exit(1);
}

pub fn print_verbose(settings: &CommandLineSettings, message: impl Into<String>) {
    if settings.verbose {
        println!("{}", message.into());
    }
}

pub fn print_step_verbose(settings: &CommandLineSettings, message: impl Into<String>) {
    if settings.verbose {
        print_step(message);
    }
}

pub fn print_success_verbose(settings: &CommandLineSettings, message: impl Into<String>) {
    if settings.verbose {
        print_success(message);
    }
}

pub fn print_warn(message: impl Into<String>) {
    println!("{} {}", Style::new().fg(Yellow).bold().paint("Warning:"), message.into());
}

pub fn print_success(message: impl Into<String>) {
    println!("{} {}", Style::new().fg(Green).bold().paint("OK:"), message.into())
}

pub fn print_note(message: impl Into<String>) {
    println!("{} {}", Style::new().fg(Purple).bold().paint("Note:"), message.into())
}

pub fn print_significant(prefix: impl Into<String>, message: impl Into<String>) {
    println!("{} {}", Style::new().fg(Cyan).bold().paint(format!("> {}:", prefix.into())), message.into())
}

pub fn get_step_prefix() -> String {
    Style::new().fg(Blue).bold().paint(">>").to_string()
}

pub fn print_step(message: impl Into<String>) {
    println!("{} {}", get_step_prefix(), message.into())
}
