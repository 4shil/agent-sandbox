use anyhow::Result;
use std::path::Path;
use crate::recorder;

pub fn show_diff(session_path: &str) -> Result<()> {
    let path = Path::new(session_path);
    
    if !path.exists() {
        // Try looking in .agent-sandbox/sessions/
        let home = std::env::var("HOME")?;
        let alt_path = Path::new(&home)
            .join(".agent-sandbox")
            .join("sessions")
            .join(session_path);
        
        if alt_path.exists() {
            return show_diff(&alt_path.to_string_lossy());
        }
        
        anyhow::bail!("Session not found: {}", session_path);
    }

    let session = recorder::load_session(path)?;
    
    println!("📊 Session: {}", session.id);
    println!("   Agent: {}", session.agent);
    println!("   Task: {}", session.task);
    println!("   Actions: {}", session.actions.len());
    println!();

    let mut file_changes: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();

    for action in &session.actions {
        if action.action_type == "file_write" {
            if let Some(path) = action.data.get("path").and_then(|v| v.as_str()) {
                if let Some(content) = action.data.get("content").and_then(|v| v.as_str()) {
                    file_changes.entry(path.to_string()).or_default().push(content.to_string());
                }
            }
        }
    }

    if file_changes.is_empty() {
        println!("(no file changes recorded)");
    } else {
        for (path, changes) in &file_changes {
            println!("📁 {}", path);
            for (i, content) in changes.iter().enumerate() {
                println!("  Change {}:", i + 1);
                for line in content.lines() {
                    println!("  + {}", line);
                }
            }
            println!();
        }
    }

    Ok(())
}
