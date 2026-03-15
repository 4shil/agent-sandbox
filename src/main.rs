mod cli;
mod db;
mod sandbox;
mod recorder;
mod replay;
mod limits;
mod network;
mod export;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use std::path::PathBuf;
use std::process::Command;
use std::time::SystemTime;

use crate::sandbox::SandboxFs;
use crate::recorder::Recorder;
use crate::limits::ResourceLimits;
use crate::network::NetworkPolicy;

#[derive(Parser)]
#[command(name = "abox")]
#[command(about = "🛡️  Sandbox wrapper for AI coding agents")]
#[command(long_about = "Run any AI agent inside an isolated sandbox.\n\nJust prefix your agent command with 'abox' — everything else works the same.")]
#[command(after_help = "EXAMPLES
  abox claude \"Build a REST API\"
  abox opencode \"Fix the bug\"
  abox codex \"Add tests\"
  abox gemini \"Refactor this\"
  
FLAGS
  --help-agent    Show agent-specific help
")]
struct Cli {
    /// The agent to run (claude, opencode, codex, gemini, etc.)
    agent: String,

    /// Task/prompt for the agent (optional — if empty, launches interactive mode)
    task: Option<String>,

    /// Max memory limit (e.g., 2gb, 512mb)
    #[arg(long)]
    memory: Option<String>,

    /// Max timeout (e.g., 30m, 1h)
    #[arg(long)]
    timeout: Option<String>,

    /// Block network access
    #[arg(long)]
    no_network: bool,

    /// Allow specific domains (repeatable)
    #[arg(long)]
    allow_domain: Vec<String>,

    /// Sandbox name (auto-generated if not set)
    #[arg(long)]
    name: Option<String>,

    /// Show session after completion
    #[arg(long)]
    stats: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Generate sandbox name if not provided
    let sandbox_name = cli.name.unwrap_or_else(|| {
        chrono::Local::now().format("%Y%m%d-%H%M%S").to_string()
    });

    // Detect agent config
    let (binary, args_template, description) = detect_agent(&cli.agent, cli.task.as_deref());

    // Check if agent exists
    if which::which(&binary).is_err() {
        eprintln!("{} {} not found in PATH", "❌".red(), binary.red().bold());
        eprintln!("   Install it first or check your PATH");
        std::process::exit(1);
    }

    // Create workspace
    let home = std::env::var("HOME")?;
    let workspace = PathBuf::from(&home).join(".agent-sandbox").join("workspaces").join(&sandbox_name);
    std::fs::create_dir_all(&workspace)?;
    std::fs::create_dir_all(workspace.join("logs"))?;

    // Init sandbox FS
    let sfs = SandboxFs::new(&workspace)?;
    sfs.mount()?;

    // Init recorder (auto-records everything)
    let logs_dir = workspace.join("logs");
    let mut recorder = Recorder::new(&sandbox_name, &cli.agent, cli.task.as_deref().unwrap_or("interactive"), &logs_dir)?;

    // Setup limits
    let limits = ResourceLimits::from_args(
        cli.memory.as_deref(),
        None,
        cli.timeout.as_deref(),
        None,
    )?;

    let network = NetworkPolicy::new(cli.no_network, cli.allow_domain.clone());

    // Show minimal info
    eprintln!("{} sandbox: {}", "🛡️".dimmed(), sandbox_name.cyan().dimmed());
    eprintln!("{} {}", limits.describe().dimmed(), network.describe().dimmed());

    // Build command
    let mut cmd = Command::new(&binary);
    for arg in &args_template {
        cmd.arg(arg);
    }
    cmd.current_dir(sfs.agent_root());
    cmd.envs(std::env::vars());
    cmd.stdin(std::process::Stdio::inherit());
    cmd.stdout(std::process::Stdio::inherit());
    cmd.stderr(std::process::Stdio::inherit());

    // Record start
    recorder.record_action("task_start", serde_json::json!({
        "agent": cli.agent,
        "binary": binary,
        "task": cli.task,
        "workspace": workspace.to_string_lossy(),
    }))?;

    let start = SystemTime::now();

    // Run the agent
    let status = cmd.status();

    let duration = start.elapsed()?.as_millis() as u64;

    // Record result
    match &status {
        Ok(s) => {
            recorder.record_action("task_end", serde_json::json!({
                "exit_code": s.code(),
                "duration_ms": duration,
            }))?;
        }
        Err(e) => {
            recorder.record_action("task_error", serde_json::json!({
                "error": e.to_string(),
                "duration_ms": duration,
            }))?;
        }
    }

    // Record modified files
    for file in sfs.modified_files()? {
        let _ = recorder.record_action("file_modified", serde_json::json!({
            "path": file.strip_prefix(&workspace).unwrap_or(&file).to_string_lossy(),
        }));
    }

    recorder.finish()?;

    // Show stats if requested
    if cli.stats {
        eprintln!();
        eprintln!("{} Session: {}", "📊".dimmed(), recorder.session_id().cyan().dimmed());
        eprintln!("{} {:.1}s", "⏱️".dimmed(), duration as f64 / 1000.0);
        eprintln!("{} {}", "📁".dimmed(), logs_dir.join(format!("{}.json", recorder.session_id())).display());
    }

    // Exit with same code as agent
    if let Ok(Some(code)) = status.map(|s| s.code()) {
        std::process::exit(code);
    }

    Ok(())
}

/// Detect agent binary and args from agent name
fn detect_agent(agent: &str, task: Option<&str>) -> (String, Vec<String>, String) {
    let task_str = task.unwrap_or("").to_string();
    
    match agent {
        "claude" | "claude-code" => (
            "claude".to_string(),
            if task_str.is_empty() { vec![] } else { vec!["--print".to_string(), task_str] },
            "Anthropic Claude Code".to_string(),
        ),
        "codex" => (
            "codex".to_string(),
            if task_str.is_empty() { vec![] } else { vec!["--quiet".to_string(), task_str] },
            "OpenAI Codex".to_string(),
        ),
        "opencode" => (
            "opencode".to_string(),
            if task_str.is_empty() { vec![] } else { vec!["run".to_string(), task_str] },
            "OpenCode".to_string(),
        ),
        "gemini" => (
            "gemini".to_string(),
            if task_str.is_empty() { vec![] } else { vec!["-p".to_string(), task_str] },
            "Google Gemini".to_string(),
        ),
        "aider" => (
            "aider".to_string(),
            if task_str.is_empty() { vec![] } else { vec!["--message".to_string(), task_str] },
            "Aider".to_string(),
        ),
        "goose" => (
            "goose".to_string(),
            if task_str.is_empty() { vec![] } else { vec!["run".to_string(), task_str] },
            "Block Goose".to_string(),
        ),
        other => (
            other.to_string(),
            vec![task_str],
            "Custom agent".to_string(),
        ),
    }
}
