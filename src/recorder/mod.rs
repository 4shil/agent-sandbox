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
    pub action_type: String,
    pub timestamp: u64,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    pub id: String,
    pub sandbox: String,
    pub agent: String,
    pub started_at: u64,
    pub ended_at: Option<u64>,
    pub duration_ms: Option<u64>,
    pub actions: Vec<ActionRecord>,
}

pub struct Recorder {
    session: SessionRecord,
    log_path: std::path::PathBuf,
    start_time: SystemTime,
}

impl Recorder {
    pub fn new(sandbox: &str, agent: &str, _mode: &str, log_dir: &Path) -> Result<Self> {
        std::fs::create_dir_all(log_dir)?;

        let session = SessionRecord {
            id: Uuid::new_v4().to_string(),
            sandbox: sandbox.to_string(),
            agent: agent.to_string(),
            started_at: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_secs(),
            ended_at: None,
            duration_ms: None,
            actions: Vec::new(),
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
            action_type: action_type.to_string(),
            timestamp: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_secs(),
            data,
        };
        self.session.actions.push(action);
        self.flush()
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
}

pub fn load_session(path: &Path) -> Result<SessionRecord> {
    let content = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content)?)
}
