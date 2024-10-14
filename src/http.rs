use reqwest::{blocking::{Client, Response}, Error};
use serde::{Deserialize};
use serde::de::DeserializeOwned;

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

pub fn fetch_struct<T: DeserializeOwned>(url: &str) -> Result<T, Error> {
    let res = Client::new()
        .get(url)
        .header("User-Agent", USER_AGENT)
        .send();

    if res.is_err() {
        return Err(res.err().unwrap());
    }

    Ok(serde_json::from_str(res.unwrap().text().unwrap().as_str()).unwrap())
}

pub fn fetch_gh_release(owner: &str, repo: &str, release: &str) -> Result<GitHubRelease, Error> {
    let url = format!("https://api.github.com/repos/{}/{}/releases/{}", owner, repo, release);
    
    fetch_struct(url.as_str())
}

pub fn fetch_gh_latest_release(owner: &str, repo: &str) -> Result<GitHubRelease, Error> {
    fetch_gh_release(owner, repo, "latest")
}