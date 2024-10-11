mod command_input;
use command_input::{CommandLineSettings, get_command_line_settings};

mod project_config;
use project_config::ProjectConfig;

mod output;
use output::print_err;

mod actions;

struct CommandContext<'a> {
    cl_settings: &'a CommandLineSettings
}

struct Command {
    alias: String,
    description: String,
    function: fn(context: CommandContext)
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
            let ctx = CommandContext {
                cl_settings: &cl_settings
            };

            (command.function)(ctx);
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
            alias: "build".to_string(),
            description: "Packages the game.".to_string(),
            function: cmd_build
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

fn cmd_help(ctx: CommandContext) {
    show_help();
}

fn cmd_run(ctx: CommandContext) {
    actions::execute("love", vec![project_config::get().source]);
}

fn cmd_build(ctx: CommandContext) {
    todo!("Not implemented")
}

fn cmd_new(ctx: CommandContext) {
    todo!("New");
}