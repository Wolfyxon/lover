mod command_input;
use std::{fs, path::Path, process::exit};

use command_input::{CommandLineSettings, get_command_line_settings};

mod project_config;
use project_config::ProjectConfig;

mod output;
use output::print_err;

mod actions;

struct Command {
    alias: String,
    description: String,
    function: fn()
}

fn main() {
    let cl_settings = get_command_line_settings();
    let alias_res = cl_settings.get_command_alias();

    if alias_res.is_none() {
        print_err("No command specified\n".to_string());
        show_help();
        return;
    }

    let alias = alias_res.unwrap();

    for command in get_commands() {
        if &command.alias == alias {
            (command.function)();
            return;
        }
    }

    print_err(format!("Unknown command: '{}'", &alias));
}

fn get_working_dir() -> String {
    std::env::current_dir().unwrap().to_str().unwrap().to_string()
}

fn get_commands() -> Vec<Command> {
    vec![
        Command {
            alias: "help".to_string(),
            description: "Shows help.".to_string(),
            function: cmd_help
        },
        Command {
            alias: "run".to_string(),
            description: "Runs the game.".to_string(),
            function: cmd_run
        },
        Command {
            alias: "parse".to_string(),
            description: "Checks the validity of Lua scripts..".to_string(),
            function: cmd_parse
        },
        Command {
            alias: "build".to_string(),
            description: "Packages the game.".to_string(),
            function: cmd_build
        },
        Command {
            alias: "clean".to_string(),
            description: "Removes compiled build files.".to_string(),
            function: cmd_clean
        },
        Command {
            alias: "new".to_string(),
            description: "Initializes a new Love2D project.".to_string(),
            function: cmd_new
        }
    ]
}

fn show_help() {
    println!("Lover");
}

fn cmd_help() {
    show_help();
}

fn cmd_run() {
    actions::execute("love", vec![project_config::get().source], false);
}

fn cmd_parse() {
    if !actions::command_exists(actions::PARSER) {
        print_err(format!("Cannot parse: '{}' not found.", actions::PARSER));
        exit(1);
    }

    actions::parse_all(Path::new(&project_config::get().source));
}

fn cmd_build() {
    todo!("Not implemented")
}

fn cmd_clean() {
    let path_str = project_config::get().build;
    let path = Path::new(&path_str);

    if !path.exists() {
        println!("Nothing to clean.");
        return;
    }

    if path.is_file() {
        print_err(format!("'{path_str}' is not a directory!"));
        exit(1);
    }

    let res = fs::remove_dir_all(path);
    if res.is_err() {
        print_err(format!("Failed to delete '{}': {}", path_str, res.err().unwrap()));
    }
}

fn cmd_new() {
    todo!("New");
}