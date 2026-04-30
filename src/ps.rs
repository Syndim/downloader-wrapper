use anyhow::{Context, Result};
use std::process::{Command, ExitStatus, Stdio};
use tracing::info;

/// Execute downloader with the given arguments
pub fn run_with(args: &[String], exe_path: &str) -> Result<ExitStatus> {
    info!("Executing downloader with arguments: {:?}", args);
    info!("Using downloader path: {}", exe_path);

    Command::new(exe_path)
        .args(args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context(format!(
            "Failed to execute downloader at path: {}",
            exe_path
        ))
}
