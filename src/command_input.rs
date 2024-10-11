pub struct CommandLineSettings {
    args: Vec<String>,
    flags: Vec<String>
}

impl CommandLineSettings {
    pub fn get_command_alias(&self) -> Option<&String> {
        return self.args.get(0);
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