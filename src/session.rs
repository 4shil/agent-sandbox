// Session utilities
use anyhow::Result;
use std::path::PathBuf;

pub fn get_workspaces_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".agent-sandbox").join("workspaces")
}
