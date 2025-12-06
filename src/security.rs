use anyhow::{bail, Result};
use regex::Regex;
use std::path::PathBuf;

use crate::config::{Config, ConfigManager};

pub struct SecurityValidator {
    config: Config,
    config_manager: ConfigManager,
    ignore_patterns: Vec<Regex>,
}

impl SecurityValidator {
    pub fn new(config: Config) -> Result<Self> {
        let config_manager = ConfigManager::new()?;
        let patterns = config_manager.load_ignore_patterns();
        let ignore_patterns: Vec<Regex> = patterns
            .iter()
            .filter_map(|p| Regex::new(&p.replace("*", ".*")).ok())
            .collect();

        Ok(Self {
            config,
            config_manager,
            ignore_patterns,
        })
    }

    pub fn validate_path(&self, path: &str) -> Result<()> {
        let path = PathBuf::from(path);

        // Check if absolute path
        if path.is_absolute() && !self.config.security.allow_absolute_paths {
            bail!("Absolute paths are not allowed");
        }

        // Check if accessing config directory
        if !self.config.security.allow_config_path_access {
            let config_dir = self.config_manager.get_config_dir();
            if path.starts_with(config_dir) {
                bail!("Access to configuration directory is not allowed");
            }
        }

        // Check blocked extensions
        if let Some(ext) = path.extension() {
            let ext_str = format!(".{}", ext.to_string_lossy());
            if self.config.security.blocked_extensions.contains(&ext_str) {
                bail!("File extension {} is blocked", ext_str);
            }
        }

        // Check ignore patterns
        let path_str = path.to_string_lossy();
        for pattern in &self.ignore_patterns {
            if pattern.is_match(&path_str) {
                bail!("Path matches ignore pattern");
            }
        }

        Ok(())
    }

    pub fn validate_operation(&self, operation: &str) -> Result<()> {
        let allowed = match operation {
            "fs.makedir" => self.config.security.allowed_operations.fs_makedir,
            "fs.makefile" => self.config.security.allowed_operations.fs_makefile,
            "fs.writefile" => self.config.security.allowed_operations.fs_writefile,
            "fs.readfile" => self.config.security.allowed_operations.fs_readfile,
            "fs.listdir" => self.config.security.allowed_operations.fs_listdir,
            "shell" => self.config.security.allowed_operations.shell,
            _ => false,
        };

        if !allowed {
            bail!("Operation {} is not allowed", operation);
        }

        Ok(())
    }

    pub fn is_whitelisted(&self, command: &str) -> bool {
        self.config.whitelist.iter().any(|pattern| {
            if let Ok(regex) = Regex::new(pattern) {
                regex.is_match(command)
            } else {
                false
            }
        })
    }
}
