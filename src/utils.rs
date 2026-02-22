#[cfg(not(target_os = "windows"))]
use std::env;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use duct::cmd;
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
    // Substitute {url}
    let rendered = template.replace("{url}", original_url);
    debug!("Command template: {}", rendered);

    let mut extra_env: Vec<(String, String)> = Vec::new();

    #[cfg(not(target_os = "windows"))]
    {
        let shell = env::var("SHELL").unwrap_or(String::from("bash"));
        debug!("Using shell: {}", shell);
        // We want environment variable expansion via fish, but we only need the environment values,
        // not to run the command. Approach:
        // 1. Capture fish environment into KEY=VALUE lines.
        // 2. Apply those vars to our current process execution of the command (without fish).
        // NOTE: This uses 'env' in shell to print all exported variables.
        let fish_env_output = match cmd!(&shell, "-l", "-c", "env").stderr_to_stdout().read() {
            Ok(out) => out,
            Err(e) => {
                warn!("Shell env capture failed: {}", e);
                String::new()
            }
        };

        // Parse KEY=VALUE lines
        for line in fish_env_output.lines() {
            if let Some((k, v)) = line.split_once('=')
                && !k.is_empty()
            {
                extra_env.push((k.to_string(), v.to_string()));
            }
        }
        debug!("Captured {} env vars from fish", extra_env.len());
    }

    // Split original (rendered) command into program/args AFTER variable substitution (fish didn't expand inside rendered).
    // If you need variable substitution inside rendered, we would need 'fish -c "echo $VAR"'. For now we keep user-provided values.
    let parts = match shlex::split(&rendered) {
        Some(v) if !v.is_empty() => v,
        _ => {
            warn!("Failed to parse command template: '{}'", rendered);
            return None;
        }
    };
    let (program, args) = parts.split_first().unwrap();
    debug!(
        "Executing program='{}' args={:?} with fish env",
        program, args
    );

    let mut command = cmd(program, args);
    for (k, v) in extra_env {
        debug!("Adding env: {}={}", k, v);
        command = command.env(k, v);
    }

    match command.stderr_to_stdout().read() {
        Ok(output) => {
            let last_line = output.lines().rev().find(|l| !l.trim().is_empty());
            last_line.map(|s| s.trim().to_string())
        }
        Err(e) => {
            warn!("Failed to execute command '{}': {}", rendered, e);
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

    debug!("Final URL: {}", result);
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
