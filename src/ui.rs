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

const LOGO: &str = r#"
    _____
   /     \
  | () () |    abox
   \  ^  /    ─────────────────────
    |||||     sandbox for ai agents
    |||||
"#;

const DASHBOARD_HEADER: &str = r#"
  +========================================+
  |   abox  /  session dashboard           |
  +========================================+"#;

/// Show quick help when no args provided
pub fn show_quick_help() {
    println!("{}", LOGO.dimmed());
    println!("  Usage: abox <agent>");
    println!();
    println!("  Detected agents:");
    for (name, desc, _) in KNOWN_AGENTS {
        let installed = which::which(name).is_ok();
        let mark = if installed { "✓" } else { " " };
        println!("    [{}] {:<12} {}", 
            mark.green(), 
            name.cyan(), 
            desc.dimmed());
    }
    println!();
    println!("  Commands:");
    println!("    abox init           setup wizard");
    println!("    abox list           show sessions");
    println!("    abox dashboard      interactive dashboard");
    println!("    abox status         quick status");
    println!("    abox --help         full help");
    println!();
}

/// Show first-run tip
pub fn show_first_run_tip(agent: &str, sandbox_name: &str) {
    println!();
    println!("  ┌──────────────────────────────────────────┐");
    println!("  │  First session! Everything is recorded.   │");
    println!("  │  Agent: {:<32} │", agent.cyan());
    println!("  │  Run 'abox list' to see sessions later.  │");
    println!("  └──────────────────────────────────────────┘");
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
    println!("  [+] Session saved");
    println!("  | ID:     {}...", &session_id[..8].cyan());
    println!("  | Time:   {}", time_str.dimmed());
    println!("  | Use 'abox inspect {}' to view details", &session_id[..8]);
    println!();
}

/// Show agent not found with install help
pub fn show_agent_not_found(agent: &str) {
    println!();
    println!("  [!] Agent not found: {}", agent.red().bold());
    
    for (name, desc, install_cmd) in KNOWN_AGENTS {
        if *name == agent {
            println!("  | {}", desc.dimmed());
            println!("  | Install: {}", install_cmd.green());
            println!();
            return;
        }
    }
    
    println!("  | Unknown agent. Install it and ensure it's in PATH");
    println!("  | Known: {}", KNOWN_AGENTS.iter().map(|(n, _, _)| *n).collect::<Vec<_>>().join(", ").cyan());
    println!();
}

/// First-run setup wizard
pub fn run_wizard() -> Result<()> {
    println!();
    println!("  +========================================+");
    println!("  |   abox setup wizard                    |");
    println!("  +========================================+");
    println!();

    let mut found = Vec::new();
    for (name, desc, install) in KNOWN_AGENTS {
        if which::which(name).is_ok() {
            println!("    [✓] {:<12} {}", name.cyan(), desc.dimmed());
            found.push(*name);
        } else {
            println!("    [ ] {:<12} {}", name.dimmed(), format!("install: {}", install).dimmed());
        }
    }

    println!();
    if found.is_empty() {
        println!("  No agents found. Install one first:");
        println!("    npm i -g opencode-ai  (free)");
    } else {
        println!("  Found {} agent(s): {}", found.len(), found.join(", ").cyan());
    }

    println!();
    print!("  Default agent [{}] > ", found.first().unwrap_or(&"claude").cyan());
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let default_agent = input.trim();
    let default_agent = if default_agent.is_empty() {
        found.first().unwrap_or(&"claude").to_string()
    } else {
        default_agent.to_string()
    };

    let home = std::env::var("HOME")?;
    let config_path = format!("{}/.aboxrc", home);
    let config = format!(r#"# abox config
ABOX_DEFAULT_AGENT="{}"
ABOX_MEMORY_LIMIT=""
ABOX_TIMEOUT=""
"#, default_agent);
    std::fs::write(&config_path, config)?;

    println!();
    println!("  [+] Config saved to {}", config_path.dimmed());
    println!("  | Try: {}", format!("abox {}", default_agent).cyan());
    println!();

    Ok(())
}

/// Show agent status
pub fn show_status() -> Result<()> {
    let home = std::env::var("HOME")?;
    let workspaces_dir = format!("{}/.agent-sandbox/workspaces", home);

    let session_count = std::fs::read_dir(&workspaces_dir)
        .map(|d| d.filter_map(|e| e.ok()).count())
        .unwrap_or(0);

    let disk_usage = get_dir_size(&workspaces_dir);
    let disk_str = format_size(disk_usage);

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

    let config_path = format!("{}/.aboxrc", home);
    let has_config = std::path::Path::new(&config_path).exists();

    println!();
    println!("  +==========================================+");
    println!("  |   abox status                            |");
    println!("  +==========================================+");
    println!("  |  Sessions:   {:>25} |", session_count);
    println!("  |  Disk used:  {:>25} |", disk_str);
    if let Some(last) = last_session {
        println!("  |  Last:       {:>25} |", &last[..last.len().min(25)]);
    }
    println!("  |  Config:     {:>25} |", if has_config { "exists" } else { "none" });
    println!("  +==========================================+");

    let old_count = count_old_sessions(&workspaces_dir, 30);
    if old_count > 0 {
        println!();
        println!("  [!] {} sessions older than 30 days", old_count);
        println!("  | Run 'abox clean' to free space");
    }
    
    println!();
    Ok(())
}

/// Interactive dashboard
pub fn show_dashboard() -> Result<()> {
    let home = std::env::var("HOME")?;
    let workspaces_dir = format!("{}/.agent-sandbox/workspaces", home);

    loop {
        print!("\x1B[2J\x1B[1;1H");
        
        let session_count = std::fs::read_dir(&workspaces_dir)
            .map(|d| d.filter_map(|e| e.ok()).count())
            .unwrap_or(0);

        let disk_usage = get_dir_size(&workspaces_dir);
        let old_count = count_old_sessions(&workspaces_dir, 30);

        println!();
        println!("  +========================================+");
        println!("  |   abox dashboard                       |");
        println!("  +========================================+");
        println!("  | Sessions:    {:>20} |", session_count);
        println!("  | Disk used:   {:>20} |", format_size(disk_usage));
        if old_count > 0 {
            println!("  | Old:         {:>20} |", format!("{} sessions", old_count));
        }
        println!("  +========================================+");
        println!();

        println!("  Recent sessions:");
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
                println!("  Press Enter to continue...");
                let _ = io::stdin().read_line(&mut String::new());
            }
            "c" | "clean" => {
                println!();
                drop(super::session::clean_sessions(30));
                println!("  Press Enter to continue...");
                let _ = io::stdin().read_line(&mut String::new());
            }
            "r" | "run" => {
                println!();
                print!("  Agent name > ");
                io::stdout().flush()?;
                let mut agent = String::new();
                io::stdin().read_line(&mut agent)?;
                let agent = agent.trim();
                if !agent.is_empty() {
                    return Ok(());
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

pub fn format_size(bytes: u64) -> String {
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
