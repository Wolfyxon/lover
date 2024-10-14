use std::{path::Path, process::exit};
use ansi_term::Style;
use ansi_term::Color::{Blue, Yellow, Green};

mod console;
use console::{get_command_line_settings, print_err, print_success, print_significant};

mod project_config;

mod files;
mod project_maker;
mod actions;

mod http;
mod config;

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

    pub fn get_string_usage(&self) -> String {
        let mut res = String::new();

        for arg in &self.args {
            let mut l = "[";
            let mut r = "]";

            if arg.required {
                 l = "<";
                 r = ">";
            }

            res += format!("{}{}{} ", l, arg.name, r).as_str();
        }

        res
    }

    pub fn get_required_arg_amount(&self) -> usize {
        let mut res = 0;

        for arg in &self.args {
            if arg.required {
                res += 1;
            }
        }

        res
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
        
        if cl_settings.has_flag("version") {
            show_version();
            return;
        }

        if !cl_settings.has_flag("help") {
            print_err("No command specified\n".to_string());
        }

        show_help();
        return;
    }

    let alias = alias_res.unwrap();

    for command in get_commands() {
        if &command.alias == alias {

            if cl_settings.args.len() - 1 < command.get_required_arg_amount() {
                print_err(format!("Not enough arguments for '{}'\n", alias));
                
                println!("Usage: {} {}", alias, command.get_string_usage());
                exit(1);
            }

            (command.function)(&command);
            return;
        }
    }

    print_err(format!("Unknown command: '{}'", &alias));
    println!("Use `lover help` to see available commands");

    exit(1);
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
            alias: "version".to_string(),
            description: "Shows the current Lover version.".to_string(),
            function: cmd_version,
            args: vec![],
            flags: vec![]
        },
        Command {
            alias: "run".to_string(),
            description: "Runs the game.".to_string(),
            function: cmd_run,
            args: vec![],
            flags: vec![
                CommandFlag::new_only_full("no-parse", "Skips the parsing process"),
                CommandFlag::new_only_full("prime", "Runs the game on the dedicated GPU")
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
                CommandArg::req("name", "Name of your new project.")
            ],
            flags: vec![]
        }
    ]
}

fn show_help() {
    println!("Available commands: \n");

    for cmd in get_commands() {
        let colored_alias = Style::new().bold().fg(Blue).paint(cmd.alias);
        println!("  {}: {}", colored_alias, cmd.description);
    }
    println!("\nUse `lover help <command>` to see the usage of a specific command.");
    println!("For additional help see the wiki: https://github.com/Wolfyxon/lover/wiki");
}

fn show_version() {
    println!("Lover {}", env!("CARGO_PKG_VERSION"));
}

fn cmd_help(cmd: &Command) {
    let alias_res = cmd.get_arg("command");

    if alias_res.is_some() {
        let alias = alias_res.unwrap();

        for command in get_commands() {
            if command.alias == alias {
                print_significant("Command", alias.to_owned());

                println!("{}\n", Style::new().italic().paint(&command.description));

                let styled_alias = Style::new().fg(Blue).paint(alias);
                println!("Usage:");
                println!("  {} {}", styled_alias, command.get_string_usage());

                println!("\nArguments:");

                for arg in command.args {
                    let mut name_style = Style::new();

                    if arg.required {
                        name_style = name_style.fg(Yellow);
                    } else {
                        name_style = name_style.fg(Green);
                    }

                    println!("  {}: {}", name_style.paint(arg.name), arg.description);
                }

                println!("\nFlags:");

                for flag in command.flags {
                    println!("  --{}: {}", flag.full, flag.description);
                }
                
                return;
            }
        }

        print_err(format!("help: Unknown command '{}'", alias));
        exit(1);

    } else {
        show_help();
    }
}

fn cmd_version(cmd: &Command) {
    show_version();
}

fn cmd_run(cmd: &Command) {
    let src = project_config::get().directories.source;
    let cmd_settings = get_command_line_settings();

    print_significant("Running", src.clone());

    if !cmd_settings.has_flag("no-parse") {
        actions::parse_all(Path::new(&src));
    }

    let mut args = vec![project_config::get().directories.source];
    args.append(&mut std::env::args().skip(2).into_iter().collect());
    
    let config = &config::get();

    if config.run.prime || cmd_settings.has_flag("prime") {
        actions::execute_prime(&config::get().software.love, args, false);
    } else { 
        actions::execute(&config::get().software.love, args, false);
    }
}

fn cmd_parse(cmd: &Command) {
    let parser = config::get().software.luac;

    if !actions::command_exists(&parser) {
        print_err(format!("Cannot parse: '{}' not found.", parser));
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
    let char_blacklist = vec!["..", "/", "\\", "\""];
    let name = cmd.get_arg("name").unwrap();
    let path = Path::new(&name);

    for blacklisted in &char_blacklist {
        if name.contains(blacklisted) {
            print_err(format!("Project name cannot contain: {}", &char_blacklist.join(" ")));
            exit(1);
        }
    }
    
    project_maker::create(name.to_owned(), path);
}