use std::io::{stdin, stdout, Read, Write};
use std::process::exit;
use ansi_term::Style;
use ansi_term::Color::{Red, Yellow, Green, Cyan, Blue};

pub struct CommandLineSettings {
    pub args: Vec<String>,
    pub flags: Vec<String>
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
    pub max: usize
}

impl ProgressBar {
    pub fn new(max: usize) -> Self {
        ProgressBar {
            max: max
        }
    }

    pub fn update(&self, progress: usize) {
        let width = 32.0;

        let mut amt: f32 = 0.0;

        if progress != 0 {
            amt = progress as f32 / self.max as f32;
        }

        let fill = "=".repeat( (amt * width) as usize );
        let spaces = " ".repeat( (width - amt * width) as usize );

        print!("\r[{}{}] {}/{}", fill, spaces, progress, self.max);
    }

    pub fn finish(&self) {
        println!();
    }
}

pub fn get_command_line_settings() -> CommandLineSettings {
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

    CommandLineSettings {
        args: args,
        flags: flags
    }
}

pub fn confirm(message: &str) -> bool {
    print!("{} [Y/N]: ", message);

    stdout().flush().expect("Failed to flush stdout.");

    let mut handle = stdin().lock();  
    let mut ch = [0_u8];

    handle.read_exact(&mut ch).expect("Failed to read stdin");

    String::from_utf8_lossy(&ch) == "y"
}

pub fn confirm_or_exit(message: &str) {
    if !confirm(message) {
        println!("Cancelled");
        exit(1);
    }
}

pub fn print_err(message: String) {
    println!("{} {}", Style::new().fg(Red).bold().paint("Error:"), message);
}

pub fn print_warn(message: String) {
    println!("{} {}", Style::new().fg(Yellow).bold().paint("Warning:"), message);
}

pub fn print_success(message: String) {
    println!("{} {}", Style::new().fg(Green).bold().paint("OK:"), message)
}

pub fn print_significant(prefix: &str, message: String) {
    println!("{} {}", Style::new().fg(Cyan).bold().paint(format!("> {}:", prefix)), message)
}

pub fn print_stage(message: String) {
    let prefix = Style::new().fg(Blue).bold().paint(">>");
    let msg = Style::new().fg(Blue).paint(message);
    println!("{} {}", prefix, msg)
}