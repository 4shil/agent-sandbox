use anyhow::Result;
use colored::Colorize;
use std::io::{self, Write};
use std::path::PathBuf;

use crate::recorder;

pub fn get_workspaces_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".agent-sandbox").join("workspaces")
}

pub fn has_sessions() -> bool {
    let dir = get_workspaces_dir();
    dir.exists() && std::fs::read_dir(&dir).map(|d| d.count() > 0).unwrap_or(false)
}

pub fn list_sessions() -> Result<()> {
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

pub fn inspect_session(id: &str) -> Result<()> {
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

pub fn replay_session(id: &str) -> Result<()> {
    let path = find_session(id)?;
    let content = std::fs::read_to_string(&path)?;
    let session: recorder::SessionRecord = serde_json::from_str(&content)?;

    println!();
    println!("  {}", "🔄 Session Replay".bold());
    println!("  Agent: {} | Actions: {}", session.agent.green(), session.actions.len());
    println!();

    if session.actions.is_empty() {
        println!("  (no actions)");
        return Ok(());
    }

    let mut current = 0;
    let total = session.actions.len();

    loop {
        let action = &session.actions[current];
        
        println!("  ┌─ [{}/{}]", current + 1, total);
        println!("  │ {}", action.action_type.cyan());
        println!("  ├─────────────────────────────");
        
        if let Some(obj) = action.data.as_object() {
            for (k, v) in obj.iter().take(5) {
                let val = v.as_str().map(|s| s.to_string()).unwrap_or_else(|| v.to_string());
                let display = if val.len() > 80 { format!("{}...", &val[..80]) } else { val };
                println!("  │ {}: {}", k.dimmed(), display);
            }
        }
        
        println!("  └─────────────────────────────\n");

        print!("  [n]ext  [p]rev  [d]etails  [q]uit > ");
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

pub fn export_session(id: &str, output: &str) -> Result<()> {
    let path = find_session(id)?;
    let session_dir = path.parent().unwrap().parent().unwrap();

    println!();
    println!("  📦 Exporting...");

    let status = std::process::Command::new("tar")
        .arg("-czf")
        .arg(output)
        .arg("-C")
        .arg(&session_dir)
        .arg(".")
        .status()?;

    if status.success() {
        let size = std::fs::metadata(output)?.len();
        println!("  {} → {}", "Done".green(), output.cyan());
        println!("  Size: {}", format_size(size));
    } else {
        anyhow::bail!("Failed to create archive");
    }

    println!();
    Ok(())
}

pub fn import_session(file: &str) -> Result<()> {
    let home = std::env::var("HOME")?;
    let import_dir = PathBuf::from(&home).join(".agent-sandbox").join("imports");
    std::fs::create_dir_all(&import_dir)?;

    println!();
    println!("  📥 Importing...");

    std::process::Command::new("tar")
        .arg("-xzf")
        .arg(file)
        .arg("-C")
        .arg(&import_dir)
        .status()?;

    let session_json = import_dir.join("session.json");
    if session_json.exists() {
        let content = std::fs::read_to_string(&session_json)?;
        let session: recorder::SessionRecord = serde_json::from_str(&content)?;
        println!("  Agent: {}", session.agent.cyan());
        println!("  Actions: {}", session.actions.len());
    }

    println!("  Location: {}", import_dir.display());
    println!();
    Ok(())
}

pub fn clean_sessions(days: u64) -> Result<()> {
    let dir = get_workspaces_dir();
    if !dir.exists() { return Ok(()); }

    let cutoff = std::time::SystemTime::now() - std::time::Duration::from_secs(days * 86400);
    let mut removed = 0;
    let mut freed = 0u64;

    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        if let Ok(metadata) = entry.metadata() {
            if let Ok(modified) = metadata.modified() {
                if modified < cutoff {
                    freed += dir_size(&entry.path());
                    std::fs::remove_dir_all(entry.path())?;
                    removed += 1;
                }
            }
        }
    }

    println!();
    if removed == 0 {
        println!("  No sessions older than {} days.", days);
    } else {
        println!("  {} Removed {} session{}, freed {}", "✓".green(), removed, if removed == 1 { "" } else { "s" }, format_size(freed));
    }
    println!();
    Ok(())
}

fn find_session(id: &str) -> Result<PathBuf> {
    let dir = get_workspaces_dir();
    
    if std::path::Path::new(id).exists() {
        return Ok(PathBuf::from(id));
    }

    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let logs = entry.path().join("logs");
        
        let exact = logs.join(format!("{}.json", id));
        if exact.exists() { return Ok(exact); }
        
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

fn dir_size(path: &PathBuf) -> u64 {
    let mut total = 0;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.filter_map(|e| e.ok()) {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_dir() {
                    total += dir_size(&entry.path());
                } else {
                    total += metadata.len();
                }
            }
        }
    }
    total
}

fn format_size(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}
