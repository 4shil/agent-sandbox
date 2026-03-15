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
use cli::{Cli, Commands};
use colored::*;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { name, template } => {
            println!();
            println!("{} {}", "🛡️  Creating sandbox".bold(), name.cyan().bold());
            db::init_workspace(&name, &format!("{:?}", template).to_lowercase())?;
            println!("{} {}", "✅".green(), format!("Sandbox '{}' ready", name).green());
            println!();
            println!("  {} {}", "cd".dimmed(), format!("~/.agent-sandbox/workspaces/{}", name).dimmed());
            println!("  {} {}", "abox run".dimmed(), "\"your task\"".dimmed());
            println!();
        }
        Commands::Status => {
            println!();
            println!("{}", "📦 Active Sandboxes".bold());
            println!();
            let sandboxes = db::list_sandboxes()?;
            if sandboxes.is_empty() {
                println!("  {}", "(no sandboxes — run `abox init <name>` to create one)".dimmed());
            } else {
                println!("  {:<20} {:<12} {}", "NAME".bold(), "AGENT".bold(), "CREATED".bold());
                println!("  {}", "─".repeat(50).dimmed());
                for sb in &sandboxes {
                    println!("  {:<20} {:<12} {}", 
                        sb.name.cyan(), 
                        sb.agent.green(),
                        sb.created_at.dimmed()
                    );
                }
                println!();
                println!("  {} {}", format!("{} sandbox{}", sandboxes.len(), if sandboxes.len() == 1 { "" } else { "es" }).dimmed(), 
                    "(run `abox replay <name>` to view history)".dimmed());
            }
            println!();
        }
        Commands::Run { agent, sandbox, task, memory, cpu, timeout, no_network, allow_domain } => {
            let sandbox_name = sandbox.unwrap_or_else(|| {
                db::list_sandboxes()
                    .ok()
                    .and_then(|s| s.into_iter().next().map(|sb| sb.name))
                    .unwrap_or_else(|| "default".to_string())
            });
            let limits = limits::ResourceLimits::from_args(
                memory.as_deref(),
                cpu.as_deref(),
                timeout.as_deref(),
                None,
            )?;
            let network_policy = network::NetworkPolicy::new(no_network, allow_domain);
            cli::run::run_agent(&agent, &task, &sandbox_name, &limits, &network_policy)?;
        }
        Commands::Diff { session } => {
            println!();
            println!("{}", "📊 Session Diff".bold());
            println!();
            cli::diff::show_diff(&session)?;
        }
        Commands::Replay { session } => {
            println!();
            println!("{}", "🔄 Session Replay".bold());
            println!();
            replay::replay_session(&session)?;
        }
        Commands::Export { session, output } => {
            println!();
            println!("{}", "📦 Exporting Session".bold());
            println!();
            export::export_session(&session, &output)?;
        }
        Commands::Import { file } => {
            println!();
            println!("{}", "📥 Importing Session".bold());
            println!();
            export::import_session(&file)?;
        }
        Commands::Inspect { session } => {
            println!();
            println!("{}", "🔍 Session Inspector".bold());
            println!();
            inspect_session(&session)?;
        }
    }

    Ok(())
}

fn inspect_session(session_path: &str) -> Result<()> {
    use std::path::Path;
    
    let path = if Path::new(session_path).exists() {
        Path::new(session_path).to_path_buf()
    } else {
        let home = std::env::var("HOME")?;
        let workspaces = Path::new(&home).join(".agent-sandbox").join("workspaces");
        
        // Search all workspaces for matching session or workspace name
        if let Ok(entries) = std::fs::read_dir(&workspaces) {
            let mut found = None;
            for entry in entries.flatten() {
                let logs = entry.path().join("logs");
                if !logs.exists() { continue; }
                
                // Exact session ID match
                let exact = logs.join(format!("{}.json", session_path));
                if exact.exists() {
                    found = Some(exact);
                    break;
                }
                
                // If path matches workspace name, get latest session
                if entry.file_name().to_string_lossy() == session_path {
                    if let Ok(log_entries) = std::fs::read_dir(&logs) {
                        let mut jsons: Vec<_> = log_entries
                            .filter_map(|e| e.ok())
                            .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("json"))
                            .collect();
                        if !jsons.is_empty() {
                            jsons.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
                            found = Some(jsons[0].path());
                            break;
                        }
                    }
                }
            }
            found.ok_or_else(|| anyhow::anyhow!("Session not found: {}", session_path))?
        } else {
            anyhow::bail!("Session not found: {}", session_path);
        }
    };

    let content = std::fs::read_to_string(&path)?;
    let session: recorder::SessionRecord = serde_json::from_str(&content)?;

    println!("  {} {}", "ID:".bold(), session.id.cyan());
    println!("  {} {}", "Agent:".bold(), session.agent.green());
    println!("  {} {}", "Task:".bold(), session.task);
    println!("  {} {}", "Sandbox:".bold(), session.sandbox_name.cyan());
    if let Some(dur) = session.duration_ms {
        println!("  {} {}", "Duration:".bold(), format!("{}ms", dur).yellow());
    }
    println!("  {} {}", "Actions:".bold(), session.actions.len().to_string().green());
    println!("  {} {}", "OS:".bold(), session.metadata.host_info.os.dimmed());
    println!("  {} {}", "Arch:".bold(), session.metadata.host_info.arch.dimmed());
    println!();

    // Action type breakdown
    let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for action in &session.actions {
        *counts.entry(action.action_type.clone()).or_default() += 1;
    }

    if !counts.is_empty() {
        println!("  {}", "Action Breakdown:".bold());
        let max_count = counts.values().max().unwrap_or(&1);
        for (action_type, count) in &counts {
            let bar_len = (*count * 20) / max_count;
            let bar = "█".repeat(bar_len);
            println!("    {:<20} {:>3} {}", action_type.cyan(), count, bar.dimmed());
        }
        println!();
    }

    Ok(())
}
