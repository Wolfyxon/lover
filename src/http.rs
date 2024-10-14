use reqwest::blocking::Client;
use serde::de::DeserializeOwned;
use std::{fs::File, io::{Read, Write}, path::Path, process::exit};

use crate::console::ProgressBar;
use crate::console::{print_err, print_success, print_stage};

const USER_AGENT: &str = "Lover";

pub fn fetch_text(url: &str) -> String {
    let res = Client::new()
        .get(url)
        .header("User-Agent", USER_AGENT)
        .send();

    if res.is_err() {
        print_err(format!("Failed to request '{}': {}", url, res.err().unwrap()));
        exit(1);
    } 

    let text_res = res.unwrap().text();

    if text_res.is_err() {
        print_err(format!("Failed to get text from '{}': {}", url, text_res.err().unwrap()));
        exit(1);
    }

    text_res.unwrap()
}

pub fn fetch_struct<T: DeserializeOwned>(url: &str) -> T {
    let res = serde_json::from_str(fetch_text(url).as_str()); 

    if res.is_err() {
        print_err(format!("Struct parse error of '{}': {}", url, res.err().unwrap()));
        exit(1);
    }

    res.unwrap()
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
