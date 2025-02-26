use std::{path::Path, process::exit};
use ansi_term::Style;
use ansi_term::Color::{Blue, Yellow, Green};

mod console;
use console::{confirm_or_exit, exit_err, get_command_line_settings, print_err, print_significant, print_step, print_success, print_warn};
use deps::DependencyInstance;
use targets::get_targets;

mod project_config;

mod files;
mod project_maker;
mod actions;
mod http;
mod config;
mod deps;
mod targets;
mod lovebrew_bundler;
mod appimage;

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
    //short: Option<&'a str>,
    description: &'a str
}

impl<'a> CommandFlag<'a> {
    /*pub fn new(full: &'a str, short: &'a str, description: &'a str) -> Self {
        CommandFlag {
            full: full,
            short: Some(short),
            description
        }
    }*/

    pub fn new_only_full(full: &'a str, description: &'a str) -> Self {
        CommandFlag {
            full: full,
            //short: None,
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
    let commands = get_commands();

    for command in &commands {
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

    for command in &commands {
        if command.alias.starts_with(alias) || alias.starts_with(&command.alias) {
            println!("\nDid you mean `lover {} ...`? \n", command.alias);
            break;
        }
    }

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
            description: "Checks the validity of Lua scripts.".to_string(),
            function: cmd_parse,
            args: vec![],
            flags: vec![]
        },
        Command {
            alias: "build".to_string(),
            description: "Packages the game.".to_string(),
            function: cmd_build,
            args: vec![
                CommandArg::opt("targets...", "Names of the targets to build.")
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
            alias: "target".to_string(),
            description: "Lists or shows info of available build targets.".to_string(),
            function: cmd_target,
            args: vec![
                CommandArg::opt("target", "Name of the target to check.")
            ],
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
        },
        Command {
            alias: "env".to_string(),
            description: "Shows a list of available Lover constants and their values.".to_string(),
            function: cmd_env,
            args: vec![],
            flags: vec![]
        },
        Command {
            alias: "module".to_string(),
            description: "Generates the extra code injected into your game when building. Mostly for testing".to_string(),
            function: cmd_module,
            args: vec![],
            flags: vec![]
        }
    ]
}

fn show_help() {
    println!("Usage: lover <command> [<arguments>] \n");

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

        exit_err(format!("help: Unknown command '{}'", alias));
    } else {
        show_help();
    }
}

fn cmd_version(_command: &Command) {
    show_version();
}

fn cmd_run(_command: &Command) {
    let mut project_conf = project_config::get();
    let src = project_conf.directories.source;
    let cmd_settings = get_command_line_settings();
    
    print_significant("Running", src.clone());

    if !cmd_settings.has_flag("no-parse") {
        actions::parse_all(Path::new(&src));
    }

    let mut args = vec![project_config::get().directories.source];
    let mut run_args: &mut Vec<String> = &mut std::env::args().skip(2).into_iter().collect();

    if run_args.len() == 0 {
        run_args = &mut project_conf.run.default_args;
    }

    args.append(run_args);
    
    let config = &config::get();
    let env = actions::get_env_map(actions::Context::Run);

    if config.run.prime || cmd_settings.has_flag("prime") {
        actions::execute_prime_with_env(&config::get().software.love, args, env, false);
    } else { 
        actions::execute_with_env(&config::get().software.love, args, env, false);
    }
}

fn cmd_parse(_command: &Command) {
    let parser = config::get().software.luac;

    if !actions::command_exists(&parser) {
        exit_err(format!("Cannot parse: '{}' not found.", parser));
    }

    actions::parse_all(Path::new(&project_config::get().directories.source));
}

fn cmd_build(command: &Command) {
    let project_conf = project_config::get();
    let mut target_names = project_conf.build.default;

    let args = command.get_args();
    
    if args.len() != 0 {
        target_names = args;
    }

    let targets = targets::get_targets_by_strings(target_names.to_owned());
    let mut to_install: Vec<String> = Vec::new();

    print_significant("Initializing build of", target_names.join(", "));

    for target in &targets {
        for dep in target.get_all_deps() {
            if !dep.is_installed() && !to_install.contains(&dep.name.to_string()) {
                to_install.push(dep.name.to_string());
            }
        }
    }

    if to_install.len() != 0 {
        print_warn("Some dependencies are missing and must be installed.".to_string());
        deps::install(to_install);
    } else {
        print_success("All dependencies are installed.".to_string());
    }

    files::create_dir(&project_conf.directories.get_temp_dir());

    let mut already_built: Vec<&str> = Vec::new();

    for target in &targets {
        if already_built.contains(&target.name) { continue; }
        already_built.push(target.name);
        
        for dep_name in &target.previous {
            if already_built.contains(dep_name) { continue; }
            already_built.push(dep_name);

            let dep_target = targets::get_target_by_string(dep_name.to_string());
            dep_target.build();
        }

        target.build();
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
            exit_err(format!("Project name cannot contain: {}", &char_blacklist.join(" ")));
        }
    }
    
    project_maker::create(name.to_owned(), path);
}

fn cmd_target(command: &Command) {
    match command.get_arg("target") {
        Some(name) => {
            let target = targets::get_target_by_string(name);

            print_significant("Details of target", target.name.to_string());
            println!("{}\n", Style::new().italic().paint(target.description));

            print_step("Previous targets:".to_string());
            for prev in target.previous {
                println!("- {}", prev);
            }

            println!();

            print_step("Dependencies:".to_string());
            for name in target.deps {
                let dep = deps::get_dep(name);
                
                let mut suffix = "";
                let mut style = Style::new();

                if dep.is_installed() {
                    suffix = "(installed)";
                    style = style.fg(Green);
                }

                println!("- {} {}", style.paint(dep.name), suffix);
            }
        },
        None => {
            print_significant("Available build targets", "\n".to_string());

            for target in get_targets() {
                println!("- {}: {}", Style::new().fg(Green).paint(target.name), target.description)
            }

            println!();
            println!("Use `lover build [name]` to build a target");
            println!("You can also use `lover build [target1] [target2] ...`");
        }
    }
}

fn cmd_dep(command: &Command) {
    match command.get_arg("dependency") {
        Some(name) => {
            let dep = deps::get_dep(name.as_str());
            let mut status = "not installed";
    
            if dep.is_installed() {
                status = "installed";
            }
    
            print_significant("Details of", dep.name.to_string());
    
            println!("{}\n", Style::new().italic().paint(dep.description));
    
            println!("Status: {}", status);
            println!("Location: {}", dep.get_path().to_str().unwrap());
            println!("Repository: {}", dep.get_repo_url());
    
            println!();
            print_step("Actions:".to_string());
    
            println!("`lover install {}` to install or update.", dep.name);
            println!("`lover uninstall {}` to remove.", dep.name);
        },
        None => {
            print_significant("Available dependencies", "\n".to_string());

            let installed_style = Style::new().fg(Green);
    
            for dep in deps::get_deps() {
                let mut styled_name = dep.name.to_string();
                let mut suffix = "";
    
                if dep.is_installed() {
                    styled_name = installed_style.paint(styled_name).to_string();
                    suffix = "(installed)";
                }
    
                println!("- {} {}: {}", styled_name, suffix, dep.description);
            }
            
            println!("\nDependencies are located in: {}\n", deps::get_dir().to_str().unwrap());

            println!("`lover install <name>` to install.");
            println!("`lover uninstall <name>` to remove.");
        }
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
        exit_err("None of the specified packages are installed.".to_string());
    }

    print_step("The following dependencies will be removed:".to_string());

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

    match dep.get_instance() {
        DependencyInstance::LatestRelease(d) => {
            print_step("Release data".to_string());
            let release = d.fetch_release();
        
            println!("Name: {}", release.name);
            println!("Tag (version): {}", release.tag_name);
            println!("Page: {}", release.html_url);
            
            print_step("Asset data".to_string());
            let asset = d.get_asset_from_release(&release);
        
            println!("Name: {}", asset.name);
            println!("Download URL: {}", asset.browser_download_url);
        },
        DependencyInstance::Source(d) => {
            print_step("Source data".to_string());
            println!("Branch: {}", d.branch);
        }
    }
}

fn cmd_env(_command: &Command) {
    for (k, v) in actions::get_env_map(actions::Context::Run) {
        println!("{}: {}", k, v);
    }

    println!("\nUse `os.getenv(\"name\")` in your game to access each value.");
}

fn cmd_module(_command: &Command) {
    println!("{}", targets::gen_module());
}