use anyhow::{Result, Context};
use std::path::PathBuf;
use std::process::Command;
use std::time::SystemTime;

use crate::db;
use crate::sandbox::SandboxFs;
use crate::recorder::Recorder;

pub fn run_agent(agent: &str, task: &str, sandbox_name: &str) -> Result<()> {
    // Get workspace path from DB
    let conn = db::get_connection()?;
    let workspace_path: String = conn.query_row(
        "SELECT workspace_path FROM sandboxes WHERE name = ?1",
        [sandbox_name],
        |row| row.get(0),
    ).context("Sandbox not found. Run 'agent-sandbox init' first.")?;

    let workspace = PathBuf::from(workspace_path);
    let logs_dir = workspace.join("logs");

    println!("🔌 Initializing sandbox filesystem...");
    let sfs = SandboxFs::new(&workspace)?;
    sfs.mount()?;

    println!("📝 Starting session recorder...");
    let mut recorder = Recorder::new(sandbox_name, agent, task, &logs_dir)?;

    println!("🤖 Launching {} with task: {}", agent, task);
    println!("   Session ID: {}", recorder.session_id());
    println!("   Workspace: {}", workspace.display());

    // Record task start
    recorder.record_action("task_start", serde_json::json!({
        "agent": agent,
        "task": task,
        "workspace": workspace.to_string_lossy(),
    }))?;

    let start = SystemTime::now();

    // Run the actual agent command
    let output = match agent {
        "claude" => Command::new("claude")
            .arg("--print")
            .arg(task)
            .current_dir(sfs.agent_root())
            .output(),
        "codex" => Command::new("codex")
            .arg(task)
            .current_dir(sfs.agent_root())
            .output(),
        _ => Command::new(agent)
            .arg(task)
            .current_dir(sfs.agent_root())
            .output(),
    };

    let duration = start.elapsed()?.as_millis() as u64;

    match output {
        Ok(result) => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            let stderr = String::from_utf8_lossy(&result.stderr);

            // Record exec
            recorder.record_exec(
                &format!("{} {}", agent, task),
                &stdout,
                &stderr,
                result.status.code().unwrap_or(-1),
                duration,
            )?;

            // Record modified files
            for file in sfs.modified_files()? {
                recorder.record_action("file_modified", serde_json::json!({
                    "path": file.strip_prefix(&workspace)?.to_string_lossy(),
                }))?;
            }

            println!("\n✅ Task completed in {:.1}s", duration as f64 / 1000.0);
            println!("   Exit code: {:?}", result.status.code());
            println!("   Session: {}", logs_dir.join(format!("{}.json", recorder.session_id())).display());

            if !stdout.is_empty() {
                println!("\n--- Output ---");
                println!("{}", stdout);
            }
            if !stderr.is_empty() {
                println!("\n--- Errors ---");
                println!("{}", stderr);
            }
        }
        Err(e) => {
            recorder.record_action("task_error", serde_json::json!({
                "error": e.to_string(),
            }))?;
            println!("❌ Failed to run {}: {}", agent, e);
        }
    }

    recorder.finish()?;
    Ok(())
}
