#![allow(dead_code)]

use anyhow::Result;
use colored::Colorize;
use std::collections::HashMap;
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::SystemTime;

use crate::recorder;
use crate::ui;

const KNOWN_AGENTS: &[&str] = &["claude", "codex", "opencode", "gemini", "aider", "goose"];

pub fn get_workspaces_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".agent-sandbox").join("workspaces")
}

pub fn get_meta_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let dir = PathBuf::from(home).join(".agent-sandbox").join("meta");
    std::fs::create_dir_all(&dir).ok();
    dir
}

pub fn has_sessions() -> bool {
    let dir = get_workspaces_dir();
    dir.exists() && std::fs::read_dir(&dir).map(|d| d.count() > 0).unwrap_or(false)
}

pub fn list_sessions() -> Result<()> {
    let dir = get_workspaces_dir();
    if !dir.exists() { println!("\n  No sessions.\n"); return Ok(()); }

    let mut entries: Vec<_> = std::fs::read_dir(&dir)?.filter_map(|e| e.ok()).collect();
    entries.sort_by(|a, b| b.file_name().cmp(&a.file_name()));

    println!();
    println!("  +--------------------------------------------+");
    println!("  |  {:<28} {:>12}    |", "SESSION".bold(), "FILES".bold());
    println!("  +--------------------------------------------+");

    for entry in &entries {
        let name = entry.file_name().to_string_lossy().to_string();
        let logs = entry.path().join("logs");
        let count = std::fs::read_dir(&logs).map(|d| d.filter_map(|e| e.ok()).count()).unwrap_or(0);
        println!("  |  {:<28} {:>12}    |", name.cyan(), count);
    }
    println!("  +--------------------------------------------+");
    println!();
    Ok(())
}

pub fn inspect_session(id: &str) -> Result<()> {
    let path = find_session(id)?;
    let content = std::fs::read_to_string(&path)?;
    let session: recorder::SessionRecord = serde_json::from_str(&content)?;

    // Load tags and notes
    let meta = get_meta_dir();
    let tags: HashMap<String, Vec<String>> = read_json(&meta.join("tags.json")).unwrap_or_default();
    let notes: HashMap<String, String> = read_json(&meta.join("notes.json")).unwrap_or_default();
    let session_tags = tags.get(&session.id).cloned().unwrap_or_default();
    let session_note = notes.get(&session.id).cloned().unwrap_or_default();

    println!();
    println!("  +--------------------------------------------+");
    println!("  |  Session Details                           |");
    println!("  +--------------------------------------------+");
    println!("  |  ID:       {:<31} |", &session.id[..31].cyan());
    println!("  |  Agent:    {:<31} |", session.agent.green());
    if let Some(dur) = session.duration_ms {
        println!("  |  Duration: {:<31} |", format!("{:.1}s", dur as f64 / 1000.0).yellow());
    }
    println!("  |  Actions:  {:<31} |", session.actions.len().to_string());
    if !session_tags.is_empty() {
        println!("  |  Tags:     {:<31} |", session_tags.join(", ").yellow());
    }
    if !session_note.is_empty() {
        println!("  |  Note:     {:<31} |", session_note.dimmed());
    }
    println!("  +--------------------------------------------+");

    let mut counts: HashMap<String, usize> = HashMap::new();
    for action in &session.actions {
        *counts.entry(action.action_type.clone()).or_default() += 1;
    }
    if !counts.is_empty() {
        println!("  Actions:");
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
    println!("  [~] Session Replay | Agent: {} | Actions: {}", session.agent.cyan(), session.actions.len());

    if session.actions.is_empty() { println!("  (empty)\n"); return Ok(()); }

    let mut current = 0;
    let total = session.actions.len();

    loop {
        let action = &session.actions[current];
        println!("  ┌─ [{}/{}]", current + 1, total);
        println!("  │ {}", action.action_type.cyan());
        println!("  ├──────────────────────────");
        if let Some(obj) = action.data.as_object() {
            for (k, v) in obj.iter().take(5) {
                let val = v.as_str().map(|s| s.to_string()).unwrap_or_else(|| v.to_string());
                let display = if val.len() > 60 { format!("{}...", &val[..60]) } else { val };
                println!("  │ {}: {}", k.dimmed(), display);
            }
        }
        println!("  └──────────────────────────\n");

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
    let dir = path.parent().unwrap().parent().unwrap();
    println!();
    println!("  [*] Exporting...");
    let status = std::process::Command::new("tar").arg("-czf").arg(output).arg("-C").arg(dir).arg(".").status()?;
    if status.success() {
        let size = std::fs::metadata(output)?.len();
        println!("  [+] {} ({})", output.cyan(), ui::format_size(size));
    } else {
        anyhow::bail!("Export failed");
    }
    println!();
    Ok(())
}

pub fn import_session(file: &str) -> Result<()> {
    let home = std::env::var("HOME")?;
    let dir = PathBuf::from(home).join(".agent-sandbox").join("imports");
    std::fs::create_dir_all(&dir)?;
    println!();
    println!("  [*] Importing...");
    std::process::Command::new("tar").arg("-xzf").arg(file).arg("-C").arg(&dir).status()?;
    if let Ok(content) = std::fs::read_to_string(dir.join("session.json")) {
        if let Ok(s) = serde_json::from_str::<recorder::SessionRecord>(&content) {
            println!("  [+] Agent: {} | Actions: {}", s.agent.cyan(), s.actions.len());
        }
    }
    println!();
    Ok(())
}

pub fn clean_sessions(days: u64) -> Result<()> {
    let dir = get_workspaces_dir();
    if !dir.exists() { return Ok(()); }
    let cutoff = SystemTime::now() - std::time::Duration::from_secs(days * 86400);
    let mut removed = 0u64;
    let mut freed = 0u64;

    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        if let Ok(m) = entry.metadata() {
            if let Ok(modified) = m.modified() {
                if modified < cutoff {
                    freed += calc_size(&entry.path());
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
        println!("  [+] Removed {} sessions, freed {}", removed, ui::format_size(freed));
    }
    println!();
    Ok(())
}

// ─── NEW FEATURES ───

pub fn tag_session(id: &str, tag: &str) -> Result<()> {
    let meta = get_meta_dir();
    let file = meta.join("tags.json");
    let mut tags: HashMap<String, Vec<String>> = read_json(&file).unwrap_or_default();
    let sid = resolve_id(id)?;
    tags.entry(sid.clone()).or_default().push(tag.to_string());
    std::fs::write(&file, serde_json::to_string_pretty(&tags)?)?;
    println!();
    println!("  [+] Tagged {} with '{}'", &sid[..31.min(sid.len())].cyan(), tag.green());
    println!();
    Ok(())
}

pub fn search_sessions(query: &str) -> Result<()> {
    let dir = get_workspaces_dir();
    let q = query.to_lowercase();
    let meta = get_meta_dir();
    let tags: HashMap<String, Vec<String>> = read_json(&meta.join("tags.json")).unwrap_or_default();
    let notes: HashMap<String, String> = read_json(&meta.join("notes.json")).unwrap_or_default();

    println!();
    println!("  +--------------------------------------------+");
    println!("  |  Search: '{:<32}' |", query);
    println!("  +--------------------------------------------+");

    if !dir.exists() { println!("  |  (no sessions)"); println!("  +--------------------------------------------+\n"); return Ok(()); }

    let mut found = 0;
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        let nl = name.to_lowercase();
        let sid = resolve_id(&name)?;
        let st = tags.get(&sid).cloned().unwrap_or_default();
        let sn = notes.get(&sid).cloned().unwrap_or_default();

        if nl.contains(&q) || st.iter().any(|t| t.to_lowercase().contains(&q)) || sn.to_lowercase().contains(&q) {
            let tag_str = if st.is_empty() { String::new() } else { format!("[{}]", st.join(",")) };
            let note_str = if sn.is_empty() { String::new() } else { format!(" \"{}\"", &sn[..30.min(sn.len())]) };
            println!("  |  {:<28} {}{}  |", name.cyan(), tag_str.yellow(), note_str.dimmed());
            found += 1;
        }
    }
    if found == 0 { println!("  |  (no matches)"); }
    println!("  +--------------------------------------------+");
    println!("  Found: {} session(s)", found);
    println!();
    Ok(())
}

pub fn add_note(id: &str, note: &str) -> Result<()> {
    let meta = get_meta_dir();
    let file = meta.join("notes.json");
    let mut notes: HashMap<String, String> = read_json(&file).unwrap_or_default();
    let sid = resolve_id(id)?;
    notes.insert(sid.clone(), note.to_string());
    std::fs::write(&file, serde_json::to_string_pretty(&notes)?)?;
    println!();
    println!("  [+] Note added to {}", &sid[..31.min(sid.len())].cyan());
    println!("  | {}", note.dimmed());
    println!();
    Ok(())
}

pub fn show_timeline() -> Result<()> {
    let dir = get_workspaces_dir();
    if !dir.exists() { println!("\n  No sessions.\n"); return Ok(()); }

    println!();
    println!("  +--------------------------------------------+");
    println!("  |  Session Timeline                          |");
    println!("  +--------------------------------------------+");

    let mut entries: Vec<_> = std::fs::read_dir(&dir)?.filter_map(|e| e.ok()).collect();
    entries.sort_by(|a, b| {
        let at = a.metadata().and_then(|m| m.modified()).unwrap_or(SystemTime::UNIX_EPOCH);
        let bt = b.metadata().and_then(|m| m.modified()).unwrap_or(SystemTime::UNIX_EPOCH);
        at.cmp(&bt)
    });

    let mut total_ms = 0u64;
    let mut count = 0usize;

    for entry in &entries {
        let logs = entry.path().join("logs");
        if let Ok(le) = std::fs::read_dir(&logs) {
            for log in le.filter_map(|e| e.ok()) {
                if let Ok(c) = std::fs::read_to_string(log.path()) {
                    if let Ok(s) = serde_json::from_str::<recorder::SessionRecord>(&c) {
                        let time = chrono::DateTime::from_timestamp(s.started_at as i64, 0)
                            .map(|t| t.format("%m-%d %H:%M").to_string()).unwrap_or_else(|| "?".into());
                        let dur = s.duration_ms.unwrap_or(0) as f64 / 1000.0;
                        total_ms += s.duration_ms.unwrap_or(0);
                        count += 1;
                        let bar = "█".repeat((dur / 30.0) as usize);
                        println!("  |  {}  {:<12} {:>6.0}s  {}", time.dimmed(), s.agent.cyan(), dur, bar.green());
                    }
                }
            }
        }
    }

    let mins = total_ms as f64 / 60000.0;
    println!("  +--------------------------------------------+");
    println!("  |  Total: {} sessions, {:.0} minutes", count, mins);
    println!("  +--------------------------------------------+");
    println!();
    Ok(())
}

pub fn show_stats() -> Result<()> {
    let dir = get_workspaces_dir();
    let mut total = 0usize;
    let mut ms = 0u64;
    let mut counts: HashMap<String, u64> = HashMap::new();
    let mut dur_map: HashMap<String, u64> = HashMap::new();
    let mut files = 0u64;

    if dir.exists() {
        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            let logs = entry.path().join("logs");
            if let Ok(le) = std::fs::read_dir(&logs) {
                for log in le.filter_map(|e| e.ok()) {
                    if let Ok(c) = std::fs::read_to_string(log.path()) {
                        if let Ok(s) = serde_json::from_str::<recorder::SessionRecord>(&c) {
                            total += 1;
                            ms += s.duration_ms.unwrap_or(0);
                            *counts.entry(s.agent.clone()).or_default() += 1;
                            *dur_map.entry(s.agent.clone()).or_default() += s.duration_ms.unwrap_or(0);
                        }
                    }
                }
            }
            let merged = entry.path().join(".sandbox-merged");
            if merged.exists() { files += count_files(&merged); }
        }
    }

    let hrs = ms as f64 / 3_600_000.0;
    println!();
    println!("  +==========================================+");
    println!("  |  abox statistics                         |");
    println!("  +==========================================+");
    println!("  |  Sessions:      {:>25} |", total);
    println!("  |  Total time:    {:>25} |", format!("{:.1}h", hrs));
    println!("  |  Files created: {:>25} |", files);
    println!("  +==========================================+");

    if !counts.is_empty() {
        println!();
        println!("  +------------------------------------------+");
        println!("  |  Agent usage                             |");
        println!("  +------------------------------------------+");
        let mut agents: Vec<_> = counts.iter().collect();
        agents.sort_by(|a, b| b.1.cmp(a.1));
        for (agent, cnt) in agents {
            let d = dur_map.get(agent.as_str()).unwrap_or(&0);
            let h = *d as f64 / 3_600_000.0;
            let bar = "█".repeat((*cnt as usize).min(20));
            println!("  |  {:<12} {:>4}x  {:>5.1}h  {}", agent.cyan(), cnt, h, bar.green());
        }
        println!("  +------------------------------------------+");
    }
    println!();
    Ok(())
}

pub fn watch_sessions() -> Result<()> {
    let dir = get_workspaces_dir();
    println!();
    println!("  [~] Watching sessions (Ctrl+C to stop)");
    println!("  +------------------------------------------+");
    let mut last = std::fs::read_dir(&dir).map(|d| d.count()).unwrap_or(0);
    loop {
        std::thread::sleep(std::time::Duration::from_secs(5));
        let cur = std::fs::read_dir(&dir).map(|d| d.count()).unwrap_or(0);
        if cur > last {
            if let Ok(entries) = std::fs::read_dir(&dir) {
                if let Some(latest) = entries.filter_map(|e| e.ok()).next() {
                    println!("  |  [+] New: {:<32} |", latest.file_name().to_string_lossy().cyan());
                }
            }
        }
        last = cur;
    }
}

// ─── HELPERS ───

fn resolve_id(id: &str) -> Result<String> {
    let dir = get_workspaces_dir();
    if std::path::Path::new(id).exists() { return Ok(id.to_string()); }
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name == id || name.starts_with(id) { return Ok(name); }
        let exact = entry.path().join("logs").join(format!("{}.json", id));
        if exact.exists() { return Ok(name); }
    }
    Ok(id.to_string())
}

fn find_session(id: &str) -> Result<PathBuf> {
    let dir = get_workspaces_dir();
    if std::path::Path::new(id).exists() { return Ok(PathBuf::from(id)); }
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let logs = entry.path().join("logs");
        let exact = logs.join(format!("{}.json", id));
        if exact.exists() { return Ok(exact); }
        let name = entry.file_name().to_string_lossy().to_string();
        if name == id || name.starts_with(id) {
            if let Ok(entries) = std::fs::read_dir(&logs) {
                let mut jsons: Vec<_> = entries.filter_map(|e| e.ok())
                    .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("json")).collect();
                if !jsons.is_empty() {
                    jsons.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
                    return Ok(jsons[0].path());
                }
            }
        }
    }
    anyhow::bail!("Session not found: {}", id)
}

fn count_files(dir: &PathBuf) -> u64 {
    let mut c = 0;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            if let Ok(m) = entry.metadata() {
                if m.is_dir() { c += count_files(&entry.path()); } else { c += 1; }
            }
        }
    }
    c
}

fn calc_size(path: &PathBuf) -> u64 {
    let mut t = 0;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.filter_map(|e| e.ok()) {
            if let Ok(m) = entry.metadata() {
                if m.is_dir() { t += calc_size(&entry.path()); } else { t += m.len(); }
            }
        }
    }
    t
}

fn read_json<T: serde::de::DeserializeOwned>(path: &PathBuf) -> Option<T> {
    std::fs::read_to_string(path).ok().and_then(|c| serde_json::from_str(&c).ok())
}
