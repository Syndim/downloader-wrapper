use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};
use regex::Regex;
use tracing::{debug, warn};

use crate::config::Config;

/// Check if the string is a URL
pub fn is_url(s: &str) -> bool {
    s.starts_with("http://")
        || s.starts_with("https://")
        || s.starts_with("ftp://")
        || s.starts_with("magnet:")
}

fn execute_prehook(template: &str, original_url: &str) {
    let cmd = template.replace("{url}", original_url);
    debug!("Executing prehook: {}", cmd);
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() { return; }
    let (program, args) = parts.split_first().unwrap();
    match Command::new(program).args(args).status() {
        Ok(status) => {
            if !status.success() {
                warn!("Prehook command '{}' exited with status {:?}", cmd, status.code());
            }
        }
        Err(e) => warn!("Failed to execute prehook '{}': {}", cmd, e),
    }
}

/// Apply URL replacement rules to the given URL
pub fn apply_url_replacements(config: &Config, url_str: &str) -> String {
    let mut result = url_str.to_string();

    for rule in &config.replacements {
        match Regex::new(&rule.pattern) {
            Ok(regex) => {
                if regex.is_match(&result) {
                    if let Some(template) = &rule.prehook {
                        execute_prehook(template, url_str);
                    }
                    result = regex.replace_all(&result, &rule.replacement).to_string();
                }
            }
            Err(e) => {
                warn!("Invalid regex pattern '{}': {}", rule.pattern, e);
            }
        }
    }

    result
}

/// Process an input file containing URLs, applying replacements
pub fn replace_urls_in_file(input_path: &Path, config: &Config) -> Result<()> {
    // Read the original file
    let content = fs::read_to_string(input_path)
        .context(format!("Failed to read input file: {:?}", input_path))?;

    // Process each line applying URL replacements
    let processed_content: Vec<String> = content
        .lines()
        .map(|line| {
            if is_url(line.trim()) {
                apply_url_replacements(config, line)
            } else {
                line.to_string()
            }
        })
        .collect();

    fs::write(input_path, processed_content.join("\n"))
        .context(format!("Failed to write to file: {:?}", input_path))?;

    Ok(())
}
