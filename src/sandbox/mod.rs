use anyhow::{Result, Context};
use std::path::{Path, PathBuf};

pub struct SandboxFs {
    pub root: PathBuf,       // Original project (lower)
    pub overlay: PathBuf,    // Sandbox workspace root
    pub upper: PathBuf,      // Agent writes go here
    pub work: PathBuf,       // FUSE internal workdir
    pub merged: PathBuf,     // What agent sees (combined view)
}

impl SandboxFs {
    pub fn new(workspace: &Path) -> Result<Self> {
        let overlay = workspace.join(".sandbox-overlay");
        let upper = overlay.join("upper");
        let work = overlay.join("work");
        let merged = workspace.join(".sandbox-merged");

        // Ensure dirs exist
        std::fs::create_dir_all(&upper)
            .context("Failed to create overlay upper dir")?;
        std::fs::create_dir_all(&work)
            .context("Failed to create overlay work dir")?;
        std::fs::create_dir_all(&merged)
            .context("Failed to create overlay merged dir")?;

        Ok(Self {
            root: workspace.to_path_buf(),
            overlay,
            upper,
            work,
            merged,
        })
    }

    /// Get the path the agent should operate on
    pub fn agent_root(&self) -> &Path {
        &self.merged
    }

    /// Initialize merged view by copying project files
    /// In real FUSE mode, lowerdir would be the original project
    pub fn mount(&self) -> Result<()> {
        // For now, create a bind mount simulation
        // Real FUSE would use: mount -t overlay overlay -o lowerdir=X,upperdir=Y,workdir=Y
        
        // Copy initial project state to merged (simulating lower layer)
        copy_dir_recursive(&self.root, &self.merged, &[".sandbox-overlay", ".sandbox-merged"])?;
        
        Ok(())
    }

    /// Unmount and sync changes back
    pub fn unmount(&self) -> Result<()> {
        // In FUSE mode, changes are already in upper dir
        // For simulation, we'd copy from merged back to root
        Ok(())
    }

    /// Get a file path in the overlay (for interception)
    pub fn overlay_path(&self, relative: &Path) -> PathBuf {
        self.upper.join(relative)
    }

    /// List files that were modified in the overlay
    pub fn modified_files(&self) -> Result<Vec<PathBuf>> {
        let mut modified = Vec::new();
        if self.upper.exists() {
            walk_dir(&self.upper, &mut modified)?;
        }
        Ok(modified)
    }
}

fn walk_dir(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                walk_dir(&path, files)?;
            } else {
                files.push(path);
            }
        }
    }
    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path, exclude: &[&str]) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        
        // Skip excluded directories
        if exclude.iter().any(|e| name_str == *e) {
            continue;
        }
        
        let src_path = entry.path();
        let dst_path = dst.join(&name);
        
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path, exclude)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_sandbox_fs_creation() {
        let tmp = std::env::temp_dir().join("agent-sandbox-test");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        
        // Create a fake project file
        fs::write(tmp.join("test.txt"), "hello").unwrap();
        
        let sfs = SandboxFs::new(&tmp).unwrap();
        assert!(sfs.upper.exists());
        assert!(sfs.work.exists());
        assert!(sfs.merged.exists());
        
        let _ = fs::remove_dir_all(&tmp);
    }
}
