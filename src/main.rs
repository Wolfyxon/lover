use std::{path::Path, process::exit};
use ansi_term::Style;
use ansi_term::Color::{Blue, Yellow, Green};

mod console;
use console::{confirm_or_exit, get_command_line_settings, print_err, print_significant, print_stage, print_success, print_warn};

mod project_config;

mod files;
mod project_maker;
mod actions;

mod http;
mod config;
mod deps;

struct Command<'a> {
    alias: String,
    description: String,
    function: fn(&Command),
    args: Vec<CommandArg<'a>>,
    flags: Vec<CommandFlag<'a>>,
}

impl<'a> Command<'a> {
    pub fn get_args(&self) -> Vec<String> {
        get_command_line_settings().args.iter().cloned().skip(1).collect()
    }

    pub fn get_arg(&self, name: &str) -> Option<String> {
        let args = self.get_args();

        for i in 0..self.args.len() {
            let arg = &self.args[i];
            
            if arg.name == name {
                let res = args.get(i);
                
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
            alias: "new".to_string(),
            description: "Initializes a new Love2D project.".to_string(),
            function: cmd_new,
            args: vec![
                CommandArg::req("name", "Name of your new project.")
            ],
            flags: vec![]
        },
        Command {
            alias: "run".to_string(),
            description: "Runs the game.".to_string(),
            function: cmd_run,
            args: vec![
                CommandArg::opt("arguments...", "Arguments handled by your game.")
            ],
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
            alias: "dep".to_string(),
            description: "Lists or shows info of available dependencies.".to_string(),
            function: cmd_dep,
            args: vec![
                CommandArg::opt("dependency", "Name of the dependency to check")
            ],
            flags: vec![]
        },
        Command {
            alias: "install".to_string(),
            description: "Installs dependencies.".to_string(),
            function: cmd_install,
            args: vec![
                CommandArg::req("dependencies...", "Names of the dependencies to install.")
            ],
            flags: vec![]
        },
        Command {
            alias: "uninstall".to_string(),
            description: "Removes installed dependencies.".to_string(),
            function: cmd_uninstall,
            args: vec![
                CommandArg::req("dependencies...", "Names of the dependencies to remove.")
            ],
            flags: vec![]
        },
        Command {
            alias: "fetch".to_string(),
            description: "Fetches a dependency. Mostly for testing".to_string(),
            function: cmd_fetch,
            args: vec![
                CommandArg::req("name", "Name of the dependency.")
            ],
            flags: vec![]
        }
    ]
}

fn show_help() {
    println!("Lover is a open source cross-platform build system for Love2D projects.");
    println!("https://github.com/Wolfyxon/lover \n");

    println!("Available commands: \n");

    for cmd in get_commands() {
        let colored_alias = Style::new().bold().fg(Blue).paint(cmd.alias);
        println!("  {}: {}", colored_alias, cmd.description);
    }
    println!("\nUse `lover help <command>` to see the usage of a specific command.");
    println!("For additional help, see the wiki: https://github.com/Wolfyxon/lover/wiki");
}

fn show_version() {
    println!("Lover {}", env!("CARGO_PKG_VERSION"));
}

fn cmd_help(command: &Command) {
    let alias_res = command.get_arg("command");

    if alias_res.is_some() {
        let alias = alias_res.unwrap();

        for cmd in get_commands() {
            if cmd.alias == alias {
                print_significant("Command", alias.to_owned());

                println!("{}\n", Style::new().italic().paint(&cmd.description));

                let styled_alias = Style::new().fg(Blue).paint(alias);
                println!("Usage:");
                println!("  {} {}", styled_alias, cmd.get_string_usage());

                if !cmd.args.is_empty() {
                    println!("\nArguments:");

                    for arg in cmd.args {
                        let mut name_style = Style::new();
    
                        if arg.required {
                            name_style = name_style.fg(Yellow);
                        } else {
                            name_style = name_style.fg(Green);
                        }
    
                        println!("  {}: {}", name_style.paint(arg.name), arg.description);
                    }
                }

                if !cmd.flags.is_empty() {
                    println!("\nFlags:");

                    for flag in cmd.flags {
                        println!("  --{}: {}", flag.full, flag.description);
                    }    
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

fn cmd_version(_command: &Command) {
    show_version();
}

fn cmd_run(_command: &Command) {
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

fn cmd_parse(_command: &Command) {
    let parser = config::get().software.luac;

    if !actions::command_exists(&parser) {
        print_err(format!("Cannot parse: '{}' not found.", parser));
        exit(1);
    }

    actions::parse_all(Path::new(&project_config::get().directories.source));
}

fn cmd_build(command: &Command) {
    let mut target = "love".to_string();

    // TODO: All targets from lover.toml
    // TODO: Platform detection and building for that platform

    let arg_target_res = command.get_arg("target");

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

fn cmd_clean(_command: &Command) {
    let build = &project_config::get().directories.build;

    print_significant("Removing", build.to_string());
    actions::clean(Path::new(build));
}

fn cmd_new(command: &Command) {
    let char_blacklist = vec!["..", "/", "\\", "\""];
    let name = command.get_arg("name").unwrap();
    let path = Path::new(&name);

    for blacklisted in &char_blacklist {
        if name.contains(blacklisted) {
            print_err(format!("Project name cannot contain: {}", &char_blacklist.join(" ")));
            exit(1);
        }
    }
    
    project_maker::create(name.to_owned(), path);
}

fn cmd_dep(command: &Command) {
    let name_res = command.get_arg("dependency");
    
    if name_res.is_some() {
        let name = name_res.unwrap();
        let dep = deps::get_dep(name.as_str());
        let mut status = "not installed";

        if dep.is_installed() {
            status = "installed";
        }

        print_significant("Details of", dep.name.to_string());

        println!("Description: \n  {}\n", Style::new().italic().paint(dep.description));

        println!("Status: {}", status);
        println!("Location: {}", dep.get_path().to_str().unwrap());
        println!("Repository: https://github.com/{}/{}", dep.repo_owner, dep.repo);

        println!();
        print_stage("Actions:".to_string());

        println!("`lover install {}` to install or update.", dep.name);
        println!("`lover uninstall {}` to remove.", dep.name);
        
    } else {
        print_significant("Available dependencies", "\n".to_string());

        let installed_style = Style::new().fg(Green);

        for dep in deps::get_deps() {
            let mut styled_name = dep.name.to_string();
            let mut suffix = "";

            if dep.is_installed() {
                styled_name = installed_style.paint(styled_name).to_string();
                suffix = "(installed)";
            }

            println!("  {} {}", styled_name, suffix);
        }

        println!();

        println!("`lover install <name>` to install.");
        println!("`lover uninstall <name>` to remove.");
    }
}

fn cmd_install(command: &Command) {
    deps::install(command.get_args());
}

fn cmd_uninstall(command: &Command) {
    let dependencies = deps::get_deps_by_strings(command.get_args());
    let mut amt = 0;

    for dep in &dependencies {
        if dep.is_installed() {
            amt += 1;
        } else {
            print_warn(format!("'{}' is not installed, ignoring.", dep.name));
        }
    }

    if amt == 0 {
        print_err("None of the specified packages are installed.".to_string());
        exit(1);
    }

    print_stage("The following dependencies will be removed:".to_string());

    for dep in &dependencies {
        if dep.is_installed() {
            println!("  {}", dep.name);
        }
    }

    confirm_or_exit("\nProceed with the removal?");

    let mut fail = false;

    for dep in &dependencies {
        if dep.is_installed() {
            let path = dep.get_path();
            let res = std::fs::remove_file(&path);

            if res.is_err() {
                print_err(format!("Failed to delete '{}': {}", &path.to_str().unwrap(), res.err().unwrap()));
                fail = true;
            }
        }
    }

    if fail {
        exit(1);
    } else {
        print_success("Removed successfully.".to_string());
    }
}

fn cmd_fetch(command: &Command) {
    let name = command.get_arg("name").unwrap();
    let dep = deps::get_dep(&name);

    print_significant("Data of dependency", name.to_owned());
    
    print_stage("Release data".to_string());
    let release = dep.fetch_release();

    println!("Name: {}", release.name);
    println!("Tag (version): {}", release.tag_name);
    println!("Page: {}", release.html_url);
    
    print_stage("Asset data".to_string());
    let asset = dep.get_asset_from_release(&release);

    println!("Name: {}", asset.name);
    println!("Download URL: {}", asset.browser_download_url);
}