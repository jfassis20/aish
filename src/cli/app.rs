use anyhow::Result;
use colored::*;
use inquire::Select;

use crate::config::{Config, ConfigManager};
use crate::fs_ops::FsOperations;
use crate::llm::{ChatMessage, LlmClient};
use crate::security::SecurityValidator;
use crate::shell::ShellExecutor;

pub struct App {
    llm_client: LlmClient,
    security: SecurityValidator,
    messages: Vec<ChatMessage>,
    accept_all: bool,
}

impl App {
    pub fn new(config: Config, initial_prompt: String, accept_all: bool) -> Result<Self> {
        let config_manager = ConfigManager::new()?;
        let api_key = config_manager.load_api_key()?;
        let llm_client = LlmClient::new(&config, api_key);
        let security = SecurityValidator::new(config.clone())?;

        let system_prompt = config_manager.load_system_prompt()?;
        let system_message = ChatMessage {
            role: "system".to_string(),
            content: Some(system_prompt),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        };

        let user_message = ChatMessage {
            role: "user".to_string(),
            content: Some(initial_prompt),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        };

        Ok(Self {
            llm_client,
            security,
            messages: vec![system_message, user_message],
            accept_all,
        })
    }

    pub fn new_empty(config: Config, accept_all: bool) -> Result<Self> {
        let config_manager = ConfigManager::new()?;
        let api_key = config_manager.load_api_key()?;
        let llm_client = LlmClient::new(&config, api_key);
        let security = SecurityValidator::new(config.clone())?;

        let system_prompt = config_manager.load_system_prompt()?;
        let system_message = ChatMessage {
            role: "system".to_string(),
            content: Some(system_prompt),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        };

        Ok(Self {
            llm_client,
            security,
            messages: vec![system_message],
            accept_all,
        })
    }

    pub fn add_user_message(&mut self, prompt: String) {
        let user_message = ChatMessage {
            role: "user".to_string(),
            content: Some(prompt),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        };
        self.messages.push(user_message);
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            println!();
            println!("{}", "→ LLM is Thinking...".bright_cyan());

            // Update system message with current CWD before each request
            self.update_system_message_cwd()?;

            let response = match self.llm_client.chat(self.messages.clone()).await {
                Ok(r) => r,
                Err(e) => {
                    eprintln!(
                        "{} {}",
                        "×".bright_red(),
                        format!("Error: {}", e).bright_red()
                    );
                    return Err(e);
                }
            };

            // Always add the response to messages first
            self.messages.push(response.clone());

            if let Some(tool_calls) = &response.tool_calls {
                for tool_call in tool_calls {
                    let args: serde_json::Value =
                        serde_json::from_str(&tool_call.function.arguments)?;

                    match tool_call.function.name.as_str() {
                        "execute_shell" => {
                            if let Some(command) = args.get("command").and_then(|c| c.as_str()) {
                                let should_execute = self.should_execute("shell", command).await?;
                                if should_execute {
                                    let result = self.execute_action("shell", command).await?;
                                    self.add_tool_result(&tool_call.id, "shell", &result)
                                        .await?;
                                } else {
                                    println!("{}", "× Command rejected".bright_red());
                                    // In interactive mode, continue instead of returning
                                    return Ok(());
                                }
                            }
                        }
                        "fs_readfile" | "fs_writefile" | "fs_makedir" | "fs_listdir" => {
                            let operation = tool_call.function.name.clone();
                            let operation_desc = self.format_operation(&operation, &args);

                            let should_execute =
                                self.should_execute(&operation, &operation_desc).await?;
                            if should_execute {
                                let result =
                                    self.execute_action(&operation, &args.to_string()).await?;
                                self.add_tool_result(&tool_call.id, &operation, &result)
                                    .await?;
                            } else {
                                println!("{}", "× Operation rejected".bright_red());
                                return Ok(());
                            }
                        }
                        _ => {}
                    }
                }
            } else if let Some(content) = &response.content {
                println!();
                println!("{}", "→ LLM Response:".bright_cyan());
                println!("{}", content);
                break;
            }
        }

        Ok(())
    }

    async fn should_execute(&self, op_type: &str, description: &str) -> Result<bool> {
        // Whitelist only applies when accept_all is true
        // If accept_all is false, always ask user (whitelist is ignored)
        if self.accept_all {
            // Check if command is whitelisted (only for shell commands)
            let is_whitelisted = op_type == "shell" && self.security.is_whitelisted(description);

            if is_whitelisted {
                println!(
                    "{} {}",
                    "+".bright_green(),
                    format!("Auto-approved (whitelisted): {}", description).bright_green()
                );
            } else {
                println!(
                    "{} {}",
                    "+".bright_green(),
                    format!("Auto-approved: {}", description).bright_green()
                );
            }
            return Ok(true);
        }

        // Otherwise, ask for user approval (whitelist is ignored)
        println!();
        println!(
            "{}",
            format!("→ Proposed {}: {}", op_type, description).bright_yellow()
        );

        let options = vec!["Accept", "Reject"];
        let choice = Select::new("What would you like to do?", options)
            .with_help_message("Use ↑/↓ to navigate, Enter to select")
            .prompt()?;

        Ok(choice == "Accept")
    }

    async fn execute_action(&self, op_type: &str, command: &str) -> Result<String> {
        self.security.validate_operation(op_type)?;

        println!();
        println!("{}", "> Executing...".bright_cyan());

        match op_type {
            "shell" => ShellExecutor::execute(command),
            "fs_readfile" => {
                let args: serde_json::Value = serde_json::from_str(command)?;
                let path = args["path"].as_str().unwrap();
                self.security.validate_path(path)?;
                FsOperations::read_file(path)
            }
            "fs_writefile" => {
                let args: serde_json::Value = serde_json::from_str(command)?;
                let path = args["path"].as_str().unwrap();
                let content = args["content"].as_str().unwrap();
                self.security.validate_path(path)?;
                FsOperations::write_file(path, content)?;
                Ok("File written successfully".to_string())
            }
            "fs_makedir" => {
                let args: serde_json::Value = serde_json::from_str(command)?;
                let path = args["path"].as_str().unwrap();
                self.security.validate_path(path)?;
                FsOperations::make_dir(path)?;
                Ok("Directory created successfully".to_string())
            }
            "fs_listdir" => {
                let args: serde_json::Value = serde_json::from_str(command)?;
                let path = args["path"].as_str().unwrap();
                self.security.validate_path(path)?;
                let entries = FsOperations::list_dir(path)?;
                Ok(entries.join("\n"))
            }
            _ => anyhow::bail!("Unknown operation"),
        }
    }

    fn format_operation(&self, operation: &str, args: &serde_json::Value) -> String {
        match operation {
            "fs_readfile" => {
                format!("Read file: {}", args["path"].as_str().unwrap_or(""))
            }
            "fs_writefile" => {
                format!("Write file: {}", args["path"].as_str().unwrap_or(""))
            }
            "fs_makedir" => {
                format!("Create directory: {}", args["path"].as_str().unwrap_or(""))
            }
            "fs_listdir" => {
                format!("List directory: {}", args["path"].as_str().unwrap_or(""))
            }
            _ => format!("{}: {:?}", operation, args),
        }
    }

    async fn add_tool_result(
        &mut self,
        tool_call_id: &str,
        name: &str,
        result: &str,
    ) -> Result<()> {
        let tool_result = ChatMessage {
            role: "tool".to_string(),
            content: Some(result.to_string()),
            tool_calls: None,
            tool_call_id: Some(tool_call_id.to_string()),
            name: Some(name.to_string()),
        };
        self.messages.push(tool_result);
        Ok(())
    }

    fn update_system_message_cwd(&mut self) -> Result<()> {
        use crate::workspace_context::WorkspaceContext;
        use std::path::PathBuf;
        
        if let Some(system_msg) = self.messages.first_mut() {
            if let Some(content) = &mut system_msg.content {
                // Update CWD
                let cwd = std::env::current_dir()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|_| "unknown".to_string());
                *content = content.replace("{{CWD}}", &cwd);
                
                // Update workspace flags
                let workspace_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                let workspace_context = WorkspaceContext::detect(&workspace_dir);
                let flags = workspace_context.to_flags_string();
                *content = content.replace("{{FLAGS}}", &flags);
            }
        }
        Ok(())
    }
}
