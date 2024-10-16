use std::path::Path;
use crate::actions;
use crate::project_config;

pub struct BuildTarget<'a> {
    pub name: &'a str,
    pub description: &'a str,
    pub deps: Vec<&'a str>,
    pub previous: Vec<&'a str>,
    builder: fn()
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