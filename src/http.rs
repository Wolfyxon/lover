use reqwest::{blocking::{Client, Response}, Error};
use serde::{Deserialize};
use serde::de::DeserializeOwned;
use std::{fs::File, io::{Read, Write}, path::Path, process::exit};

use crate::console::ProgressBar;
use crate::console::{print_err, print_warn, print_success, print_stage};

const USER_AGENT: &str = "Lover";

#[derive(Deserialize)]
pub struct GitHubRelease { 
    // Not all fields are needed. Add only those that are necessary.
    pub assets: Vec<GithubReleaseAsset>
}

#[derive(Deserialize)]
pub struct GithubReleaseAsset {
    pub url: String,
    pub name: String,
    pub size: u32
}

pub fn fetch_text(url: &str) -> Result<String, Error> {
    let res = Client::new()
        .get(url)
        .header("User-Agent", USER_AGENT)
        .send()?;

    res.text()
}

pub fn fetch_struct<T: DeserializeOwned>(url: &str) -> Result<T, Error> {
    let res = fetch_text(url)?;

    Ok(serde_json::from_str(res.as_str()).unwrap())
}

pub fn download(url: &str, path: &Path) {
    print_stage(format!("Downloading '{}'...", url));

    let file_res = File::create(path);

    if file_res.is_err() {
        print_err(format!("Failed to open '{}': {}", path.to_str().unwrap(), file_res.err().unwrap()));
        exit(1);
    }

    let req_res = Client::new()
        .get(url)
        .header("User-Agent", USER_AGENT)
        .send();

    if req_res.is_err() {
        print_err(format!("Request failed: {}", req_res.err().unwrap()));
        exit(1);
    }

    let mut req = req_res.unwrap();
    let mut file = file_res.unwrap();
    let len_res = req.content_length();

    if len_res.is_none() {
        print_err("Failed to get content length".to_string());
        exit(1);
    }

    let len = len_res.unwrap() as usize;
    let mut bar = ProgressBar::new(len);
    let mut bytes: usize = 0;
    
    loop {
        let mut buf: [u8; 1024] = [0; 1024];
        let read_res = req.read(&mut buf);

        if read_res.is_err() {
            print_err(format!("Read failed: {}", read_res.err().unwrap()));
            exit(1);
        }

        let bytes_read = read_res.unwrap();
        if bytes_read == 0 { break; }

        let write_res = file.write_all(&buf[..bytes_read]);

        if write_res.is_err() {
            print_err(format!("Write failed: {}", write_res.err().unwrap()));

            bar.finish();
            exit(1);
        }

        bytes += bytes_read;

        if bytes > len {
            bytes = len;
        }

        bar.update(bytes);
    }
    
    bar.finish();
    print_success(format!("Downloaded to: '{}'", path.to_str().unwrap()));
}

pub fn fetch_gh_release(owner: &str, repo: &str, release: &str) -> Result<GitHubRelease, Error> {
    let url = format!("https://api.github.com/repos/{}/{}/releases/{}", owner, repo, release);
    
    fetch_struct(url.as_str())
}

pub fn fetch_gh_latest_release(owner: &str, repo: &str) -> Result<GitHubRelease, Error> {
    fetch_gh_release(owner, repo, "latest")
}