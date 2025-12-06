use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::config::Config;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: FunctionCall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub tool_call_id: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatResponse {
    pub choices: Vec<Choice>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Choice {
    pub message: ChatMessage,
}

pub struct LlmClient {
    client: reqwest::Client,
    api_url: String,
    api_key: String,
    model: String,
    max_tokens: u32,
}

impl LlmClient {
    pub fn new(config: &Config, api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_url: config.llm.api_url.clone(),
            api_key,
            model: config.llm.model.clone(),
            max_tokens: config.llm.max_tokens,
        }
    }

    pub async fn chat(&self, messages: Vec<ChatMessage>) -> Result<ChatMessage> {
        let tools = self.get_tools();

        let response = self
            .client
            .post(format!("{}/chat/completions", self.api_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&json!({
                "model": self.model,
                "messages": messages,
                "tools": tools,
                "max_tokens": self.max_tokens,
            }))
            .send()
            .await?;

        let status = response.status();
        let response_text = response.text().await?;

        if !status.is_success() {
            // Try to extract a cleaner error message from JSON response
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&response_text) {
                if let Some(error_obj) = json.get("error") {
                    if let Some(message) = error_obj.get("message").and_then(|m| m.as_str()) {
                        anyhow::bail!("API error ({}): {}", status, message);
                    }
                }
            }
            anyhow::bail!("API error ({}): {}", status, response_text);
        }

        let chat_response: ChatResponse = serde_json::from_str(&response_text).map_err(|e| {
            anyhow::anyhow!(
                "Failed to parse API response: {}\nResponse body: {}",
                e,
                response_text
            )
        })?;

        if chat_response.choices.is_empty() {
            anyhow::bail!("API returned empty choices array");
        }

        Ok(chat_response.choices[0].message.clone())
    }

    pub async fn test_api_key(&self) -> Result<()> {
        // Make a minimal API call to test authentication
        let test_message = ChatMessage {
            role: "user".to_string(),
            content: Some("test".to_string()),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.api_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&json!({
                "model": self.model,
                "messages": [test_message],
                "max_tokens": 5,
            }))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("API returned error: {}", error_text)
        }
    }

    fn get_tools(&self) -> Vec<serde_json::Value> {
        vec![
            json!({
                "type": "function",
                "function": {
                    "name": "execute_shell",
                    "description": "Execute a shell command and return its output",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "command": {
                                "type": "string",
                                "description": "The shell command to execute"
                            }
                        },
                        "required": ["command"]
                    }
                }
            }),
            json!({
                "type": "function",
                "function": {
                    "name": "fs_readfile",
                    "description": "Read the contents of a file",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "Relative path to the file"
                            }
                        },
                        "required": ["path"]
                    }
                }
            }),
            json!({
                "type": "function",
                "function": {
                    "name": "fs_writefile",
                    "description": "Write content to a file",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "Relative path to the file"
                            },
                            "content": {
                                "type": "string",
                                "description": "Content to write"
                            }
                        },
                        "required": ["path", "content"]
                    }
                }
            }),
            json!({
                "type": "function",
                "function": {
                    "name": "fs_makedir",
                    "description": "Create a directory",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "Relative path to the directory"
                            }
                        },
                        "required": ["path"]
                    }
                }
            }),
            json!({
                "type": "function",
                "function": {
                    "name": "fs_listdir",
                    "description": "List contents of a directory",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "Relative path to the directory"
                            }
                        },
                        "required": ["path"]
                    }
                }
            }),
        ]
    }
}
