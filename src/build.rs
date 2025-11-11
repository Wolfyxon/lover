use std::process::Command;

fn main() {
    let out = Command::new("git").args(["rev-parse", "HEAD"]).output();

    let hash = match out {
        Ok(out) => String::from_utf8(out.stdout).unwrap_or_else(|err| {
            println!("Failed to decode: {}", err);
            "unknown".to_string()
        }),
        Err(err) => {
            println!("Cannot run git: {}", err);
            "unknown".to_string()
        }
    };

    println!("cargo:rerun-if-changed=NULL");
    println!("cargo:rustc-env=GIT_HASH={}", hash);
}
