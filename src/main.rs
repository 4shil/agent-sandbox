mod db;
mod sandbox;
mod recorder;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::SystemTime;

use crate::sandbox::SandboxFs;
use crate::recorder::Recorder;

#[derive(Parser)]
#[command(name = "abox")]
#[command(about = "🛡️  Invisible sandbox for AI coding agents")]
struct Cli {
    /// Agent to launch (claude, opencode, codex, gemini, etc.)
    agent: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Silent setup
    let home = std::env::var("HOME")?;
    let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
    let sandbox_name = format!("{}-{}", cli.agent, timestamp);
    let workspace = PathBuf::from(&home)
        .join(".agent-sandbox")
        .join("workspaces")
        .join(&sandbox_name);
    
    std::fs::create_dir_all(&workspace)?;
    std::fs::create_dir_all(workspace.join("logs"))?;

    // Setup sandbox (silent)
    let sfs = SandboxFs::new(&workspace)?;
    let _ = sfs.mount();

    // Setup recorder (silent)
    let logs_dir = workspace.join("logs");
    let mut recorder = Recorder::new(&sandbox_name, &cli.agent, "interactive", &logs_dir)?;

    // Launch agent in sandbox — pass through to user
    let mut cmd = Command::new(&cli.agent);
    cmd.current_dir(sfs.agent_root());
    cmd.envs(std::env::vars());
    cmd.stdin(Stdio::inherit());
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());

    let start = SystemTime::now();
    let status = cmd.status();
    let duration = start.elapsed()?.as_millis() as u64;

    // Record in background (silent)
    let _ = recorder.record_action("session", serde_json::json!({
        "duration_ms": duration,
        "exit_code": status.as_ref().ok().and_then(|s| s.code()),
    }));

    for file in sfs.modified_files().unwrap_or_default() {
        let _ = recorder.record_action("file_modified", serde_json::json!({
            "path": file.strip_prefix(&workspace).unwrap_or(&file).to_string_lossy(),
        }));
    }

    let _ = recorder.finish();

    // Exit with agent's exit code
    if let Ok(Some(code)) = status.map(|s| s.code()) {
        std::process::exit(code);
    }

    Ok(())
}
