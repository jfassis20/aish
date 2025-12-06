use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;

mod cli;
mod config;
mod fs_ops;
mod llm;
mod security;
mod shell;

use cli::app::App;
use config::{Config, ConfigManager};

#[derive(Parser)]
#[command(name = "aish")]
#[command(about = "AI-powered shell assistant", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Automatically accept all commands
    #[arg(long)]
    accept_all: bool,

    /// The prompt to execute
    #[arg(trailing_var_arg = true)]
    prompt: Vec<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize configuration
    Init,
    /// Get or set configuration values
    Config {
        /// Configuration key (e.g., llm.max_tokens)
        key: Option<String>,
        /// Value to set
        value: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config_manager = ConfigManager::new()?;

    match cli.command {
        Some(Commands::Init) => {
            cli::init::run_init_wizard(&config_manager).await?;
        }
        Some(Commands::Config { key, value }) => {
            handle_config_command(&config_manager, key, value)?;
        }
        None => {
            if !config_manager.is_initialized() {
                eprintln!("Configuration not found. Please run: aish init");
                std::process::exit(1);
            }

            let prompt = cli.prompt.join(" ");
            if prompt.is_empty() {
                eprintln!("Usage: aish <prompt>");
                eprintln!("Example: aish edit file ./test.mp4 first 5 seconds and last 3 seconds with ffmpeg");
                std::process::exit(1);
            }

            let config = config_manager.load_config()?;
            let mut app = App::new(config, prompt, cli.accept_all)?;
            app.run().await?;
        }
    }

    Ok(())
}

fn handle_config_command(
    config_manager: &ConfigManager,
    key: Option<String>,
    value: Option<String>,
) -> Result<()> {
    if let Some(k) = key {
        if let Some(v) = value {
            config_manager.set_config_value(&k, &v)?;
            println!(
                "{} Set {} = {}",
                "✓".green(),
                k.bright_cyan(),
                v.bright_yellow()
            );
        } else {
            let val = config_manager.get_config_value(&k)?;
            println!("{} = {}", k.bright_cyan(), val.bright_yellow());
        }
    } else {
        let config = config_manager.load_config()?;
        print_config_pretty(&config);
    }
    Ok(())
}

fn print_config_pretty(config: &Config) {
    println!(
        "{}",
        "╔════════════════════════════════════════════════════════╗".bright_black()
    );
    println!(
        "{}",
        "║              Configuration Settings                  ║"
            .bright_white()
            .bold()
    );
    println!(
        "{}",
        "╚════════════════════════════════════════════════════════╝".bright_black()
    );
    println!();

    // LLM Configuration
    println!("{}", "┌─ LLM Configuration".bright_blue().bold());
    println!(
        "{} {} {}",
        "│".bright_black(),
        "provider:".bright_white(),
        config.llm.provider.bright_yellow()
    );
    println!(
        "{} {} {}",
        "│".bright_black(),
        "api_url:".bright_white(),
        config.llm.api_url.bright_cyan()
    );
    println!(
        "{} {} {}",
        "│".bright_black(),
        "model:".bright_white(),
        config.llm.model.bright_yellow()
    );
    println!(
        "{} {} {}",
        "│".bright_black(),
        "max_tokens:".bright_white(),
        config.llm.max_tokens.to_string().bright_green()
    );
    println!(
        "{}",
        "└─────────────────────────────────────────────────────".bright_black()
    );
    println!();

    // Security Configuration
    println!("{}", "┌─ Security Settings".bright_blue().bold());
    println!(
        "{} {} {}",
        "│".bright_black(),
        "allow_absolute_paths:".bright_white(),
        format_bool(config.security.allow_absolute_paths)
    );
    println!(
        "{} {} {}",
        "│".bright_black(),
        "allow_config_path_access:".bright_white(),
        format_bool(config.security.allow_config_path_access)
    );
    println!(
        "{}",
        "└─────────────────────────────────────────────────────".bright_black()
    );
    println!();

    // Blocked Extensions
    if !config.security.blocked_extensions.is_empty() {
        println!("{}", "┌─ Blocked Extensions".bright_blue().bold());
        for ext in &config.security.blocked_extensions {
            println!("{} {}", "│".bright_black(), ext.bright_red());
        }
        println!(
            "{}",
            "└─────────────────────────────────────────────────────".bright_black()
        );
        println!();
    }

    // Operation Permissions
    println!("{}", "┌─ Operation Permissions".bright_blue().bold());
    println!(
        "{} {} {}",
        "│".bright_black(),
        "fs.makedir:".bright_white(),
        format_bool(config.security.allowed_operations.fs_makedir)
    );
    println!(
        "{} {} {}",
        "│".bright_black(),
        "fs.makefile:".bright_white(),
        format_bool(config.security.allowed_operations.fs_makefile)
    );
    println!(
        "{} {} {}",
        "│".bright_black(),
        "fs.writefile:".bright_white(),
        format_bool(config.security.allowed_operations.fs_writefile)
    );
    println!(
        "{} {} {}",
        "│".bright_black(),
        "fs.readfile:".bright_white(),
        format_bool(config.security.allowed_operations.fs_readfile)
    );
    println!(
        "{} {} {}",
        "│".bright_black(),
        "fs.listdir:".bright_white(),
        format_bool(config.security.allowed_operations.fs_listdir)
    );
    println!(
        "{} {} {}",
        "│".bright_black(),
        "shell:".bright_white(),
        format_bool(config.security.allowed_operations.shell)
    );
    println!(
        "{}",
        "└─────────────────────────────────────────────────────".bright_black()
    );
    println!();

    // Whitelist
    if !config.whitelist.is_empty() {
        println!("{}", "┌─ Whitelist".bright_blue().bold());
        for item in &config.whitelist {
            println!("{} {}", "│".bright_black(), item.bright_green());
        }
        println!(
            "{}",
            "└─────────────────────────────────────────────────────".bright_black()
        );
    }
}

fn format_bool(value: bool) -> ColoredString {
    if value {
        "true".bright_green()
    } else {
        "false".bright_red()
    }
}
