use std::path::Path;

use crate::actions;
use crate::project_config;
use crate::deps;

pub struct BuildTarget<'a> {
    pub name: &'a str,
    pub description: &'a str,
    pub deps: Vec<&'a str>,
    pub previous: Vec<&'a str>,
    builder: fn()
}

impl<'a> BuildTarget<'a> {
    pub fn get_deps(&self) -> Vec<String> {
        let mut res:Vec<String> = Vec::new();

        for name in &self.previous {
            let prev = deps::get_dep(name);

            res.push(prev.name.to_string());
        }

        res
    }
}

pub fn get_targets<'a>() -> Vec<BuildTarget<'a>> {
    vec![
        BuildTarget {
            name: "love",
            description: "Game's code packaged in the Love format.",
            deps: Vec::new(),
            previous: Vec::new(),
            builder: || {
                let config = project_config::get();
                let output = Path::new(config.directories.build.as_str()).join(config.package.name + ".love");
            
                actions::parse_all(Path::new(&project_config::get().directories.source));
                actions::archive(Path::new(config.directories.source.as_str()), &output);
            }
        }
    ]
}

pub fn get_target_by_string<'a>(name: String) -> Option<BuildTarget<'a>> {
    for target in get_targets() {
        if target.name == name {
            return Some(target);
        }
    }

    None
}