use anyhow::{Result, Context};
use std::path::PathBuf;
use std::process::Command;
use std::time::SystemTime;
use colored::Colorize;

use crate::db;
use crate::sandbox::SandboxFs;
use crate::recorder::Recorder;
use crate::limits::ResourceLimits;
use crate::network::NetworkPolicy;

/// Supported AI agents with their CLI commands
#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub name: String,
    pub binary: String,
    pub args: Vec<String>,
    pub description: String,
}

impl AgentConfig {
    pub fn detect(agent: &str, task: &str) -> Self {
        match agent {
            "claude" | "claude-code" => AgentConfig {
                name: "claude".to_string(),
                binary: "claude".to_string(),
                args: vec!["--print".to_string(), task.to_string()],
                description: "Anthropic Claude Code".to_string(),
            },
            "codex" => AgentConfig {
                name: "codex".to_string(),
                binary: "codex".to_string(),
                args: vec!["--quiet".to_string(), task.to_string()],
                description: "OpenAI Codex CLI".to_string(),
            },
            "opencode" => AgentConfig {
                name: "opencode".to_string(),
                binary: "opencode".to_string(),
                args: vec!["run".to_string(), task.to_string()],
                description: "OpenCode AI agent".to_string(),
            },
            "cursor" => AgentConfig {
                name: "cursor".to_string(),
                binary: "cursor".to_string(),
                args: vec!["--agent".to_string(), task.to_string()],
                description: "Cursor AI Editor".to_string(),
            },
            "gemini" => AgentConfig {
                name: "gemini".to_string(),
                binary: "gemini".to_string(),
                args: vec!["-p".to_string(), task.to_string()],
                description: "Google Gemini CLI".to_string(),
            },
            "aider" => AgentConfig {
                name: "aider".to_string(),
                binary: "aider".to_string(),
                args: vec!["--message".to_string(), task.to_string()],
                description: "Aider AI pair programmer".to_string(),
            },
            "goose" => AgentConfig {
                name: "goose".to_string(),
                binary: "goose".to_string(),
                args: vec!["run".to_string(), task.to_string()],
                description: "Block Goose agent".to_string(),
            },
            "sweep" => AgentConfig {
                name: "sweep".to_string(),
                binary: "sweep".to_string(),
                args: vec![task.to_string()],
                description: "Sweep AI".to_string(),
            },
            "echo" => AgentConfig {
                name: "echo".to_string(),
                binary: "echo".to_string(),
                args: vec![task.to_string()],
                description: "Echo (testing only)".to_string(),
            },
            // Fallback: treat agent as binary name with task as arg
            other => AgentConfig {
                name: other.to_string(),
                binary: other.to_string(),
                args: vec![task.to_string()],
                description: "Custom agent".to_string(),
            },
        }
    }

    pub fn all_supported() -> Vec<(&'static str, &'static str)> {
        vec![
            ("claude", "Anthropic Claude Code"),
            ("codex", "OpenAI Codex CLI"),
            ("opencode", "OpenCode AI agent"),
            ("cursor", "Cursor AI Editor"),
            ("gemini", "Google Gemini CLI"),
            ("aider", "Aider AI pair programmer"),
            ("goose", "Block Goose agent"),
            ("sweep", "Sweep AI"),
        ]
    }
}

pub fn run_agent(agent: &str, task: &str, sandbox_name: &str, limits: &ResourceLimits, network: &NetworkPolicy) -> Result<()> {
    let conn = db::get_connection()?;
    let workspace_path: String = conn.query_row(
        "SELECT workspace_path FROM sandboxes WHERE name = ?1",
        [sandbox_name],
        |row| row.get(0),
    ).context("Sandbox not found. Run 'abox init' first.")?;

    let workspace = PathBuf::from(workspace_path);
    let logs_dir = workspace.join("logs");

    let agent_config = AgentConfig::detect(agent, task);

    println!("🔌 Initializing sandbox filesystem...");
    let sfs = SandboxFs::new(&workspace)?;
    sfs.mount()?;

    println!("📝 Starting session recorder...");
    let mut recorder = Recorder::new(sandbox_name, agent, task, &logs_dir)?;

    println!("🛡️  Resource limits: {}", limits.describe());
    println!("🌐 {}", network.describe());
    println!("🤖 Agent: {} ({})", agent_config.name.cyan(), agent_config.description.dimmed());
    println!("   Task: {}", task);
    println!("   Session: {}", recorder.session_id().cyan());
    println!("   Workspace: {}", workspace.display().to_string().dimmed());
    println!();

    recorder.record_action("task_start", serde_json::json!({
        "agent": agent_config.name,
        "agent_binary": agent_config.binary,
        "agent_description": agent_config.description,
        "task": task,
        "workspace": workspace.to_string_lossy(),
        "limits": limits.describe(),
        "network": network.describe(),
    }))?;

    let start = SystemTime::now();

    let agent_binary = agent_config.binary.clone();
    let agent_args = agent_config.args.clone();
    
    let mut cmd = Command::new(&agent_binary);
    for arg in &agent_args {
        cmd.arg(arg);
    }
    cmd.current_dir(sfs.agent_root());

    let output = cmd.output();

    let duration = start.elapsed()?.as_millis() as u64;

    match output {
        Ok(result) => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            let stderr = String::from_utf8_lossy(&result.stderr);

            recorder.record_exec(
                &format!("{} {}", agent_binary, agent_args.join(" ")),
                &stdout,
                &stderr,
                result.status.code().unwrap_or(-1),
                duration,
            )?;

            for file in sfs.modified_files()? {
                recorder.record_action("file_modified", serde_json::json!({
                    "path": file.strip_prefix(&workspace)?.to_string_lossy(),
                }))?;
            }

            println!("✅ Task completed in {:.1}s", duration as f64 / 1000.0);
            println!("   Exit code: {:?}", result.status.code());
            println!("   Actions recorded: {}", recorder.action_count());
            println!("   Log: {}", logs_dir.join(format!("{}.json", recorder.session_id())).display());

            if !stdout.is_empty() {
                println!("\n{}", "--- Output ---".dimmed());
                println!("{}", stdout);
            }
            if !stderr.is_empty() {
                println!("\n{}", "--- Errors ---".red());
                println!("{}", stderr);
            }
        }
        Err(e) => {
            let suggestion = if e.kind() == std::io::ErrorKind::NotFound {
                format!("Not found. Install it first: abox agent install {}", agent_config.name)
            } else {
                e.to_string()
            };
            
            recorder.record_action("task_error", serde_json::json!({
                "error": e.to_string(),
                "suggestion": suggestion,
            }))?;
            
            println!();
            println!("{} Failed to run {}", "❌".red(), agent_config.name.red().bold());
            println!("   {}", suggestion.yellow());
        }
    }

    recorder.finish()?;
    Ok(())
}
