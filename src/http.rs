use reqwest::blocking::{Client, Response};
use serde::de::DeserializeOwned;
use std::{fs::File, io::{Read, Write}, path::Path};

use crate::console::{exit_err, print_warn, ProgressBar};
use crate::console::print_success;

pub struct Downloadable {
    response: Response,
    //url: String
}

impl<'a> Downloadable {
    pub fn request(url: String) -> Self {
        let res = get_request(url.to_owned().as_str());

        Self {
            response: res,
            //url: url
        }
    }

    pub fn download(&mut self, path: &Path) {
        download_response(&mut self.response, path);
    }

    pub fn len(&self) -> Option<u64> {
        self.response.content_length()
    }
}

pub fn get_user_agent() -> String {
    format!("Lover/{}", env!("CARGO_PKG_VERSION"))
}

pub fn fetch_text(url: &str) -> String {
    let res = Client::new()
        .get(url)
        .header("User-Agent", get_user_agent())
        .send();

    if res.is_err() {
        exit_err(format!("Failed to request '{}': {}", url, res.err().unwrap()));
    } 

    match res.unwrap().text() {
        Ok(text) => text,
        Err(err) => exit_err(format!("Failed to get text from '{}': {}", url, err))
    }
}

pub fn fetch_struct<T: DeserializeOwned>(url: &str) -> T {
    match serde_json::from_str(fetch_text(url).as_str()) {
        Ok(parsed) => parsed,
        Err(err) => exit_err(format!("Struct parse error of '{}': {}", url, err))
    }
}

pub fn get_request(url: &str) -> Response {
    let client = Client::new()
        .get(url)
        .header("User-Agent", get_user_agent());

    match client.send() {
        Ok(res) => res,
        Err(err) => exit_err(format!("Request failed: {}", err))
    }
}

pub fn download_response(response: &mut Response, path: &Path) {
    let mut file = match File::create(path) {
        Ok(file) => file,
        Err(err) => exit_err(format!("Failed to open '{}': {}", path.to_str().unwrap(), err)) 
    };

    let len = match response.content_length() {
        Some(len) => len as usize,
        None => 0
    };

    let bar = ProgressBar::new(len);
    let mut bytes: usize = 0;
    
    loop {
        let mut buf: [u8; 1024] = [0; 1024];

        let bytes_read = match response.read(&mut buf) {
            Ok(bytes) => bytes,
            Err(err) => exit_err(format!("Read failed: {}", err)) 
        };

        if bytes_read == 0 { break; }

        let write_res = file.write_all(&buf[..bytes_read]);

        if write_res.is_err() {
            bar.finish();
            exit_err(format!("Write failed: {}", write_res.err().unwrap()));
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
