use ansi_term::Style;
use ansi_term::Color::{Red, Yellow, Green};

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

pub fn print_err(message: String) {
    println!("{} {}", Style::new().fg(Red).bold().paint("Error:"), message);
}

pub fn print_warn(message: String) {
    println!("{} {}", Style::new().fg(Yellow).bold().paint("Warning:"), message);
}

pub fn print_success(message: String) {
    println!("{} {}", Style::new().fg(Green).bold().paint("Ok:"), message)
}