use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::ai::workspace_context::WorkspaceContext;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub llm: LlmConfig,
    pub security: SecurityConfig,
    pub whitelist: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LlmConfig {
    pub provider: String,
    pub api_url: String,
    pub model: String,
    pub max_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SecurityConfig {
    pub allow_absolute_paths: bool,
    pub allow_config_path_access: bool,
    pub blocked_extensions: Vec<String>,
    pub allowed_operations: OperationPermissions,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OperationPermissions {
    #[serde(rename = "fs.makedir")]
    pub fs_makedir: bool,
    #[serde(rename = "fs.makefile")]
    pub fs_makefile: bool,
    #[serde(rename = "fs.writefile")]
    pub fs_writefile: bool,
    #[serde(rename = "fs.readfile")]
    pub fs_readfile: bool,
    #[serde(rename = "fs.listdir")]
    pub fs_listdir: bool,
    pub shell: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            llm: LlmConfig {
                provider: "OpenAI".to_string(),
                api_url: "https://api.openai.com/v1".to_string(),
                model: "gpt-4".to_string(),
                max_tokens: 4096,
            },
            security: SecurityConfig {
                allow_absolute_paths: false,
                allow_config_path_access: false,
                blocked_extensions: vec![".env".to_string()],
                allowed_operations: OperationPermissions {
                    fs_makedir: true,
                    fs_makefile: true,
                    fs_writefile: true,
                    fs_readfile: true,
                    fs_listdir: true,
                    shell: true,
                },
            },
            whitelist: vec![],
        }
    }
}

pub struct ConfigManager {
    config_dir: PathBuf,
    config_path: PathBuf,
    env_path: PathBuf,
    ignore_path: PathBuf,
    system_prompt_path: PathBuf,
}

impl ConfigManager {
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir().context("Could not find home directory")?;
        let config_dir = home.join(".aish");
        let config_path = config_dir.join("config.toml");
        let env_path = config_dir.join("tokens.env");
        let ignore_path = config_dir.join(".aishignore");
        let system_prompt_path = config_dir.join("system_prompt.txt");

        Ok(Self {
            config_dir,
            config_path,
            env_path,
            ignore_path,
            system_prompt_path,
        })
    }

    pub fn is_initialized(&self) -> bool {
        self.config_path.exists() && self.env_path.exists()
    }

    pub fn get_config_dir(&self) -> &PathBuf {
        &self.config_dir
    }

    pub fn load_config(&self) -> Result<Config> {
        let content =
            fs::read_to_string(&self.config_path).context("Failed to read config file")?;
        let config: Config = toml::from_str(&content).context("Failed to parse config file")?;
        Ok(config)
    }

    pub fn save_config(&self, config: &Config) -> Result<()> {
        fs::create_dir_all(&self.config_dir)?;
        let content = toml::to_string_pretty(config)?;
        fs::write(&self.config_path, content)?;
        Ok(())
    }

    pub fn load_api_key(&self) -> Result<String> {
        dotenv::from_path(&self.env_path).ok();
        std::env::var("API_KEY").context("API_KEY not found in tokens.env")
    }

    pub fn save_api_key(&self, api_key: &str) -> Result<()> {
        fs::create_dir_all(&self.config_dir)?;
        fs::write(&self.env_path, format!("API_KEY={}", api_key))?;
        Ok(())
    }

    pub fn load_ignore_patterns(&self) -> Vec<String> {
        if let Ok(content) = fs::read_to_string(&self.ignore_path) {
            content.lines().map(|s| s.to_string()).collect()
        } else {
            vec![]
        }
    }

    pub fn get_config_value(&self, key: &str) -> Result<String> {
        let config = self.load_config()?;
        let parts: Vec<&str> = key.split('.').collect();

        match parts.as_slice() {
            ["llm", "max_tokens"] => Ok(config.llm.max_tokens.to_string()),
            ["llm", "model"] => Ok(config.llm.model),
            ["llm", "provider"] => Ok(config.llm.provider),
            ["llm", "api_url"] => Ok(config.llm.api_url),
            ["security", "allow_absolute_paths"] => {
                Ok(config.security.allow_absolute_paths.to_string())
            }
            ["security", "allow_config_path_access"] => {
                Ok(config.security.allow_config_path_access.to_string())
            }
            _ => anyhow::bail!("Unknown config key: {}", key),
        }
    }

    pub fn set_config_value(&self, key: &str, value: &str) -> Result<()> {
        let mut config = self.load_config()?;
        let parts: Vec<&str> = key.split('.').collect();

        match parts.as_slice() {
            ["llm", "max_tokens"] => config.llm.max_tokens = value.parse()?,
            ["llm", "model"] => config.llm.model = value.to_string(),
            ["llm", "provider"] => config.llm.provider = value.to_string(),
            ["llm", "api_url"] => config.llm.api_url = value.to_string(),
            ["security", "allow_absolute_paths"] => {
                config.security.allow_absolute_paths = value.parse()?
            }
            ["security", "allow_config_path_access"] => {
                config.security.allow_config_path_access = value.parse()?
            }
            _ => anyhow::bail!("Unknown config key: {}", key),
        }

        self.save_config(&config)
    }

    pub fn load_system_prompt(&self) -> Result<String> {
        let prompt = if self.system_prompt_path.exists() {
            fs::read_to_string(&self.system_prompt_path)
                .context("Failed to read system_prompt.txt")?
        } else {
            // Create default system prompt if it doesn't exist
            let default_prompt = get_default_system_prompt();
            fs::create_dir_all(&self.config_dir)?;
            fs::write(&self.system_prompt_path, &default_prompt)?;
            default_prompt
        };

        // Replace placeholders with actual values
        Ok(replace_system_prompt_placeholders(&prompt))
    }

    pub fn get_system_prompt_path(&self) -> &PathBuf {
        &self.system_prompt_path
    }
}

fn get_default_system_prompt() -> String {
    include_str!("../data/system_prompt.txt").to_string()
}

fn replace_system_prompt_placeholders(prompt: &str) -> String {
    let mut result = prompt.to_string();

    // Get hostname
    let hostname = get_hostname().unwrap_or_else(|| "unknown".to_string());
    result = result.replace("{{HOSTNAME}}", &hostname);

    // Get OS
    let os = get_os().unwrap_or_else(|| "Linux".to_string());
    result = result.replace("{{OS}}", &os);

    // Get distribution
    let distribution = get_distribution().unwrap_or_else(|| "Unknown".to_string());
    result = result.replace("{{DISTRIBUTION}}", &distribution);

    // Get current user
    let user = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".to_string());
    result = result.replace("{{USER}}", &user);

    // Get current working directory
    let cwd = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    result = result.replace("{{CWD}}", &cwd);

    // Get workspace context flags
    let workspace_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let workspace_context = WorkspaceContext::detect(&workspace_dir);
    let flags = workspace_context.to_flags_string();
    result = result.replace("{{FLAGS}}", &flags);

    result
}

fn get_hostname() -> Option<String> {
    // Try to get hostname from system
    if let Ok(output) = Command::new("hostname").output() {
        if output.status.success() {
            return String::from_utf8(output.stdout)
                .ok()
                .map(|s| s.trim().to_string());
        }
    }

    // Fallback to hostname crate or env var
    std::env::var("HOSTNAME")
        .ok()
        .or_else(|| std::env::var("COMPUTERNAME").ok())
}

fn get_os() -> Option<String> {
    #[cfg(target_os = "linux")]
    return Some("Linux".to_string());

    #[cfg(target_os = "macos")]
    return Some("macOS".to_string());

    #[cfg(target_os = "windows")]
    return Some("Windows".to_string());

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    return Some("Unknown".to_string());
}

fn get_distribution() -> Option<String> {
    // Try to read /etc/os-release (Linux)
    if let Ok(content) = fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if line.starts_with("PRETTY_NAME=") {
                let distro = line
                    .strip_prefix("PRETTY_NAME=")
                    .and_then(|s| s.strip_prefix('"'))
                    .and_then(|s| s.strip_suffix('"'))
                    .or_else(|| line.strip_prefix("PRETTY_NAME=").map(|s| s.trim()));
                if let Some(d) = distro {
                    return Some(d.to_string());
                }
            }
            if line.starts_with("NAME=") && !content.contains("PRETTY_NAME") {
                let distro = line
                    .strip_prefix("NAME=")
                    .and_then(|s| s.strip_prefix('"'))
                    .and_then(|s| s.strip_suffix('"'))
                    .or_else(|| line.strip_prefix("NAME=").map(|s| s.trim()));
                if let Some(d) = distro {
                    return Some(d.to_string());
                }
            }
        }
    }

    // Try /etc/redhat-release (RHEL, CentOS, Fedora)
    if let Ok(content) = fs::read_to_string("/etc/redhat-release") {
        return Some(content.trim().to_string());
    }

    // Try /etc/debian_version (Debian-based)
    if let Ok(content) = fs::read_to_string("/etc/debian_version") {
        return Some(format!("Debian {}", content.trim()));
    }

    None
}
