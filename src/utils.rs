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

fn execute_command_template(template: &str, original_url: &str) -> Option<String> {
    let cmd = template.replace("{url}", original_url);
    debug!("Executing URL command template: {}", cmd);
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }
    let (program, args) = parts.split_first().unwrap();
    match Command::new(program).args(args).output() {
        Ok(output) => {
            if !output.status.success() {
                warn!(
                    "Command template '{}' exited with status {:?}",
                    cmd,
                    output.status.code()
                );
                return None;
            }
            let stdout = String::from_utf8_lossy(&output.stdout);
            let last_line = stdout.lines().filter(|l| !l.trim().is_empty()).last();
            last_line.map(|s| s.trim().to_string())
        }
        Err(e) => {
            warn!("Failed to execute command template '{}': {}", cmd, e);
            None
        }
    }
}

/// Apply URL replacement rules to the given URL
pub fn apply_url_replacements(config: &Config, url_str: &str) -> String {
    let mut result = url_str.to_string();

    for rule in &config.replacements {
        match Regex::new(&rule.pattern) {
            Ok(regex) => {
                if regex.is_match(&result) {
                    // If replacement contains `{url}`, treat as a command template.
                    if rule.replacement.contains("{url}") {
                        if let Some(new_url) = execute_command_template(&rule.replacement, &result)
                        {
                            debug!(
                                "URL replaced via command template: {} -> {}",
                                result, new_url
                            );
                            result = new_url;
                        }
                    } else {
                        let replaced = regex.replace_all(&result, &rule.replacement).to_string();
                        debug!("URL replaced via regex: {} -> {}", result, replaced);
                        result = replaced;
                    }
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
