use std::collections::HashMap;
use std::env::split_paths;
use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::process::Command;
use std::process::Stdio;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use ansi_term::Style;
use ansi_term::Color::Blue;
use zip::write::SimpleFileOptions;
use zip::ZipArchive;

use crate::config;
use crate::console;
use crate::console::exit_err;
use crate::console::get_command_line_settings;
use crate::console::print_step_verbose;
use crate::console::print_success_verbose;
use crate::console::ProgressBar;
use crate::console::{print_warn, print_success, print_step};
use crate::files;
use crate::files::get_file_tree;
use crate::project_config;

pub enum Context {
    Run,
    Build
}

pub struct Archiver {
    dir: PathBuf,
    progress_bar: Option<ProgressBar>,
    ignored_files: Vec<PathBuf>
}

impl Archiver {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self {
            dir: dir.into(),
            progress_bar: None,
            ignored_files: Vec::new()
        }
    }

    pub fn add_progress_bar(&mut self, prefix: impl Into<String>) -> &mut Self {
        let mut bar = ProgressBar::new(1);
        bar.set_prefix(format!("{} {}", console::get_step_prefix(), prefix.into()));
        
        self.progress_bar = Some(bar);
        self
    }

    pub fn ignore_file(&mut self, file: impl Into<PathBuf>) -> &mut Self {
        self.ignored_files.push(file.into());
        
        self
    }

    pub fn archive(&mut self, output: impl Into<PathBuf>) {
        let output_dir = output.into();
        files::create_dir(&output_dir.parent().unwrap());

        let output_file = files::create(&output_dir);
        let tree: Vec<PathBuf> = get_file_tree(&self.dir);
        let options = SimpleFileOptions::default();
        let mut zip = zip::ZipWriter::new(output_file);
        let mut buffer: Vec<u8> = Vec::new();
        let mut progress: usize = 0;

        self.progress_bar.as_mut().map(|bar| {
            bar.max = tree.len();
        });
    
        print_step_verbose(
            &get_command_line_settings(), 
            format!("Archiving '{}' into '{}'...", &self.dir.to_str().unwrap(), output_dir.to_str().unwrap())
        );
        
        for path in tree {
            let mut ignore = false;
    
            for ignored_path in &self.ignored_files {
                if path.as_path() == &self.dir.join(ignored_path) {
                    ignore = true;
                    break;
                }
            }
    
            if ignore {
                continue
            }
    
            let out_path = PathBuf::from_iter(path.components().skip(self.dir.components().count()));
            let mut file = File::open(path).unwrap();
                
            file.read_to_end(&mut buffer).unwrap();
            zip.start_file_from_path(out_path, options).unwrap();
            zip.write_all(&buffer).unwrap();
    
            buffer.clear();
    
            progress += 1;
            
            self.progress_bar.as_mut().map(|bar| {
                bar.update(progress);
            });
        }
    
        self.progress_bar.as_mut().map(|bar| {
            bar.finish();
        });
    }
}

pub struct Extractor {
    source: PathBuf,
    progress_bar: Option<ProgressBar>
}

impl Extractor {
    pub fn new(source: impl Into<PathBuf>) -> Self {
        Self {
            source: source.into(),
            progress_bar: None
        }
    }

    pub fn add_progress_bar(&mut self, prefix: impl Into<String>) -> &mut Self {
        let mut bar = ProgressBar::new(1);
        bar.set_prefix(format!("{} {}", console::get_step_prefix(), prefix.into()));
        
        self.progress_bar = Some(bar);
        self
    }

    pub fn extract(&mut self, output: impl Into<PathBuf>) {
        let output_dir = output.into();

        files::create_dir(&output_dir);

        let zip_file = files::open(&self.source);
        let source_str = &self.source.to_str().unwrap();
        let dir_str = &output_dir.to_str().unwrap();
    
        let mut archive = ZipArchive::new(zip_file).unwrap_or_else(|err| {
            exit_err(format!("ZIP failed for '{}': {}", source_str, err))
        });
    
        let archive_len = archive.len();
    
        print_step_verbose(
            &get_command_line_settings(),
            format!("Extracting '{}' to '{}'...", source_str, dir_str)
        );
        
        self.progress_bar.as_mut().map(|bar| {
            bar.max = archive_len;
            bar.update(0);
        });
    
        for i in 0..archive_len {
            let mut file = archive.by_index(i).unwrap_or_else(|err| {
                exit_err(format!("Failed to get file at index {} of '{}': {}", i, source_str, err));
            });
    
            let path = match file.enclosed_name() {
                Some(path) => {
                    PathBuf::from(path.file_name().unwrap())
    
                    // Enable if nested files are needed.
                    /*
                    let mut res = path.to_owned();
                    let components = path.components();
                    let cmp_len = components.to_owned().count();
    
                    if cmp_len > 1 {
                        let mut new_path = PathBuf::new();
                        let mut skipped = false;
    
                        for cmp in components {
                            if !skipped {
                                skipped = true;
                                continue;
                            }
    
                            new_path = new_path.join(cmp);
                        }
    
                        res = new_path;
                    }
    
                    res*/
                },
                None => exit_err(format!("Failed to get path of file at index {} of '{}'", i, source_str))
            };
    
            let mut out_file = files::create(output_dir.join(path).as_path());
            
            loop {
                let mut buf: [u8; 1024] = [0; 1024];
                
                let bytes_read = file.read(&mut buf).unwrap_or_else(|err| {
                    exit_err(format!("Read failed: {}", err))
                });
        
                if bytes_read == 0 { break; }
        
                out_file.write_all(&buf[..bytes_read]).unwrap_or_else(|err| {
                    exit_err(format!("Write failed: {}", err));
                });
            }
    
            self.progress_bar.as_mut().map(|bar| {
                bar.update(i + 1);
            });
        }
    
        self.progress_bar.as_mut().map(|bar| {
            bar.finish();
        });
    }
}

#[derive(Clone)]
pub struct CommandRunner {
    command: String,
    args: Vec<String>,
    env: HashMap<String, String>,
    paths: Vec<PathBuf>,
    quiet: bool,
    ignore_os_path: bool
}

impl CommandRunner {
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            args: Vec::new(),
            env: HashMap::new(),
            paths: Vec::new(),
            quiet: false,
            ignore_os_path: false
        }
    }

    pub fn add_path(&mut self, path: impl Into<PathBuf>) -> &mut Self {
        self.paths.push(path.into());
        self
    }
    
    pub fn get_all_paths(&self) -> Vec<PathBuf> {
        let mut res: Vec<PathBuf> = Vec::new();

        res.append(&mut self.paths.clone());

        if !&self.ignore_os_path {
            match std::env::var_os("PATH") {
                Some(paths) => {
                    for path in split_paths(&paths) {
                        let file_path = path.join(&self.command);
                        res.push(file_path);
                    }
                },
                None => ()
            }
        }

        res
    }

    pub fn get_path(&self) -> Option<PathBuf> {
        for path in self.get_all_paths() {
            if path.is_file() {
                return Some(path);
            }
        }

        None
    }

    pub fn add_args(&mut self, args: Vec<impl Into<String>>) -> &mut Self {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }

    pub fn set_env(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.env.insert(key.into(), value.into());
        self
    }

    pub fn envs(&mut self, map: &HashMap<String, String>) {
        for (k, v) in map {
            self.env.insert(
                k.to_owned(), 
                v.to_owned()
            );
        }
    }

    pub fn set_quiet(&mut self, state: bool) -> &mut Self {
        self.quiet = state;
        self
    }

    pub fn prime(&mut self) -> &mut Self {
        self.set_env("__NV_PRIME_RENDER_OFFLOAD", "1");
        self.set_env("__GLX_VENDOR_LIBRARY_NAME", "nvidia");
        self.set_env("__VK_LAYER_NV_optimus", "NVIDIA_only");
        self.set_env("VK_ICD_FILENAMES", "/usr/share/vulkan/icd.d/nvidia_icd.json");

        self
    }

    pub fn exists(&self) -> bool {
        return command_exists(&self.command);
    }

    pub fn to_string(&self) -> String {
        format!("{} {}", self.command, self.args.join(" "))
    }

    fn get_exe_prefix() -> String {
        Style::new().fg(Blue).paint("Executing >").to_string()
    }

    pub fn to_wine(&mut self) -> Self {
        #[cfg(target_family = "windows")] {
            return self.to_owned();
        }

        #[cfg(target_family = "unix")] {
            let wine = config::get().software.wine;
            let mut new = CommandRunner::new(wine);
    
            self.ignore_os_path = true;

            new.set_env("WINEDEBUG", "-all");
            new.envs(&self.env);
            new.set_quiet(self.quiet);
            new.add_args(vec![self.get_path().unwrap().to_str().unwrap()]); // TODO: Error handling
            new.add_args(self.args.to_owned());
    
            new
        }
    }

    pub fn run(&self) {
        let mut quiet = self.quiet;
        
        if get_command_line_settings().verbose {
            quiet = false;
        }

        if !self.exists() {
            exit_err(format!("Can't run '{}': not found.", &self.command));
        }

        let mut command = Command::new(&self.get_path().unwrap_or_else(|| {
            exit_err(format!("Command '{}' not found. (OS PATH and Lover config was searched too)", &self.command));
        }));

        let cmd_str = self.to_string();

        if quiet {
            command.stdout(Stdio::null());
        } else {
            println!("{} {}", Self::get_exe_prefix(), &cmd_str);

            command.stdout(Stdio::inherit());
            command.stderr(Stdio::inherit());
        }

        command.args(&self.args);
        command.envs(&self.env);

        let out = command.output().unwrap_or_else(|err| {
            exit_err(format!("Failed to execute: {}:\n {}", err, &cmd_str));
        });

        let exit_code_text = match out.status.code() {
            Some(code) => code.to_string(),
            None => "unknown".to_string()
        };

        if !out.status.success() {
            if quiet {
                let stdout = String::from_utf8(out.stdout).unwrap_or_else(|err| {
                    print_warn(format!("Failed to decode stdout utf8: {}", err));
                    "".to_string()
                });

                let stderr = String::from_utf8(out.stderr).unwrap_or_else(|err| {
                    print_warn(format!("Failed to decode stderr utf8: {}", err));
                    "".to_string()
                });

                println!("{} {}", Self::get_exe_prefix(), cmd_str);
                println!("{}{}", stdout, stderr);
            }

            exit_err(format!("Command failed with code: {}", exit_code_text))
        }
    }
}

pub fn command_exists(command: &str) -> bool {
    let as_path = Path::new(command);
    
    if as_path.is_file() {
        return true;
    }

    #[cfg(target_os = "windows")]
    let command = if command.ends_with(".exe") {
        command.to_string()
    } else {
        format!("{}.exe", command)
    };

    match std::env::var_os("PATH") {
        Some(path_env) => {
            for path_dir in std::env::split_paths(&path_env) {
                if path_dir.join(&command).is_file() {
                    return true;
                }
            }

            false
        }
        None => false
    }
}

pub fn parse_all(root: &Path) {
    let parser = config::get().software.luac;
    let scripts = files::get_file_tree_of_type(root, "lua");

    if !command_exists(&parser) {
        print_warn(format!("'{}' not found. Skipping luac parse.", &parser));
        return;
    }

    print_step("Checking validity of Lua scripts...");

    for script in &scripts {
        CommandRunner::new(&parser)
            .add_args(vec!["-p", script.to_str().unwrap()])
            .set_quiet(true)
            .run();
    }

    print_step("Checking for deprecated features...");

    let env_repl = get_env_replacement_map();

    for script in &scripts {
        let mut file = files::open(&script);
        let mut buf = String::new();
        let script_path_str = script.to_str().unwrap();

        file.read_to_string(&mut buf).unwrap_or_else(|err| {
            exit_err(format!("Failed to read buffer of {}: {}", script_path_str, err))
        });

        let lines: Vec<&str> = buf.lines().collect();
        
        for i in 0..lines.len() {
            let line = lines[i].to_string();

            for (old, new) in &env_repl {
                if line.contains(old) {
                    print_warn(format!("{}:{} Use '{}' instead of {}", script_path_str, i + 1, new, old));
                }
            }
        }
    }

    print_success_verbose(
        &get_command_line_settings(),
        "Parsing successful"
    );
}

pub fn clean(path: &Path) {
    if !path.exists() {
        print_success("Nothing to clean.");
        return;
    }

    if path.is_file() {
        exit_err(format!("'{}' is not a directory!", path.to_str().unwrap()));
    }

    match std::fs::remove_dir_all(path) {
        Ok(()) => print_success("Clean successful"),
        Err(err) => exit_err(format!("Failed to delete '{}': {}", path.to_str().unwrap(), err))
    };
}

pub fn add_to_archive(archive_path: &Path, file_path: &Path, inner_path: &Path) {
    let archive_file = files::open_rw(archive_path);
    let mut file = files::open(file_path);

    let mut zip = zip::ZipWriter::new_append(archive_file).unwrap_or_else(|err| {
        exit_err(format!("Failed to open zip: {}", err));
    });

    print_step_verbose(
        &get_command_line_settings(), 
        format!("Adding {} to {}", file_path.to_str().unwrap(), archive_path.to_str().unwrap())
    );

    let mut buf: Vec<u8> = Vec::new();
    file.read_to_end(&mut buf).unwrap();

    zip.start_file_from_path(&inner_path, SimpleFileOptions::default()).unwrap_or_else(|err| {
        exit_err(format!("Failed to start file '{}': {}", &inner_path.to_str().unwrap(), err))
    });

    zip.write_all(&buf).unwrap_or_else(|err| {
        exit_err(format!("Failed to write to zip: {}", err));
    });
}

pub fn append_file(from: &Path, to: &Path, text: impl Into<String>) {
    let mut from_file = files::open(from);
    let mut to_file = files::open_append(to);

    let size = files::get_size(from);

    let mut bar = ProgressBar::new(size);
    bar.set_prefix(text);

    let mut progress: usize = 0;

    loop {
        let mut buf: [u8; 1024] = [0; 1024];
        
        let bytes_read = from_file.read(&mut buf).unwrap_or_else(|err| {
            exit_err(format!("Read failed: {}", err));
        });

        if bytes_read == 0 { 
            break; 
        } else {
            progress = progress.saturating_add(bytes_read);
            bar.update(progress);
        }

        let write_res = to_file.write_all(&buf[..bytes_read]);

        if write_res.is_err() {
            exit_err(format!("Write failed: {}", write_res.err().unwrap()));
        }
    }

    bar.finish();
}

pub fn get_env_replacement_map() -> HashMap<String, String> {
    let mut map: HashMap<String, String> = HashMap::new();

    map.insert("LOVER_NAME".to_string(), "LOVER_PKG_VERSION".to_string());
    map.insert("LOVER_VERSION".to_string(), "LOVER_PKG_VERSION".to_string());
    map.insert("LOVER_AUTHOR".to_string(), "LOVER_PKG_AUTHOR".to_string());
    map.insert("LOVER_DESCRIPTION".to_string(), "LOVER_PKG_DESCRIPTION".to_string());
    
    map
}

pub fn get_env_map(context: Context) -> HashMap<String, String> {
    let mut map: HashMap<String, String> = HashMap::new();

    let project_conf = project_config::get();
    let env = project_conf.env;
    let pkg = project_conf.package;

    let ctx_map = match context {
        Context::Build => env.build,
        Context::Run => env.run
    };

    for (k, v) in ctx_map {
        map.insert(k, v);
    }

    for (k, v) in env.global {
        map.insert(k, v);
    }

    let ctx_str = match context {
        Context::Build => "build",
        Context::Run => "run"
    }.to_string();

    let timestamp = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(res) => res,
        Err(err) => {
            print_warn(format!("Error getting UNIX timestamp: {}.\nLOVER_TIMESTAMP will be equal to 0", err));
            Duration::from_secs(0)
        }
    }.as_secs();

    map.insert("LOVER_CONTEXT".to_string(), ctx_str);
    map.insert("LOVER_TIMESTAMP".to_string(), timestamp.to_string());

    map.insert("LOVER_PKG_VERSION".to_string(), pkg.version);
    map.insert("LOVER_PKG_NAME".to_string(), pkg.name);
    map.insert("LOVER_PKG_AUTHOR".to_string(), pkg.author);
    map.insert("LOVER_PKG_DESCRIPTION".to_string(), pkg.description);
    
    return map;
}