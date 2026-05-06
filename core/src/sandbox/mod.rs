use anyhow::Result;
use std::path::{Path, PathBuf};

pub struct SandboxFs {
    pub root: PathBuf,
}

impl SandboxFs {
    pub fn new(workspace: &Path) -> Result<Self> {
        Ok(Self {
            root: workspace.to_path_buf(),
        })
    }

    pub fn agent_root(&self) -> &Path {
        &self.root
    }

    pub fn mount(&self) -> Result<()> {
        Ok(())
    }

    pub fn modified_files(&self) -> Result<Vec<PathBuf>> {
        Ok(vec![])
    }
}
