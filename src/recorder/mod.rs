use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::time::SystemTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRecord {
    pub id: String,
    pub session_id: String,
    pub action_type: String,
    pub timestamp: u64,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    pub id: String,
    pub sandbox_name: String,
    pub agent: String,
    pub task: String,
    pub started_at: u64,
    pub ended_at: Option<u64>,
    pub duration_ms: Option<u64>,
    pub actions: Vec<ActionRecord>,
    pub metadata: SessionMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub cwd: String,
    pub env: std::collections::HashMap<String, String>,
    pub host_info: HostInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostInfo {
    pub os: String,
    pub arch: String,
    pub hostname: String,
}

pub struct Recorder {
    session: SessionRecord,
    log_path: std::path::PathBuf,
    start_time: SystemTime,
}

impl Recorder {
    pub fn new(sandbox_name: &str, agent: &str, task: &str, log_dir: &Path) -> Result<Self> {
        std::fs::create_dir_all(log_dir)?;

        let host_info = HostInfo {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            hostname: gethostname(),
        };

        let env: std::collections::HashMap<String, String> = std::env::vars()
            .filter(|(k, _)| k.starts_with("PATH") || k.starts_with("HOME") || k.starts_with("USER"))
            .collect();

        let session = SessionRecord {
            id: Uuid::new_v4().to_string(),
            sandbox_name: sandbox_name.to_string(),
            agent: agent.to_string(),
            task: task.to_string(),
            started_at: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_secs(),
            ended_at: None,
            duration_ms: None,
            actions: Vec::new(),
            metadata: SessionMetadata {
                cwd: std::env::current_dir()?.to_string_lossy().to_string(),
                env,
                host_info,
            },
        };

        let log_path = log_dir.join(format!("{}.json", session.id));
        let recorder = Self {
            session,
            log_path,
            start_time: SystemTime::now(),
        };
        recorder.flush()?;

        Ok(recorder)
    }

    pub fn session_id(&self) -> &str {
        &self.session.id
    }

    pub fn record_action(&mut self, action_type: &str, data: serde_json::Value) -> Result<()> {
        let action = ActionRecord {
            id: Uuid::new_v4().to_string(),
            session_id: self.session.id.clone(),
            action_type: action_type.to_string(),
            timestamp: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_secs(),
            data,
        };
        self.session.actions.push(action);
        self.flush()?;
        Ok(())
    }

    pub fn record_file_read(&mut self, path: &str, content: &str) -> Result<()> {
        self.record_action("file_read", serde_json::json!({
            "path": path,
            "content": content,
            "content_length": content.len(),
        }))
    }

    pub fn record_file_write(&mut self, path: &str, content: &str, diff: Option<&str>) -> Result<()> {
        self.record_action("file_write", serde_json::json!({
            "path": path,
            "content": content,
            "content_length": content.len(),
            "diff": diff,
        }))
    }

    pub fn record_file_delete(&mut self, path: &str) -> Result<()> {
        self.record_action("file_delete", serde_json::json!({
            "path": path,
        }))
    }

    pub fn record_exec(&mut self, cmd: &str, stdout: &str, stderr: &str, exit_code: i32, duration_ms: u64) -> Result<()> {
        self.record_action("exec", serde_json::json!({
            "command": cmd,
            "stdout": stdout,
            "stderr": stderr,
            "exit_code": exit_code,
            "duration_ms": duration_ms,
        }))
    }

    pub fn record_network_call(&mut self, method: &str, url: &str, status: Option<u16>, duration_ms: u64) -> Result<()> {
        self.record_action("network_call", serde_json::json!({
            "method": method,
            "url": url,
            "status": status,
            "duration_ms": duration_ms,
        }))
    }

    pub fn record_llm_call(&mut self, model: &str, prompt: &str, response: &str, prompt_tokens: u32, completion_tokens: u32) -> Result<()> {
        self.record_action("llm_call", serde_json::json!({
            "model": model,
            "prompt_preview": &prompt[..prompt.len().min(200)],
            "response_preview": &response[..response.len().min(200)],
            "prompt_tokens": prompt_tokens,
            "completion_tokens": completion_tokens,
        }))
    }

    pub fn record_error(&mut self, error_type: &str, message: &str) -> Result<()> {
        self.record_action("error", serde_json::json!({
            "error_type": error_type,
            "message": message,
        }))
    }

    pub fn finish(&mut self) -> Result<()> {
        let duration = self.start_time.elapsed()?.as_millis() as u64;
        self.session.ended_at = Some(
            SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_secs()
        );
        self.session.duration_ms = Some(duration);
        self.flush()
    }

    fn flush(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.session)?;
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.log_path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    pub fn get_session(&self) -> &SessionRecord {
        &self.session
    }

    pub fn action_count(&self) -> usize {
        self.session.actions.len()
    }
}

pub fn load_session(path: &Path) -> Result<SessionRecord> {
    let content = std::fs::read_to_string(path)?;
    let session: SessionRecord = serde_json::from_str(&content)?;
    Ok(session)
}

pub fn list_sessions(log_dir: &Path) -> Result<Vec<SessionRecord>> {
    let mut sessions = Vec::new();
    if !log_dir.exists() {
        return Ok(sessions);
    }
    for entry in std::fs::read_dir(log_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            if let Ok(session) = load_session(&path) {
                sessions.push(session);
            }
        }
    }
    sessions.sort_by(|a, b| b.started_at.cmp(&a.started_at));
    Ok(sessions)
}

fn gethostname() -> String {
    std::process::Command::new("hostname")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_else(|| "unknown".to_string())
        .trim()
        .to_string()
}
