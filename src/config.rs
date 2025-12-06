use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

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
}

impl ConfigManager {
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir().context("Could not find home directory")?;
        let config_dir = home.join(".aish");
        let config_path = config_dir.join("config.toml");
        let env_path = config_dir.join("tokens.env");
        let ignore_path = config_dir.join(".aishignore");

        Ok(Self {
            config_dir,
            config_path,
            env_path,
            ignore_path,
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
}
