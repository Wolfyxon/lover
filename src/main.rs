use std::{fs, path::Path, process::exit};

mod console;
use console::{get_command_line_settings, print_err, print_success, print_significant, CommandLineSettings};

mod project_config;
use project_config::ProjectConfig;

mod files;
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
    
    if !get_command_line_settings().has_flag("no-parse") {
        actions::parse_all(Path::new(&project_config::get().directories.source));
    }

    actions::execute("love", vec![project_config::get().directories.source], false);
}

fn cmd_parse() {
    if !actions::command_exists(actions::PARSER) {
        print_err(format!("Cannot parse: '{}' not found.", actions::PARSER));
        exit(1);
    }

    actions::parse_all(Path::new(&project_config::get().directories.source));
}

fn cmd_build() {
    let mut target = "love";

    // TODO: All targets from lover.toml
    // TODO: Platform detection and building for that platform

    let cmd = get_command_line_settings();
    let arg_target_res = cmd.args.get(1);

    if arg_target_res.is_some() {
        target = arg_target_res.unwrap();
    }

    print_significant("Building target", target.to_string());   

    match target {
        "love" => {
            let config = project_config::get();
            let output = Path::new(config.directories.build.as_str()).join(config.package.name + ".love");
        
            actions::parse_all(Path::new(&project_config::get().directories.source));
            actions::archive(Path::new(config.directories.source.as_str()), &output);
        
            print_success(format!("Successfully built: {}", output.to_str().unwrap()));
        }

        _ => {
            print_err(format!("Unknown target '{}'", target));
            exit(1);
        }
    } 
}

fn cmd_clean() {
    actions::clean(Path::new(&project_config::get().directories.build));
}

fn cmd_new() {
    todo!("New");
}