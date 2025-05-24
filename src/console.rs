use std::io::{stdin, stdout, BufRead, Read, Write};
use std::process::exit;
use ansi_term::Style;
use ansi_term::Color::{Red, Yellow, Green, Cyan, Blue, Purple};
use termsize::Size;

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
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub convert: fn(unit: f32) -> f32
}

impl ProgressBar {
    pub fn new(max: usize) -> Self {
        ProgressBar {
            max: max,
            prefix: None,
            suffix: None,
            convert: |unit| unit
        }
    }

    pub fn memory_mode(&mut self) -> &mut Self {
        const KB: f32 = 1024.0;
        const MB: f32 = KB * 1024.0;
        const GB: f32 = MB * 1024.0;

        let max = self.max as f32;

        if max > GB {
            self.set_converter(|u| u / GB);
            self.set_suffix("GB");
        } else if max > MB {
            self.set_converter(|u| u / MB);
            self.set_suffix("MB");
        } else if max > KB {
            self.set_converter(|u| u / KB);
            self.set_suffix("KB");
        } else {
            self.set_suffix("B");
        }

        self
    }

    pub fn set_converter(&mut self, func: fn(unit: f32) -> f32) -> &mut Self {
        self.convert = func;
        self
    }

    pub fn set_prefix(&mut self, prefix: impl Into<String>) -> &mut Self {
        self.prefix = Some(prefix.into());
        self
    }

    pub fn set_suffix(&mut self, suffix: impl Into<String>) -> &mut Self {
        self.suffix = Some(suffix.into());
        self
    }

    pub fn update(&self, progress: usize) {
        let term_width = termsize::get().unwrap_or(Size {rows: 1, cols: 200}).cols;

        let mut bar_margin: usize = (term_width as f32 * 0.3).max(42.0) as usize;
        let width = (term_width as f32 * 0.2).min(30.0);

        let mut amt: f32 = 0.0;

        if progress != 0 {
            amt = progress as f32 / self.max as f32;
        }

        let prefix_len = match &self.prefix {
            Some(prefix) => strip_ansi_escapes::strip(prefix).len(),
            None => {
                bar_margin = 0;
                0
            }
        };

        let pre_space_size = bar_margin.saturating_sub(prefix_len);

        let fill = "=".repeat( (amt * width) as usize );
        let spaces = " ".repeat( (width - amt * width).ceil() as usize );
        let pre_space = " ".repeat(pre_space_size);
        let prefix = self.prefix.clone().unwrap_or("".to_string());
        let suffix = self.suffix.clone().unwrap_or("".to_string());

        let zero_factor = 1000.0; 
        let disp_progress = ((self.convert)(progress as f32) * zero_factor).round() / zero_factor;
        let disp_max = ((self.convert)(self.max as f32) * zero_factor).round() / zero_factor;
        
        let bar_string = format!("{prefix} {pre_space} [{fill}{spaces}] {disp_progress}/{disp_max} {suffix}");
        let clear_space = " ".repeat(term_width.saturating_sub(bar_string.len() as u16) as usize);

        print!("\r{}{}", bar_string, clear_space);
        flush();
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

    let verbose_flag = (&flags).contains(&"verbose".to_string());

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

    String::from_utf8_lossy(&ch).to_lowercase() == "y"
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
