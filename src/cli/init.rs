use anyhow::Result;
use inquire::{Confirm, Password, Select, Text};

use crate::config::{Config, ConfigManager};
use crate::llm::LlmClient;

const PROVIDERS: &[(&str, &str, &str)] = &[
    ("OpenAI", "https://api.openai.com/v1", "gpt-4"),
    ("OpenRouter", "https://openrouter.ai/api/v1", "openai/gpt-4"),
    ("Custom", "", ""),
];

pub async fn run_init_wizard(config_manager: &ConfigManager) -> Result<()> {
    let mut config = Config::default();

    // 1. Select Provider
    let provider_options: Vec<&str> = PROVIDERS.iter().map(|p| p.0).collect();
    let provider_choice = Select::new("provider:", provider_options)
        .with_help_message("Use ↑/↓ to navigate, Enter to select")
        .prompt()?;

    let provider_idx = PROVIDERS
        .iter()
        .position(|p| p.0 == provider_choice)
        .unwrap();
    let (_, default_url, default_model) = PROVIDERS[provider_idx];

    config.llm.provider = provider_choice.to_string();

    // 2. API URL (only if Custom)
    if provider_choice == "Custom" {
        let api_url = Text::new("api_url:")
            .with_help_message("Enter the API endpoint URL")
            .prompt()?;
        config.llm.api_url = api_url;
    } else {
        config.llm.api_url = default_url.to_string();
    }

    // 3. API Key
    let api_key = Password::new("api_key:")
        .with_help_message("Your API key will be hidden")
        .prompt()?;

    // 4. Test API Key
    let test_api = Confirm::new("test_api_key:")
        .with_default(true)
        .with_help_message("y/n, Enter for default")
        .prompt()?;

    if test_api {
        print!("test_api_key: Testing... ");
        std::io::Write::flush(&mut std::io::stdout())?;

        let test_client = LlmClient::new(&config, api_key.clone());
        match test_client.test_api_key().await {
            Ok(_) => {
                println!("✓ Valid");
            }
            Err(e) => {
                let error_msg = e.to_string();
                let short_error = error_msg.lines().next().unwrap_or("Invalid");
                println!("✗ Error: {}", short_error);
            }
        }
    }

    // 5. Model (only if Custom)
    if provider_choice == "Custom" {
        let model = Text::new("model:")
            .with_help_message("Enter the model name")
            .prompt()?;
        config.llm.model = model;
    } else {
        config.llm.model = default_model.to_string();
    }

    // 6. Max Tokens
    let max_tokens_str = Text::new("max_tokens:")
        .with_default(&config.llm.max_tokens.to_string())
        .with_help_message("Maximum tokens per response")
        .prompt()?;

    let max_tokens = max_tokens_str
        .parse::<u32>()
        .unwrap_or(config.llm.max_tokens);
    config.llm.max_tokens = max_tokens;

    // 7. Allow Absolute Paths
    let allow_abs = Confirm::new("allow_absolute_paths:")
        .with_default(config.security.allow_absolute_paths)
        .with_help_message("y/n, Enter for default")
        .prompt()?;
    config.security.allow_absolute_paths = allow_abs;

    // 8. Allow Config Access
    let allow_config = Confirm::new("allow_config_access:")
        .with_default(config.security.allow_config_path_access)
        .with_help_message("y/n, Enter for default")
        .prompt()?;
    config.security.allow_config_path_access = allow_config;

    // Save configuration
    config_manager.save_config(&config)?;
    config_manager.save_api_key(&api_key)?;

    println!("\n✓ Configuration saved successfully!");
    println!("You can now use aish with: aish <your prompt>");

    Ok(())
}
