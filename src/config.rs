use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::fs;
use std::path::Path;

pub enum Downloader {
    Aria2c,
    Curl,
}

impl Display for Downloader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Downloader::Aria2c => write!(f, "aria2c"),
            Downloader::Curl => write!(f, "curl"),
        }
    }
}

/// Config for URL replacements
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub replacements: Vec<UrlReplacement>,
    #[serde(default = "default_aria2c_path")]
    pub aria2c_path: String,
    #[serde(default = "default_curl_path")]
    pub curl_path: String,
}

/// Default path for aria2c executable
fn default_aria2c_path() -> String {
    "aria2c".to_string()
}

/// Default path for curl executable
fn default_curl_path() -> String {
    "curl".to_string()
}

/// A single URL replacement rule
/// `replacement` serves dual purpose:
/// - If it contains the token `{url}` it is treated as a command template.
///   The command is executed (with `{url}` substituted by the original URL) and
///   the last line of stdout becomes the new URL.
/// - Otherwise it is treated as a regex replacement string applied to `pattern`.
#[derive(Debug, Serialize, Deserialize)]
pub struct UrlReplacement {
    pub pattern: String,
    pub replacement: String,
}

impl Config {
    /// Load configuration from the specified file
    pub fn from_file(path: &Path) -> Result<Self> {
        let content =
            fs::read_to_string(path).context(format!("Failed to read config file: {:?}", path))?;
        let config: Config = toml::from_str(content.as_str())?;
        Ok(config)
    }

    pub fn get_downloader_path(&self, downloader: Downloader) -> &str {
        match downloader {
            Downloader::Aria2c => &self.aria2c_path,
            Downloader::Curl => &self.curl_path,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            replacements: Vec::new(),
            aria2c_path: default_aria2c_path(),
            curl_path: default_curl_path(),
        }
    }
}
