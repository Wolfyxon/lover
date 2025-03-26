use std::collections::HashMap;
use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::process::Command;
use std::process::ExitStatus;
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

pub fn command_exists(command: &str) -> bool {
    let as_path = Path::new(command);
    
    if as_path.is_file() {
        return true;
    }
    
    let path_env_res = std::env::var_os("PATH");

    if path_env_res.is_none() {
        return false;
    }

    for path_dir in std::env::split_paths(&path_env_res.unwrap()) {
        if path_dir.join(command).is_file() {
            return true;
        }
    }

    return false;
}

pub fn execute_with_env(command: &str, args: Vec<String>, env: HashMap<String, String>, quiet: bool) -> ExitStatus {
    if !command_exists(command) {
        exit_err(format!("Can't run '{}': not found.", command));
    }

    if !quiet {
        let prefix = Style::new().fg(Blue).paint("Executing >");
        println!("{} {} {}", prefix, command, args.join(" "));
    }

    let mut pre_run = Command::new(command);
    pre_run.args(args);
    
    for (key, value) in env {
        pre_run.env(key, value);
    }

    if quiet {
        pre_run.stdout(Stdio::null());
    }

    let status = pre_run.status().unwrap_or_else(|err| {
        exit_err(format!("Failed to execute command: {}", err)) 
    });
    
    let exit_code = status.code().unwrap();

    if status.success() {
        if !quiet {
            println!("");
            print_success(format!("Command completed with code: {}", exit_code));
        }
    } else {
        println!("");
        exit_err(format!("Command failed with code: {}", exit_code));
    }

    status
}

pub fn execute(command: &str, args: Vec<String>, quiet: bool) -> ExitStatus{
    execute_with_env(command, args, HashMap::new(), quiet)
}

pub fn execute_wine(command: &str, mut args: Vec<String>, quiet: bool) -> ExitStatus {
    if std::env::consts::FAMILY == "unix" {
        let mut all_args = vec![command.to_string()];
        all_args.append(&mut args);

        let mut env: HashMap<String, String> = HashMap::new();
        env.insert("WINEDEBUG".to_string(), "-all".to_string());

        execute_with_env(&config::get().software.wine, all_args, env, quiet)
    } else {
        execute(command, args, quiet)
    }
}

pub fn execute_prime_with_env(command: &str, args: Vec<String>, env: HashMap<String, String>, quiet: bool) -> ExitStatus {
    let mut env = env.clone();

    env.insert("__NV_PRIME_RENDER_OFFLOAD".to_string(), "1".to_string());
    env.insert("__GLX_VENDOR_LIBRARY_NAME".to_string(), "nvidia".to_string());
    env.insert("__VK_LAYER_NV_optimus".to_string(), "NVIDIA_only".to_string());
    env.insert("VK_ICD_FILENAMES".to_string(), "/usr/share/vulkan/icd.d/nvidia_icd.json".to_string());
    
    execute_with_env(command, args, env, quiet)
}

/*
pub fn execute_prime(command: &str, args: Vec<String>, quiet: bool) -> ExitStatus {
    execute_prime_with_env(command, args, HashMap::new(),quiet)
}*/

pub fn parse_all(root: &Path) {
    let parser = config::get().software.luac;

    if !command_exists(&parser) {
        print_warn(format!("'{}' not found. Skipping parse.", &parser));
        return;
    }

    print_step("Parsing Lua scripts...");

    let scripts = files::get_file_tree_of_type(root, "lua");
    
    for script in scripts {
        execute(&parser, vec!["-p".to_string(), script.to_str().unwrap().to_string()], true);
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

pub fn append_file(from: &Path, to: &Path) {
    let mut from_file = files::open(from);
    let mut to_file = files::open_append(to);

    loop {
        let mut buf: [u8; 1024] = [0; 1024];
        
        let bytes_read = from_file.read(&mut buf).unwrap_or_else(|err| {
            exit_err(format!("Read failed: {}", err));
        });

        if bytes_read == 0 { break; }

        let write_res = to_file.write_all(&buf[..bytes_read]);

        if write_res.is_err() {
            exit_err(format!("Write failed: {}", write_res.err().unwrap()));
        }
    }
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