use ansi_term::Color::Blue;
use ansi_term::Style;
use std::collections::HashMap;
use std::env::split_paths;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::process::Command;
use std::process::Stdio;
use zip::write::SimpleFileOptions;
use zip::ZipArchive;

use crate::config;
use crate::console;
use crate::console::exit_err;
use crate::console::get_command_line_settings;
use crate::console::get_step_prefix;
use crate::console::print_err;
use crate::console::print_step_verbose;
use crate::console::print_success_verbose;
use crate::console::ProgressBar;
use crate::console::{print_step, print_success, print_warn};
use crate::files;
use crate::files::get_file_tree;
use crate::project_config;
use crate::targets::Arch;
use crate::targets::OS;

pub enum Context {
    Run,
    Build,
}

pub struct Archiver {
    dir: PathBuf,
    progress_bar: Option<ProgressBar>,
    ignored_files: Vec<PathBuf>,
}

impl Archiver {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self {
            dir: dir.into(),
            progress_bar: None,
            ignored_files: Vec::new(),
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

    pub fn ignore_files(&mut self, files: &Vec<PathBuf>) -> &mut Self {
        for path in files {
            self.ignore_file(path);
        }
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
            bar.max = tree.len() - self.ignored_files.len();
        });

        print_step_verbose(
            &get_command_line_settings(),
            format!(
                "Archiving '{}' into '{}'...",
                &self.dir.to_str().unwrap(),
                output_dir.to_str().unwrap()
            ),
        );

        let ignored_set: std::collections::HashSet<PathBuf> = self
            .ignored_files
            .iter()
            .map(|p| self.dir.join(p))
            .collect();

        for path in tree {
            if ignored_set.contains(&path) {
                continue;
            }

            let out_path = files::skip_path(&path, &self.dir);
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
    progress_bar: Option<ProgressBar>,
}

impl Extractor {
    pub fn new(source: impl Into<PathBuf>) -> Self {
        Self {
            source: source.into(),
            progress_bar: None,
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

        let mut archive = ZipArchive::new(zip_file)
            .unwrap_or_else(|err| exit_err(format!("ZIP failed for '{}': {}", source_str, err)));

        let archive_len = archive.len();

        print_step_verbose(
            &get_command_line_settings(),
            format!("Extracting '{}' to '{}'...", source_str, dir_str),
        );

        self.progress_bar.as_mut().map(|bar| {
            bar.max = archive_len;
            bar.update(0);
        });

        for i in 0..archive_len {
            let mut file = archive.by_index(i).unwrap_or_else(|err| {
                exit_err(format!(
                    "Failed to get file at index {} of '{}': {}",
                    i, source_str, err
                ));
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
                }
                None => exit_err(format!(
                    "Failed to get path of file at index {} of '{}'",
                    i, source_str
                )),
            };

            let mut out_file = files::create(output_dir.join(path).as_path());

            loop {
                let mut buf: [u8; 1024] = [0; 1024];

                let bytes_read = file
                    .read(&mut buf)
                    .unwrap_or_else(|err| exit_err(format!("Read failed: {}", err)));

                if bytes_read == 0 {
                    break;
                }

                out_file
                    .write_all(&buf[..bytes_read])
                    .unwrap_or_else(|err| {
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
    required: bool,
    ignore_os_path: bool,
    ignore: bool,
    error_hint: Option<String>,
}

impl CommandRunner {
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            args: Vec::new(),
            env: HashMap::new(),
            paths: Vec::new(),
            quiet: false,
            required: true,
            ignore_os_path: false,
            ignore: false,
            error_hint: None,
        }
    }

    pub fn with_args(&self, args: Vec<impl Into<String>>) -> Self {
        let mut new = self.clone();
        new.add_args(args);

        new
    }

    pub fn unrequire(&mut self) -> &mut Self {
        self.required = false;
        self
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
                        res.push(file_path.clone());

                        #[cfg(target_family = "windows")]
                        {
                            res.push(file_path.with_extension("exe"));
                        }
                    }
                }
                None => (),
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
            self.env.insert(k.to_owned(), v.to_owned());
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
        
        self.set_env(
            "VK_ICD_FILENAMES",
            "/usr/share/vulkan/icd.d/nvidia_icd.json",
        );

        self
    }

    pub fn exists(&self) -> bool {
        return self.get_path().is_some();
    }

    pub fn to_string(&self) -> String {
        format!("{} {}", self.command, self.args.join(" "))
    }

    fn get_exe_prefix() -> String {
        Style::new().fg(Blue).paint("Executing >").to_string()
    }

    pub fn check_exists(&self) -> bool {
        if !self.exists() {
            let err = format!(
                "'{}' not found. (OS PATH and Lover config was searched too)",
                &self.command
            );

            if self.required {
                exit_err(err);
            } else {
                print_warn(err);
                return false;
            }
        }

        return true;
    }

    pub fn to_wine(&mut self) -> Self {
        #[cfg(target_family = "windows")]
        {
            return self.to_owned();
        }

        #[cfg(target_family = "unix")]
        {
            let wine = config::get().software.wine;
            let mut new = CommandRunner::new(wine);

            self.ignore_os_path = true;

            if !self.check_exists() {
                self.ignore = true;
                return self.to_owned();
            }

            new.required = self.required;
            new.set_env("WINEDEBUG", "-all");
            new.envs(&self.env);
            new.set_quiet(self.quiet);

            new.add_args(vec![self
                .get_path()
                .expect("Path retrieval failed despite previous checks")
                .to_str()
                .expect("Path to str failed")]);

            new.add_args(self.args.to_owned());

            new
        }
    }

    #[allow(unused)]
    pub fn set_error_hint(&mut self, text: impl Into<String>) -> &mut Self {
        self.error_hint = Some(text.into());
        self
    }

    pub fn run(&self) {
        if self.ignore || !self.check_exists() {
            return;
        }

        let mut quiet = self.quiet;

        if get_command_line_settings().verbose {
            quiet = false;
        }

        let mut command = Command::new(&self.get_path().unwrap());

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

        let out = match command.output() {
            Ok(out) => out,
            Err(err) => {
                let msg = format!("Failed to execute: {}:\n {}", err, &cmd_str);

                if self.required {
                    exit_err(msg);
                } else {
                    print_err(msg);
                    return;
                }
            }
        };

        let exit_code_text = match out.status.code() {
            Some(code) => code.to_string(),
            None => "unknown".to_string(),
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

            println!();
            print_err(format!("Command failed with code: {}", exit_code_text));

            self.error_hint.as_ref().map(|hint| println!("{}", hint));

            if self.required {
                exit(1);
            }
        }
    }
}

pub fn compile(arch: Arch, os: OS) {
    let mut compiler = CommandRunner::new("luajit");
    compiler.set_quiet(true);
    compiler.check_exists();

    let project = project_config::get();
    let src = project.directories.get_source_dir();
    let build = project.directories.get_build_dir();
    let comp_dir = project.directories.get_temp_dir().join("compiled_lua");
    let dir = comp_dir.join(format!("{}-{}", os.to_string(), arch.to_short_string()));

    let all_scripts = files::get_file_tree_of_type(&src, "lua");

    let scripts: Vec<&PathBuf> = all_scripts
        .iter()
        .filter(|path| !path.starts_with(&build))
        .collect();

    let mut bar = ProgressBar::new(scripts.len());
    bar.set_prefix(get_step_prefix() + " Compiling scripts");

    let mut progress: usize = 0;

    for script in scripts {
        let new_path = dir.join(PathBuf::from_iter(
            script.components().skip(src.components().count()),
        ));

        files::create_dir(&new_path.parent().unwrap());

        let cmd = compiler.with_args(vec![
            "-b".to_string(),
            script.display().to_string(),
            new_path.display().to_string(),
            "-a".to_string(),
            arch.to_short_string(),
            "-o".to_string(),
            os.to_string(),
        ]);

        cmd.run();

        progress = progress.saturating_add(1);
        bar.update(progress);
    }

    bar.finish();
}

pub fn get_parser() -> Option<CommandRunner> {
    let parser = CommandRunner::new(config::get().software.luac);

    if !parser.exists() {
        return None;
    }

    Some(parser)
}

pub fn parse_all(root: &Path) {
    let parser_res = get_parser();

    if parser_res.is_none() {
        print_warn(format!("luac Lua parser not found. Skipping."));
        return;
    }

    let parser = parser_res.unwrap();
    let scripts = files::get_file_tree_of_type(root, "lua");

    print_step("Checking validity of Lua scripts...");

    for script in &scripts {
        parser
            .with_args(vec!["-p", script.to_str().unwrap()])
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
            exit_err(format!(
                "Failed to read buffer of {}: {}",
                script_path_str, err
            ))
        });

        let lines: Vec<&str> = buf.lines().collect();

        for i in 0..lines.len() {
            let line = lines[i].to_string();

            for (old, new) in &env_repl {
                if line.contains(old) {
                    print_warn(format!(
                        "{}:{} Use '{}' instead of {}",
                        script_path_str,
                        i + 1,
                        new,
                        old
                    ));
                }
            }
        }
    }

    print_success_verbose(&get_command_line_settings(), "Parsing successful");
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
        Err(err) => exit_err(format!(
            "Failed to delete '{}': {}",
            path.to_str().unwrap(),
            err
        )),
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
        format!(
            "Adding {} to {}",
            file_path.to_str().unwrap(),
            archive_path.to_str().unwrap()
        ),
    );

    let mut buf: Vec<u8> = Vec::new();
    file.read_to_end(&mut buf).unwrap();

    zip.start_file_from_path(&inner_path, SimpleFileOptions::default())
        .unwrap_or_else(|err| {
            exit_err(format!(
                "Failed to start file '{}': {}",
                &inner_path.to_str().unwrap(),
                err
            ))
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

    bar.set_prefix(format!("{} {}", console::get_step_prefix(), text.into()));
    bar.memory_mode();

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
    map.insert(
        "LOVER_DESCRIPTION".to_string(),
        "LOVER_PKG_DESCRIPTION".to_string(),
    );

    map
}

pub fn get_comment_locations(code: impl Into<String>) -> Vec<(usize, usize)> {
    let code: String = code.into();
    let mut res: Vec<(usize, usize)> = Vec::new();

    let mut in_string = false;
    let mut in_ml_string = false;
    let mut escaped = false;

    let mut comment: Option<(usize, usize)> = None;
    let mut comment_ml = false;

    let chars: Vec<char> = code.chars().collect();

    for (i, char) in chars.iter().enumerate() {
        let next = chars.get(i + 1);
        let cmt_begin = *char == '-' && next.is_some() && *next.unwrap() == '-';
        let ml_begin = *char == '[' && next.is_some() && *next.unwrap() == '[';
        let ml_end = *char == ']' && next.is_some() && *next.unwrap() == ']';

        if !escaped && comment.is_none() {
            if *char == '\\' {
                escaped = true;
                continue;
            }

            if !in_ml_string {
                if *char == '"' || *char == '\'' {
                    in_string = !in_string;
                }

                if !in_string && ml_begin {
                    in_ml_string = true;
                }
            } else {
                if !in_string && ml_end {
                    in_ml_string = false;
                }
            }
        }

        if !in_string && !in_ml_string && comment.is_none() && cmt_begin {
            comment = Some((i, i));
        }

        if comment.is_some() {
            let (begin, mut end) = comment.unwrap();

            if ml_begin && (end - begin == 2) {
                comment_ml = true;
            }

            if (!comment_ml && *char == '\n') || (comment_ml && ml_end) {
                if comment_ml && ml_end {
                    end += 2;
                }

                res.push((begin, end));
                comment = None;
                comment_ml = false;
                continue;
            }

            comment = Some((begin, end + 1));
        }

        escaped = false;
    }

    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comment_finding() {
        let code = include_str!("testData/projects/project/src/unoptimized.lua").to_string();

        let sure = vec![
            "-- Do not touch any comments here this or the unit test will go mad",
            "--[[\n    This contains a bunch of useless comments\n]]",
            "-- this prints \"Hi\" on the screen",
        ];

        let mut found: Vec<String> = Vec::new();

        for (begin, end) in get_comment_locations(&code) {
            let cmt = code[begin..end].to_string();

            println!("= {} {} ==================", begin, end);
            println!("<<{}>>", &cmt);

            found.push(cmt);
        }

        assert_eq!(found.len(), sure.len(), "Comment amount differs");

        for comment in found {
            assert!(
                sure.contains(&comment.as_str()),
                "Extra comment: \n{}",
                comment
            );
        }
    }
}
