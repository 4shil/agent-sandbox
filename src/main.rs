mod db;
mod sandbox;
mod recorder;
mod session;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::SystemTime;
use colored::Colorize;

use crate::sandbox::SandboxFs;
use crate::recorder::Recorder;

#[derive(Parser)]
#[command(name = "abox")]
#[command(about = "🛡️  Transparent sandbox for AI coding agents")]
#[command(long_about = "Launch any AI agent inside an isolated sandbox.\n\n  abox claude          launch Claude\n  abox opencode        launch OpenCode\n  abox list            show recorded sessions\n  abox replay <id>     replay a session")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Agent to launch (when no subcommand)
    agent: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Launch agent in sandbox
    Run {
        /// Agent to launch
        agent: String,
    },
    /// List recorded sessions
    #[command(alias = "ls")]
    List,
    /// Show session details
    Inspect {
        /// Session ID or workspace name
        id: String,
    },
    /// Replay session in terminal
    Replay {
        /// Session ID or workspace name
        id: String,
    },
    /// Export session as tar.gz
    Export {
        /// Session ID or workspace name
        id: String,

        /// Output file
        #[arg(short, long, default_value = "session.tar.gz")]
        output: String,
    },
    /// Import a shared session
    Import {
        /// Path to session archive
        file: String,
    },
    /// Clean old sessions
    Clean {
        /// Delete sessions older than N days
        #[arg(short, long, default_value = "30")]
        days: u64,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Some(cmd) => match cmd {
            Commands::Run { agent } => run_agent(agent)?,
            Commands::List => list_sessions()?,
            Commands::Inspect { id } => inspect_session(id)?,
            Commands::Replay { id } => replay_session(id)?,
            Commands::Export { id, output } => export_session(id, output)?,
            Commands::Import { file } => import_session(file)?,
            Commands::Clean { days } => clean_sessions(*days)?,
        },
        None => {
            let agent = cli.agent.unwrap_or_default();
            if agent.is_empty() {
                eprintln!("Usage: abox <agent>");
                eprintln!("       abox run <agent>");
                eprintln!("       abox list");
                std::process::exit(1);
            }
            run_agent(&agent)?;
        }
    }

    Ok(())
}

fn run_agent(agent: &str) -> Result<()> {
    let home = std::env::var("HOME")?;
    let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
    let sandbox_name = format!("{}-{}", agent, timestamp);
    let workspace = PathBuf::from(&home)
        .join(".agent-sandbox")
        .join("workspaces")
        .join(&sandbox_name);
    
    std::fs::create_dir_all(&workspace)?;
    std::fs::create_dir_all(workspace.join("logs"))?;

    let sfs = SandboxFs::new(&workspace)?;
    let _ = sfs.mount();

    let logs_dir = workspace.join("logs");
    let mut recorder = Recorder::new(&sandbox_name, agent, "interactive", &logs_dir)?;

    let mut cmd = Command::new(agent);
    cmd.current_dir(sfs.agent_root());
    cmd.envs(std::env::vars());
    cmd.stdin(Stdio::inherit());
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());

    let start = SystemTime::now();
    let status = cmd.status();
    let duration = start.elapsed()?.as_millis() as u64;

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

    if let Ok(Some(code)) = status.map(|s| s.code()) {
        std::process::exit(code);
    }

    Ok(())
}

fn get_workspaces_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".agent-sandbox").join("workspaces")
}

fn list_sessions() -> Result<()> {
    let dir = get_workspaces_dir();

    if !dir.exists() {
        println!("\n  No sessions yet.\n");
        return Ok(());
    }

    let mut entries: Vec<_> = std::fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|f| f.is_dir()).unwrap_or(false))
        .collect();
    
    entries.sort_by(|a, b| b.file_name().cmp(&a.file_name()));

    println!();
    println!("  {:<40} {:>10}", "SESSION".bold(), "FILES".bold());
    println!("  {}", "─".repeat(52).dimmed());

    for entry in &entries {
        let name = entry.file_name().to_string_lossy().to_string();
        let logs_dir = entry.path().join("logs");
        let session_count = std::fs::read_dir(&logs_dir)
            .map(|d| d.filter_map(|e| e.ok()).count())
            .unwrap_or(0);
        
        println!("  {:<40} {:>10}", name.cyan(), session_count);
    }
    println!();
    Ok(())
}

fn inspect_session(id: &str) -> Result<()> {
    let path = find_session(id)?;
    let content = std::fs::read_to_string(&path)?;
    let session: recorder::SessionRecord = serde_json::from_str(&content)?;

    println!();
    println!("  {:<15} {}", "ID".bold(), session.id.cyan());
    println!("  {:<15} {}", "Agent".bold(), session.agent.green());
    if let Some(dur) = session.duration_ms {
        println!("  {:<15} {}", "Duration".bold(), format!("{:.1}s", dur as f64 / 1000.0).yellow());
    }
    println!("  {:<15} {}", "Actions".bold(), session.actions.len());

    // Action breakdown
    let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for action in &session.actions {
        *counts.entry(action.action_type.clone()).or_default() += 1;
    }

    if !counts.is_empty() {
        println!();
        println!("  {}", "Actions".bold());
        for (t, c) in &counts {
            println!("    {:<20} {}", t.cyan(), c);
        }
    }

    println!();
    Ok(())
}

fn replay_session(id: &str) -> Result<()> {
    use std::io::{self, Write};
    
    let path = find_session(id)?;
    let content = std::fs::read_to_string(&path)?;
    let session: recorder::SessionRecord = serde_json::from_str(&content)?;

    println!();
    println!("🔄 {}", "Session Replay".bold());
    println!("   Agent: {} | Actions: {}", session.agent.green(), session.actions.len());
    println!();

    if session.actions.is_empty() {
        println!("  (no actions)");
        return Ok(());
    }

    let mut current = 0;
    let total = session.actions.len();

    loop {
        let action = &session.actions[current];
        
        println!("┌─ [{}/{}] ──────────────────────────────", current + 1, total);
        println!("│ Type: {}", action.action_type.cyan());
        println!("├─────────────────────────────────────────");
        
        // Pretty print data
        if let Some(obj) = action.data.as_object() {
            for (k, v) in obj {
                let val = if v.is_string() {
                    v.as_str().unwrap_or("").to_string()
                } else {
                    v.to_string()
                };
                if val.len() > 100 {
                    println!("│ {}: {}...", k.dimmed(), &val[..100]);
                } else {
                    println!("│ {}: {}", k.dimmed(), val);
                }
            }
        }
        
        println!("└─────────────────────────────────────────\n");

        print!("[n]ext [p]rev [d]etails [q]uit > ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        match input.trim().to_lowercase().as_str() {
            "n" | "" => { if current + 1 < total { current += 1; } }
            "p" => { if current > 0 { current -= 1; } }
            "d" => println!("{}\n", serde_json::to_string_pretty(&action.data)?),
            "q" => break,
            _ => {}
        }
    }

    Ok(())
}

fn export_session(id: &str, output: &str) -> Result<()> {
    use std::io::Write;
    
    let path = find_session(id)?;
    let session_dir = path.parent().unwrap().parent().unwrap();
    let session_name = session_dir.file_name().unwrap().to_string_lossy();

    println!();
    println!("📦 Exporting session...");

    // Find session.json
    let logs_dir = session_dir.join("logs");
    let session_file = find_session(id)?;

    // Create tar.gz using tar command (simpler than Rust tar crate)
    let status = Command::new("tar")
        .arg("-czf")
        .arg(output)
        .arg("-C")
        .arg(&session_dir)
        .arg(".")
        .status()?;

    if status.success() {
        let size = std::fs::metadata(output)?.len();
        println!("   {} → {} ({})", session_name.cyan(), output.green(), format_bytes(size));
        println!();
    } else {
        anyhow::bail!("Failed to create archive");
    }

    Ok(())
}

fn import_session(file: &str) -> Result<()> {
    let home = std::env::var("HOME")?;
    let import_dir = PathBuf::from(&home).join(".agent-sandbox").join("imports");
    std::fs::create_dir_all(&import_dir)?;

    println!();
    println!("📥 Importing session...");

    Command::new("tar")
        .arg("-xzf")
        .arg(file)
        .arg("-C")
        .arg(&import_dir)
        .status()?;

    // Find session.json
    let session_json = import_dir.join("session.json");
    if session_json.exists() {
        let content = std::fs::read_to_string(&session_json)?;
        let session: recorder::SessionRecord = serde_json::from_str(&content)?;
        println!("   Agent: {}", session.agent.cyan());
        println!("   Actions: {}", session.actions.len().to_string().green());
    }

    println!("   Imported to: {}", import_dir.display());
    println!();
    Ok(())
}

fn clean_sessions(days: u64) -> Result<()> {
    let dir = get_workspaces_dir();
    if !dir.exists() { return Ok(()); }

    let cutoff = std::time::SystemTime::now() - std::time::Duration::from_secs(days * 86400);
    let mut removed = 0;

    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        if let Ok(metadata) = entry.metadata() {
            if let Ok(modified) = metadata.modified() {
                if modified < cutoff {
                    std::fs::remove_dir_all(entry.path())?;
                    removed += 1;
                }
            }
        }
    }

    println!("🗑️  Removed {} old sessions (older than {} days)", removed, days);
    Ok(())
}

fn find_session(id: &str) -> Result<PathBuf> {
    let dir = get_workspaces_dir();
    
    // Direct path
    if std::path::Path::new(id).exists() {
        return Ok(PathBuf::from(id));
    }

    // Search all workspaces
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let logs = entry.path().join("logs");
        
        // Exact session ID match
        let exact = logs.join(format!("{}.json", id));
        if exact.exists() { return Ok(exact); }
        
        // Workspace name match → latest session
        if entry.file_name().to_string_lossy() == id {
            if let Ok(entries) = std::fs::read_dir(&logs) {
                let mut jsons: Vec<_> = entries
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("json"))
                    .collect();
                if !jsons.is_empty() {
                    jsons.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
                    return Ok(jsons[0].path());
                }
            }
        }
    }

    anyhow::bail!("Session not found: {}", id)
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.1}MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1}KB", bytes as f64 / 1024.0)
    } else {
        format!("{}B", bytes)
    }
}
