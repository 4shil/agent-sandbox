use anyhow::Result;
use std::path::{Path, PathBuf};

pub struct SandboxFs {
    pub root: PathBuf,
    pub merged: PathBuf,
}

impl SandboxFs {
    pub fn new(workspace: &Path) -> Result<Self> {
        let merged = workspace.join(".sandbox-merged");
        std::fs::create_dir_all(&merged)?;
        Ok(Self {
            root: workspace.to_path_buf(),
            merged,
        })
    }

    pub fn agent_root(&self) -> &Path {
        &self.merged
    }

    pub fn mount(&self) -> Result<()> {
        // Copy template files to merged
        copy_dir_recursive(&self.root, &self.merged, &[".sandbox-merged", ".sandbox-overlay", "logs"])?;
        Ok(())
    }

    pub fn modified_files(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        walk_dir(&self.merged, &mut files)?;
        Ok(files)
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
    if !src.exists() { return Ok(()); }
    std::fs::create_dir_all(dst)?;
    
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        
        if exclude.iter().any(|e| name_str == *e) { continue; }
        
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
