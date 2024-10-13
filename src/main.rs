use std::{env, fs, path::Path, process::exit};

mod console;
use console::{get_command_line_settings, print_err, print_success, print_significant, CommandLineSettings};

mod project_config;
use project_config::ProjectConfig;

mod files;
mod actions;

struct Command<'a> {
    alias: String,
    description: String,
    function: fn(&Command),
    args: Vec<CommandArg<'a>>,
    flags: Vec<CommandFlag<'a>>,
}

impl<'a> Command<'a> {
    pub fn get_arg(&self, name: &str) -> Option<String> {

        for i in 0..self.args.len() {
            let arg = &self.args[i];
            
            if arg.name == name {
                let stgs = get_command_line_settings();
                let res = stgs.args.get(i + 1);
                
                if res.is_none() {
                    return None;
                }

                return Some(res.unwrap().clone());
            }
        }

        None
    }
}

struct CommandArg<'a> {
    name: &'a str,
    description: &'a str,
    required: bool
}

impl<'a> CommandArg<'a> {
    pub fn opt(name: &'a str, description: &'a str) -> Self {
        CommandArg { 
            name: name, 
            description: description, 
            required: false 
        }
    }

    pub fn req(name: &'a str, description: &'a str) -> Self {
        CommandArg { 
            name: name, 
            description: description, 
            required: true 
        }
    }
}

struct CommandFlag<'a> {
    full: &'a str,
    short: Option<&'a str>,
    description: &'a str
}

impl<'a> CommandFlag<'a> {
    pub fn new(full: &'a str, short: &'a str, description: &'a str) -> Self {
        CommandFlag {
            full: full,
            short: Some(short),
            description
        }
    }

    pub fn new_only_full(full: &'a str, description: &'a str) -> Self {
        CommandFlag {
            full: full,
            short: None,
            description
        }
    }
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
            (command.function)(&command);
            return;
        }
    }

    print_err(format!("Unknown command: '{}'", &alias));
}

fn get_working_dir() -> String {
    std::env::current_dir().unwrap().to_str().unwrap().to_string()
}

fn get_commands<'a>() -> Vec<Command<'a>> {
    vec![
        Command {
            alias: "help".to_string(),
            description: "Shows help.".to_string(),
            function: cmd_help,
            args: vec![
                CommandArg::opt("command", "Command to check the usage of")
            ],
            flags: vec![]
        },
        Command {
            alias: "run".to_string(),
            description: "Runs the game.".to_string(),
            function: cmd_run,
            args: vec![],
            flags: vec![
                CommandFlag::new_only_full("no-parse", "Skips the parsing process")
            ]
        },
        Command {
            alias: "parse".to_string(),
            description: "Checks the validity of Lua scripts..".to_string(),
            function: cmd_parse,
            args: vec![],
            flags: vec![]
        },
        Command {
            alias: "build".to_string(),
            description: "Packages the game.".to_string(),
            function: cmd_build,
            args: vec![
                CommandArg::opt("target", "Build target.")
            ],
            flags: vec![]
        },
        Command {
            alias: "clean".to_string(),
            description: "Removes compiled build files.".to_string(),
            function: cmd_clean,
            args: vec![],
            flags: vec![]
        },
        Command {
            alias: "new".to_string(),
            description: "Initializes a new Love2D project.".to_string(),
            function: cmd_new,
            args: vec![
                CommandArg::opt("name", "Name of your new project.")
            ],
            flags: vec![]
        }
    ]
}

fn show_help() {
    println!("Lover");
}

fn cmd_help(cmd: &Command) {
    show_help();
}

fn cmd_run(cmd: &Command) {
    let src = project_config::get().directories.source;

    print_significant("Running", src.clone());

    if !get_command_line_settings().has_flag("no-parse") {
        actions::parse_all(Path::new(&src));
    }

    let mut args = vec![project_config::get().directories.source];
    args.append(&mut std::env::args().skip(2).into_iter().collect());
    
    actions::execute("love", args, false);
}

fn cmd_parse(cmd: &Command) {
    if !actions::command_exists(actions::PARSER) {
        print_err(format!("Cannot parse: '{}' not found.", actions::PARSER));
        exit(1);
    }

    actions::parse_all(Path::new(&project_config::get().directories.source));
}

fn cmd_build(cmd: &Command) {
    let mut target = "love".to_string();

    // TODO: All targets from lover.toml
    // TODO: Platform detection and building for that platform

    let arg_target_res = cmd.get_arg("target");

    if arg_target_res.is_some() {
        target = arg_target_res.unwrap();
    }

    print_significant("Building target", target.to_string());   

    match target.as_str() {
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

fn cmd_clean(cmd: &Command) {
    let build = &project_config::get().directories.build;

    print_significant("Removing", build.to_string());
    actions::clean(Path::new(build));
}

fn cmd_new(cmd: &Command) {
    todo!("New");
}