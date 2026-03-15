use anyhow::Result;
use colored::Colorize;
use std::io::{self, Write};

const KNOWN_AGENTS: &[(&str, &str, &str)] = &[
    ("claude", "Claude Code", "npm i -g @anthropic-ai/claude-code"),
    ("codex", "OpenAI Codex CLI", "npm i -g @openai/codex"),
    ("opencode", "OpenCode", "npm i -g opencode-ai"),
    ("gemini", "Google Gemini CLI", "npm i -g @google/gemini-cli"),
    ("aider", "Aider", "pip install aider"),
    ("goose", "Block Goose", "pip install goose-ai"),
];

/// Show quick help when no args provided
pub fn show_quick_help() {
    println!();
    println!("  {}", "🛡️ abox".bold());
    println!("  {}", "Transparent sandbox for AI coding agents".dimmed());
    println!();
    println!("  {} {}", "Usage:".bold(), "abox <agent>".cyan());
    println!();
    println!("  Common agents:");
    for (name, desc, _) in KNOWN_AGENTS {
        let installed = which::which(name).is_ok();
        let status = if installed { "✅".to_string() } else { "  ".dimmed().to_string() };
        println!("    {} {:<12} {}", status, name.cyan(), desc.dimmed());
    }
    println!();
    println!("  Other commands:");
    println!("    {:<15} {}", "abox init".cyan(), "first-run setup wizard".dimmed());
    println!("    {:<15} {}", "abox list".cyan(), "show sessions".dimmed());
    println!("    {:<15} {}", "abox dashboard".cyan(), "interactive dashboard".dimmed());
    println!("    {:<15} {}", "abox status".cyan(), "quick status".dimmed());
    println!();
}

/// Show first-run tip
pub fn show_first_run_tip(agent: &str, sandbox_name: &str) {
    println!();
    println!("  {}", "💡 First session!".yellow().bold());
    println!("  Agent: {}", agent.cyan());
    println!("  Sandbox: {}", sandbox_name.dimmed());
    println!("  Everything is recorded. Work normally!");
    println!("  Use {} to see sessions later.", "abox list".cyan());
    println!();
}

/// Show session complete summary
pub fn show_session_complete(sandbox_name: &str, duration_ms: u64, session_id: &str) {
    let secs = duration_ms as f64 / 1000.0;
    let time_str = if secs < 60.0 {
        format!("{:.0}s", secs)
    } else {
        format!("{:.0}m {:.0}s", secs / 60.0, secs % 60.0)
    };
    
    println!();
    println!("  {} Session saved", "✓".green());
    println!("  {} {} {}", "ID:".dimmed(), session_id[..8].cyan(), "...".dimmed());
    println!("  {} {}", "Time:".dimmed(), time_str.dimmed());
    println!();
}

/// Show agent not found with install help
pub fn show_agent_not_found(agent: &str) {
    println!();
    println!("  {} {} not found", "❌".red(), agent.red().bold());
    
    // Check if we know how to install it
    for (name, desc, install_cmd) in KNOWN_AGENTS {
        if *name == agent {
            println!("  {}", desc.dimmed());
            println!("  Install with: {}", install_cmd.green());
            println!();
            return;
        }
    }
    
    // Unknown agent
    println!("  {}", "Unknown agent. Install it and make sure it's in your PATH.".dimmed());
    println!("  {} {}", "Known agents:".dimmed(), KNOWN_AGENTS.iter().map(|(n, _, _)| *n).collect::<Vec<_>>().join(", ").cyan());
    println!();
}

/// First-run setup wizard
pub fn run_wizard() -> Result<()> {
    println!();
    println!("  {}", "🛡️ abox setup wizard".bold());
    println!("  {}", "Let's get you started".dimmed());
    println!();
    println!("  {}", "─── Detecting agents ───".dimmed());
    println!();

    let mut found = Vec::new();
    for (name, desc, install) in KNOWN_AGENTS {
        if which::which(name).is_ok() {
            println!("    {} {} ({})", "✅".green(), name.cyan(), desc);
            found.push(*name);
        } else {
            println!("    {} {} — {}", "  ".dimmed(), name.dimmed(), install.dimmed());
        }
    }

    println!();
    if found.is_empty() {
        println!("  {}", "No agents found. Install one first!".yellow());
        println!();
        println!("  Recommended:");
        println!("    npm i -g opencode-ai    {}", "(free)".green());
    } else {
        println!("  Found {} agent{}: {}", found.len(), if found.len() == 1 { "" } else { "s" }, found.join(", ").cyan());
    }

    // Ask for default
    println!();
    print!("  Default agent? [{}] > ", found.first().unwrap_or(&"claude").cyan());
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let default_agent = input.trim();
    let default_agent = if default_agent.is_empty() {
        found.first().unwrap_or(&"claude").to_string()
    } else {
        default_agent.to_string()
    };

    // Save config
    let home = std::env::var("HOME")?;
    let config_path = format!("{}/.aboxrc", home);
    let config = format!(r#"# abox config (auto-generated)
ABOX_DEFAULT_AGENT="{}"
ABOX_MEMORY_LIMIT=""
ABOX_TIMEOUT=""
"#, default_agent);
    std::fs::write(&config_path, config)?;

    println!();
    println!("  {} Config saved to {}", "✓".green(), config_path.dimmed());
    println!();
    println!("  You're ready! Try:");
    println!("    {}", format!("abox {}", default_agent).cyan());
    println!();

    Ok(())
}

/// Show agent status
pub fn show_status() -> Result<()> {
    let home = std::env::var("HOME")?;
    let workspaces_dir = format!("{}/.agent-sandbox/workspaces", home);
    let imports_dir = format!("{}/.agent-sandbox/imports", home);

    // Count sessions
    let session_count = std::fs::read_dir(&workspaces_dir)
        .map(|d| d.filter_map(|e| e.ok()).count())
        .unwrap_or(0);

    // Calculate disk usage
    let disk_usage = get_dir_size(&workspaces_dir) + get_dir_size(&imports_dir);
    let disk_str = format_size(disk_usage);

    // Find last session
    let last_session = std::fs::read_dir(&workspaces_dir)
        .ok()
        .and_then(|d| {
            let mut entries: Vec<_> = d.filter_map(|e| e.ok()).collect();
            entries.sort_by(|a, b| {
                let a_time = a.metadata().and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                let b_time = b.metadata().and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                b_time.cmp(&a_time)
            });
            entries.first().map(|e| e.file_name().to_string_lossy().to_string())
        });

    // Check for .aboxrc config
    let config_path = format!("{}/.aboxrc", home);
    let has_config = std::path::Path::new(&config_path).exists();

    println!();
    println!("  {}", "🛡️ abox status".bold());
    println!();
    println!("  {:<15} {}", "Sessions:".bold(), session_count);
    println!("  {:<15} {}", "Disk used:".bold(), disk_str);
    if let Some(last) = last_session {
        println!("  {:<15} {}", "Last session:".bold(), last.cyan());
    }
    let config_display = if has_config { config_path.green().to_string() } else { "(none)".dimmed().to_string() };
    println!("  {:<15} {}", "Config:".bold(), config_display);
    
    // Check for old sessions needing cleanup
    let old_count = count_old_sessions(&workspaces_dir, 30);
    if old_count > 0 {
        println!();
        println!("  {} {} sessions older than 30 days", "💡".yellow(), old_count);
        println!("  Run {} to free space", "abox clean".cyan());
    }
    
    println!();
    Ok(())
}

/// Interactive dashboard
pub fn show_dashboard() -> Result<()> {
    let home = std::env::var("HOME")?;
    let workspaces_dir = format!("{}/.agent-sandbox/workspaces", home);

    loop {
        // Clear screen
        print!("\x1B[2J\x1B[1;1H");
        
        let session_count = std::fs::read_dir(&workspaces_dir)
            .map(|d| d.filter_map(|e| e.ok()).count())
            .unwrap_or(0);

        let disk_usage = get_dir_size(&workspaces_dir);
        let old_count = count_old_sessions(&workspaces_dir, 30);

        println!();
        println!("  ┌──────────────────────────────────────────────┐");
        println!("  │  {}                                        │", "🛡️ abox dashboard".bold());
        println!("  ├──────────────────────────────────────────────┤");
        println!("  │  {:<20} {:>20}  │", "Sessions".dimmed(), session_count);
        println!("  │  {:<20} {:>20}  │", "Disk used".dimmed(), format_size(disk_usage));
        println!("  │  {:<20} {:>20}  │", "Old sessions".dimmed(), if old_count > 0 { format!("{} ⚠️", old_count).yellow().to_string() } else { "none".to_string() });
        println!("  └──────────────────────────────────────────────┘");
        println!();

        // Recent sessions
        println!("  {}", "Recent sessions:".bold());
        if let Ok(entries) = std::fs::read_dir(&workspaces_dir) {
            let mut entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
            entries.sort_by(|a, b| {
                let a_time = a.metadata().and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                let b_time = b.metadata().and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                b_time.cmp(&a_time)
            });
            
            for entry in entries.iter().take(5) {
                let name = entry.file_name().to_string_lossy().to_string();
                let logs = entry.path().join("logs");
                let count = std::fs::read_dir(&logs).map(|d| d.filter_map(|e| e.ok()).count()).unwrap_or(0);
                println!("    {:<35} {} sessions", name.cyan(), count);
            }
        }
        
        println!();
        println!("  [L]ist  [C]lean  [R]un  [Q]uit");
        println!();

        print!("  > ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        match input.trim().to_lowercase().as_str() {
            "l" | "list" => {
                println!();
                drop(super::session::list_sessions());
                println!("\nPress Enter to continue...");
                let _ = io::stdin().read_line(&mut String::new());
            }
            "c" | "clean" => {
                println!();
                drop(super::session::clean_sessions(30));
                println!("\nPress Enter to continue...");
                let _ = io::stdin().read_line(&mut String::new());
            }
            "r" | "run" => {
                println!();
                println!("  Exit dashboard and run an agent:");
                print!("  Agent name > ");
                io::stdout().flush()?;
                let mut agent = String::new();
                io::stdin().read_line(&mut agent)?;
                let agent = agent.trim();
                if !agent.is_empty() {
                    return Ok(()); // Return, main will handle agent launch
                }
            }
            "q" | "quit" | "" => break,
            _ => {}
        }
    }

    println!();
    Ok(())
}

fn get_dir_size(path: &str) -> u64 {
    let mut total = 0;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.filter_map(|e| e.ok()) {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_dir() {
                    total += get_dir_size(&entry.path().to_string_lossy());
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

fn count_old_sessions(workspaces_dir: &str, days: u64) -> usize {
    let cutoff = std::time::SystemTime::now() - std::time::Duration::from_secs(days * 86400);
    
    std::fs::read_dir(workspaces_dir)
        .map(|d| {
            d.filter_map(|e| e.ok())
                .filter(|e| {
                    e.metadata()
                        .and_then(|m| m.modified())
                        .map(|t| t < cutoff)
                        .unwrap_or(false)
                })
                .count()
        })
        .unwrap_or(0)
}
