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
    pub actions: Vec<ActionRecord>,
}

pub struct Recorder {
    session: SessionRecord,
    log_path: std::path::PathBuf,
}

impl Recorder {
    pub fn new(sandbox_name: &str, agent: &str, task: &str, log_dir: &Path) -> Result<Self> {
        std::fs::create_dir_all(log_dir)?;
        
        let session = SessionRecord {
            id: Uuid::new_v4().to_string(),
            sandbox_name: sandbox_name.to_string(),
            agent: agent.to_string(),
            task: task.to_string(),
            started_at: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_secs(),
            ended_at: None,
            actions: Vec::new(),
        };

        let log_path = log_dir.join(format!("{}.json", session.id));

        Ok(Self { session, log_path })
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

    pub fn record_file_write(&mut self, path: &str, content: &str) -> Result<()> {
        self.record_action("file_write", serde_json::json!({
            "path": path,
            "content": content,
            "content_length": content.len(),
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

    pub fn record_llm_call(&mut self, model: &str, prompt_tokens: u32, completion_tokens: u32) -> Result<()> {
        self.record_action("llm_call", serde_json::json!({
            "model": model,
            "prompt_tokens": prompt_tokens,
            "completion_tokens": completion_tokens,
        }))
    }

    pub fn finish(&mut self) -> Result<()> {
        self.session.ended_at = Some(
            SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_secs()
        );
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
}

pub fn load_session(path: &Path) -> Result<SessionRecord> {
    let content = std::fs::read_to_string(path)?;
    let session: SessionRecord = serde_json::from_str(&content)?;
    Ok(session)
}
