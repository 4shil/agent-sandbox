use anyhow::{Result, Context};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

const DB_DIR: &str = "~/.agent-sandbox";

#[derive(Debug, Serialize, Deserialize)]
pub struct Sandbox {
    pub name: String,
    pub agent: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Action {
    pub id: String,
    pub session_id: String,
    pub action_type: String,
    pub timestamp: String,
    pub data: serde_json::Value,
}

fn db_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".agent-sandbox").join("sandboxes.db")
}

fn ensure_db_dir() -> Result<()> {
    let home = std::env::var("HOME")?;
    let dir = PathBuf::from(home).join(".agent-sandbox");
    std::fs::create_dir_all(&dir)
        .context("Failed to create .agent-sandbox directory")?;
    Ok(())
}

pub fn get_connection() -> Result<Connection> {
    ensure_db_dir()?;
    let path = db_path();
    let conn = Connection::open(&path)?;
    init_schema(&conn)?;
    Ok(conn)
}

fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS sandboxes (
            id TEXT PRIMARY KEY,
            name TEXT UNIQUE NOT NULL,
            agent TEXT NOT NULL DEFAULT 'claude',
            template TEXT NOT NULL DEFAULT 'empty',
            workspace_path TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            status TEXT NOT NULL DEFAULT 'active'
        );

        CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            sandbox_id TEXT NOT NULL,
            started_at TEXT NOT NULL DEFAULT (datetime('now')),
            ended_at TEXT,
            task TEXT,
            FOREIGN KEY (sandbox_id) REFERENCES sandboxes(id)
        );

        CREATE TABLE IF NOT EXISTS actions (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            action_type TEXT NOT NULL,
            timestamp TEXT NOT NULL DEFAULT (datetime('now')),
            data TEXT NOT NULL,
            FOREIGN KEY (session_id) REFERENCES sessions(id)
        );

        CREATE TABLE IF NOT EXISTS files (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            file_path TEXT NOT NULL,
            content TEXT,
            diff TEXT,
            action TEXT NOT NULL,
            timestamp TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (session_id) REFERENCES sessions(id)
        );
    ")?;
    Ok(())
}

pub fn init_workspace(name: &str, template: &str) -> Result<()> {
    let conn = get_connection()?;
    let id = Uuid::new_v4().to_string();

    let home = std::env::var("HOME")?;
    let workspace = PathBuf::from(home)
        .join(".agent-sandbox")
        .join("workspaces")
        .join(name);

    std::fs::create_dir_all(&workspace)
        .context("Failed to create workspace directory")?;

    // Copy template files
    match template {
        "node" => {
            std::fs::write(workspace.join("package.json"), r#"{
  "name": "project",
  "version": "1.0.0",
  "main": "index.js"
}
"#)?;
            std::fs::write(workspace.join("index.js"), "// Start here\n")?;
        }
        "python" => {
            std::fs::write(workspace.join("pyproject.toml"), r#"[project]
name = "project"
version = "1.0.0"
"#)?;
            std::fs::write(workspace.join("main.py"), "# Start here\n")?;
        }
        "rust" => {
            std::fs::write(workspace.join("Cargo.toml"), r#"[package]
name = "project"
version = "0.1.0"
edition = "2021"
"#)?;
            std::fs::create_dir_all(workspace.join("src"))?;
            std::fs::write(workspace.join("src").join("main.rs"), "fn main() {\n    println!(\"Hello\");\n}\n")?;
        }
        _ => {} // empty
    }

    // Create overlay directories for FUSE
    let overlay = workspace.join(".sandbox-overlay");
    std::fs::create_dir_all(overlay.join("upper"))?;
    std::fs::create_dir_all(overlay.join("work"))?;

    // Insert into database
    conn.execute(
        "INSERT INTO sandboxes (id, name, agent, template, workspace_path) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, name, "claude", template, workspace.to_string_lossy()],
    )?;

    Ok(())
}

pub fn list_sandboxes() -> Result<Vec<Sandbox>> {
    let conn = get_connection()?;
    let mut stmt = conn.prepare(
        "SELECT name, agent, created_at FROM sandboxes WHERE status = 'active' ORDER BY created_at DESC"
    )?;

    let sandboxes = stmt.query_map([], |row| {
        Ok(Sandbox {
            name: row.get(0)?,
            agent: row.get(1)?,
            created_at: row.get(2)?,
        })
    })?;

    let mut result = Vec::new();
    for sb in sandboxes {
        result.push(sb?);
    }
    Ok(result)
}

pub fn log_action(session_id: &str, action_type: &str, data: &serde_json::Value) -> Result<()> {
    let conn = get_connection()?;
    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO actions (id, session_id, action_type, data) VALUES (?1, ?2, ?3, ?4)",
        params![id, session_id, action_type, data.to_string()],
    )?;
    Ok(())
}
