use anyhow::Result;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use tar::Archive;
use flate2::write::GzEncoder;
use flate2::Compression;

use crate::recorder;

pub fn export_session(session_path: &str, output: &str) -> Result<()> {
    let session_file = resolve_session_path(session_path)?;
    let session = recorder::load_session(&session_file)?;
    let session_dir = session_file.parent().unwrap_or(Path::new("."));

    println!("📦 Exporting session: {}", session.id);
    println!("   Agent: {}", session.agent);
    println!("   Task: {}", session.task);
    println!("   Actions: {}", session.actions.len());

    let tar_path = if output.ends_with(".tar.gz") {
        output.to_string()
    } else {
        format!("{}.tar.gz", output)
    };

    let file = File::create(&tar_path)?;
    let enc = GzEncoder::new(file, Compression::default());
    let mut tar = tar::Builder::new(enc);

    // Add session.json
    tar.append_path_with_name(&session_file, "session.json")?;

    // Add workspace files (excluding .sandbox-overlay)
    let workspace = session_dir.parent().unwrap_or(Path::new("."));
    add_workspace_files(&mut tar, workspace)?;

    // Add metadata
    let metadata = serde_json::json!({
        "exported_at": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION"),
        "session_id": session.id,
        "agent": session.agent,
        "task": session.task,
        "action_count": session.actions.len(),
        "duration_ms": session.duration_ms,
    });
    let metadata_json = serde_json::to_string_pretty(&metadata)?;
    let mut header = tar::Header::new_gnu();
    header.set_size(metadata_json.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    tar.append_data(&mut header, "metadata.json", metadata_json.as_bytes())?;

    // Generate and add HTML viewer
    let html = generate_html_viewer(&session)?;
    let mut header = tar::Header::new_gnu();
    header.set_size(html.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    tar.append_data(&mut header, "replay.html", html.as_bytes())?;

    tar.finish()?;

    let size = std::fs::metadata(&tar_path)?.len();
    println!("   Output: {} ({})", tar_path, format_bytes(size));
    println!("✅ Export complete");

    Ok(())
}

fn add_workspace_files<W: Write>(tar: &mut tar::Builder<W>, workspace: &Path) -> Result<()> {
    if !workspace.exists() {
        return Ok(());
    }

    for entry in walkdir::WalkDir::new(workspace).into_iter().filter_entry(|e| {
        let name = e.file_name().to_string_lossy();
        !name.starts_with('.') && name != "node_modules" && name != "target"
    }) {
        let entry = entry?;
        if entry.file_type().is_file() {
            let rel_path = entry.path().strip_prefix(workspace)?;
            tar.append_path_with_name(entry.path(), format!("files/{}", rel_path.display()))?;
        }
    }

    Ok(())
}

fn generate_html_viewer(session: &recorder::SessionRecord) -> Result<String> {
    let actions_json = serde_json::to_string(&session.actions)?;
    
    Ok(format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Session Replay - {}</title>
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{ font-family: 'SF Mono', 'Fira Code', monospace; background: #0d1117; color: #c9d1d9; }}
        .header {{ background: #161b22; padding: 20px; border-bottom: 1px solid #30363d; }}
        .header h1 {{ color: #58a6ff; font-size: 18px; }}
        .header p {{ color: #8b949e; margin-top: 5px; font-size: 14px; }}
        .container {{ display: flex; height: calc(100vh - 100px); }}
        .sidebar {{ width: 300px; background: #161b22; border-right: 1px solid #30363d; overflow-y: auto; }}
        .content {{ flex: 1; padding: 20px; overflow-y: auto; }}
        .action-item {{ padding: 10px 15px; border-bottom: 1px solid #21262d; cursor: pointer; }}
        .action-item:hover {{ background: #21262d; }}
        .action-item.active {{ background: #1f6feb33; border-left: 3px solid #58a6ff; }}
        .action-type {{ color: #7ee787; font-size: 12px; }}
        .action-time {{ color: #8b949e; font-size: 11px; }}
        .action-data {{ background: #161b22; padding: 15px; border-radius: 6px; margin-top: 10px; }}
        .action-data pre {{ white-space: pre-wrap; word-break: break-all; }}
        .controls {{ padding: 15px; background: #161b22; border-top: 1px solid #30363d; display: flex; gap: 10px; }}
        button {{ background: #21262d; color: #c9d1d9; border: 1px solid #30363d; padding: 8px 16px; border-radius: 6px; cursor: pointer; }}
        button:hover {{ background: #30363d; }}
        button.primary {{ background: #1f6feb; border-color: #1f6feb; }}
        .counter {{ color: #8b949e; padding: 8px; }}
    </style>
</head>
<body>
    <div class="header">
        <h1>🔄 Session Replay</h1>
        <p>Agent: {} | Task: {} | Actions: {}</p>
    </div>
    <div class="container">
        <div class="sidebar" id="sidebar"></div>
        <div class="content" id="content">
            <p>Select an action to view details</p>
        </div>
    </div>
    <div class="controls">
        <button onclick="prev()">← Prev</button>
        <button onclick="next()" class="primary">Next →</button>
        <span class="counter" id="counter">0 / {}</span>
    </div>
    <script>
        const actions = {};
        let current = 0;
        
        function renderSidebar() {{
            const sb = document.getElementById('sidebar');
            sb.innerHTML = actions.map((a, i) => `
                <div class="action-item ${{i === current ? 'active' : ''}}" onclick="goTo(${{i}})">
                    <div class="action-type">${{a.action_type}}</div>
                    <div class="action-time">#${{i + 1}} - ${{a.timestamp}}</div>
                </div>
            `).join('');
        }}
        
        function renderContent() {{
            const a = actions[current];
            const content = document.getElementById('content');
            content.innerHTML = `
                <h2 style="color: #7ee787; margin-bottom: 10px;">${{a.action_type}}</h2>
                <div class="action-data">
                    <pre>${{JSON.stringify(a.data, null, 2)}}</pre>
                </div>
            `;
            document.getElementById('counter').textContent = `${{current + 1}} / ${{actions.length}}`;
            renderSidebar();
        }}
        
        function next() {{ if (current < actions.length - 1) {{ current++; renderContent(); }} }}
        function prev() {{ if (current > 0) {{ current--; renderContent(); }} }}
        function goTo(i) {{ current = i; renderContent(); }}
        
        document.addEventListener('keydown', (e) => {{
            if (e.key === 'ArrowRight' || e.key === 'n') next();
            if (e.key === 'ArrowLeft' || e.key === 'p') prev();
        }});
        
        renderSidebar();
        if (actions.length > 0) renderContent();
    </script>
</body>
</html>"#, 
        session.id,
        session.agent, session.task, session.actions.len(),
        session.actions.len(),
        actions_json
    ))
}

fn resolve_session_path(path: &str) -> Result<PathBuf> {
    // Direct file path
    if Path::new(path).exists() {
        return Ok(Path::new(path).to_path_buf());
    }
    
    let home = std::env::var("HOME")?;
    let workspaces_dir = Path::new(&home).join(".agent-sandbox").join("workspaces");
    
    // First: search for exact session ID match in any workspace
    if let Ok(entries) = std::fs::read_dir(&workspaces_dir) {
        for entry in entries.flatten() {
            let logs_dir = entry.path().join("logs");
            if logs_dir.exists() {
                // Check for exact match
                let exact = logs_dir.join(format!("{}.json", path));
                if exact.exists() {
                    return Ok(exact);
                }
                // If path matches workspace name, return most recent session
                if let Ok(log_entries) = std::fs::read_dir(&logs_dir) {
                    let mut json_files: Vec<_> = log_entries
                        .filter_map(|e| e.ok())
                        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("json"))
                        .collect();
                    if !json_files.is_empty() {
                        json_files.sort_by(|a, b| {
                            b.file_name().cmp(&a.file_name())
                        });
                        if entry.file_name().to_string_lossy() == path {
                            return Ok(json_files[0].path());
                        }
                    }
                }
            }
        }
    }
    
    anyhow::bail!("Session not found: {}", path)
}

pub fn import_session(file: &str) -> Result<()> {
    let input_path = Path::new(file);
    if !input_path.exists() {
        anyhow::bail!("File not found: {}", file);
    }

    println!("📥 Importing session from: {}", file);

    let home = std::env::var("HOME")?;
    let import_dir = Path::new(&home).join(".agent-sandbox").join("imports");
    std::fs::create_dir_all(&import_dir)?;

    // Extract the tar.gz
    let file = File::open(input_path)?;
    let dec = flate2::read::GzDecoder::new(file);
    let mut archive = Archive::new(dec);
    archive.unpack(&import_dir)?;

    // Find session.json
    let session_json = import_dir.join("session.json");
    if session_json.exists() {
        let session = recorder::load_session(&session_json)?;
        println!("✅ Imported session:");
        println!("   ID: {}", session.id);
        println!("   Agent: {}", session.agent);
        println!("   Task: {}", session.task);
        println!("   Actions: {}", session.actions.len());
        println!("   Location: {}", import_dir.display());
    } else {
        println!("⚠️  No session.json found in archive");
    }

    Ok(())
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
