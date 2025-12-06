use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

pub struct FsOperations;

impl FsOperations {
    pub fn read_file(path: &str) -> Result<String> {
        let path = PathBuf::from(path);
        fs::read_to_string(&path).with_context(|| format!("Failed to read file: {:?}", path))
    }

    pub fn write_file(path: &str, content: &str) -> Result<()> {
        let path = PathBuf::from(path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, content).with_context(|| format!("Failed to write file: {:?}", path))
    }

    pub fn make_dir(path: &str) -> Result<()> {
        let path = PathBuf::from(path);
        fs::create_dir_all(&path).with_context(|| format!("Failed to create directory: {:?}", path))
    }

    pub fn list_dir(path: &str) -> Result<Vec<String>> {
        let path = PathBuf::from(path);
        let entries =
            fs::read_dir(&path).with_context(|| format!("Failed to list directory: {:?}", path))?;

        let mut result = Vec::new();
        for entry in entries {
            let entry = entry?;
            if let Some(name) = entry.file_name().to_str() {
                result.push(name.to_string());
            }
        }

        Ok(result)
    }
}
