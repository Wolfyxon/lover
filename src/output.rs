use ansi_term::Style;
use ansi_term::Color::{Red, Yellow};


pub fn print_err(message: String) {
    println!("{} {}", Style::new().fg(Red).bold().paint("Error:"), message);
}

pub fn print_warn(message: String) {
    println!("{} {}", Style::new().fg(Yellow).bold().paint("Warning:"), message);
}