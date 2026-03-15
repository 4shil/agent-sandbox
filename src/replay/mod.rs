use anyhow::Result;
use std::io::{self, Write};
use std::path::Path;
use crate::recorder::{self, SessionRecord, ActionRecord};

pub fn replay_session(path: &str) -> Result<()> {
    let session_path = if Path::new(path).exists() {
        Path::new(path).to_path_buf()
    } else {
        // Try finding in any workspace's logs directory
        let home = std::env::var("HOME")?;
        let workspaces_dir = Path::new(&home).join(".agent-sandbox").join("workspaces");
        
        // First try: path is a session ID in any workspace
        if let Ok(entries) = std::fs::read_dir(&workspaces_dir) {
            for entry in entries.flatten() {
                let logs_dir = entry.path().join("logs");
                let potential = logs_dir.join(format!("{}.json", path));
                if potential.exists() {
                    return replay_session(potential.to_str().unwrap_or(path));
                }
            }
        }
        
        // Second try: path is a workspace name
        let logs_dir = workspaces_dir.join(path).join("logs");
        if logs_dir.exists() {
            // Find the most recent session
            let mut sessions: Vec<_> = std::fs::read_dir(&logs_dir)?
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("json"))
                .collect();
            sessions.sort_by_key(|e| e.metadata().and_then(|m| m.modified()).ok());
            if let Some(latest) = sessions.last() {
                return replay_session(latest.path().to_str().unwrap_or(path));
            }
        }
        
        anyhow::bail!("Session not found: {}", path);
    };

    let session = recorder::load_session(&session_path)?;

    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  🔄 SESSION REPLAY                                       ║");
    println!("╠══════════════════════════════════════════════════════════╣");
    println!("║  ID:       {:<44} ║", &session.id[..session.id.len().min(44)]);
    println!("║  Agent:    {:<44} ║", session.agent);
    println!("║  Task:     {:<44} ║", &session.task[..session.task.len().min(44)]);
    println!("║  Actions:  {:<44} ║", session.actions.len());
    if let Some(duration) = session.duration_ms {
        println!("║  Duration: {:<44} ║", format!("{}ms", duration));
    }
    println!("╚══════════════════════════════════════════════════════════╝\n");

    if session.actions.is_empty() {
        println!("(no actions to replay)");
        return Ok(());
    }

    let mut current = 0;
    let total = session.actions.len();

    loop {
        let action = &session.actions[current];
        
        println!("┌─ [{}/{}] ─────────────────────────────────────────────", current + 1, total);
        println!("│ Type:      {}", action.action_type);
        println!("│ Timestamp: {}", action.timestamp);
        println!("├────────────────────────────────────────────────────────");

        match action.action_type.as_str() {
            "task_start" => {
                println!("│ Task started:");
                if let Some(task) = action.data.get("task").and_then(|v| v.as_str()) {
                    println!("│   {}", task);
                }
                if let Some(agent) = action.data.get("agent").and_then(|v| v.as_str()) {
                    println!("│   Agent: {}", agent);
                }
            }
            "file_write" => {
                if let Some(path) = action.data.get("path").and_then(|v| v.as_str()) {
                    println!("│ File: {}", path);
                }
                if let Some(content) = action.data.get("content").and_then(|v| v.as_str()) {
                    println!("│ Content:");
                    for line in content.lines().take(10) {
                        println!("│   + {}", line);
                    }
                    if content.lines().count() > 10 {
                        println!("│   ... ({} more lines)", content.lines().count() - 10);
                    }
                }
            }
            "file_read" => {
                if let Some(path) = action.data.get("path").and_then(|v| v.as_str()) {
                    println!("│ Read: {}", path);
                }
                if let Some(len) = action.data.get("content_length").and_then(|v| v.as_u64()) {
                    println!("│ Size: {} bytes", len);
                }
            }
            "exec" => {
                if let Some(cmd) = action.data.get("command").and_then(|v| v.as_str()) {
                    println!("│ Command: {}", cmd);
                }
                if let Some(code) = action.data.get("exit_code").and_then(|v| v.as_i64()) {
                    println!("│ Exit: {}", code);
                }
                if let Some(stdout) = action.data.get("stdout").and_then(|v| v.as_str()) {
                    if !stdout.is_empty() {
                        println!("│ stdout:");
                        for line in stdout.lines().take(5) {
                            println!("│   {}", line);
                        }
                    }
                }
                if let Some(stderr) = action.data.get("stderr").and_then(|v| v.as_str()) {
                    if !stderr.is_empty() {
                        println!("│ stderr:");
                        for line in stderr.lines().take(5) {
                            println!("│   ⚠ {}", line);
                        }
                    }
                }
            }
            "error" => {
                if let Some(msg) = action.data.get("message").and_then(|v| v.as_str()) {
                    println!("│ ❌ Error: {}", msg);
                }
            }
            _ => {
                println!("│ Data: {}", serde_json::to_string_pretty(&action.data)?);
            }
        }

        println!("└────────────────────────────────────────────────────────\n");

        print!("Actions: [n]ext, [p]rev, [j]ump, [d]etails, [q]uit > ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();

        match input.as_str() {
            "n" | "" => {
                if current + 1 < total {
                    current += 1;
                } else {
                    println!("(end of session)");
                }
            }
            "p" => {
                if current > 0 {
                    current -= 1;
                }
            }
            "j" => {
                print!("Jump to action # (1-{}): ", total);
                io::stdout().flush()?;
                let mut jump = String::new();
                io::stdin().read_line(&mut jump)?;
                if let Ok(idx) = jump.trim().parse::<usize>() {
                    if idx >= 1 && idx <= total {
                        current = idx - 1;
                    }
                }
            }
            "d" => {
                println!("\n📋 Full action data:");
                println!("{}", serde_json::to_string_pretty(&action.data)?);
                println!();
            }
            "q" => break,
            _ => println!("Unknown command: {}", input),
        }
    }

    Ok(())
}
