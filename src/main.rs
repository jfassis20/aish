use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;

mod ai;
mod cli;
mod config;
mod ops;
mod security;
mod ui;

use cli::app::App;
use config::{Config, ConfigManager};
use inquire::Text;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::signal;
use ui::{
    render_box, render_section, render_section_footer, render_section_item, render_section_line,
};

#[derive(Parser)]
#[command(name = "aish")]
#[command(about = "AI-powered shell assistant", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Automatically accept all commands
    #[arg(long)]
    accept_all: bool,

    /// Interactive mode: keep conversation open and prompt for new inputs
    #[arg(short, long)]
    interactive: bool,

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
    /// Regenerate system prompt template
    Regen,
    /// Show the current system prompt
    Showsystem,
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
        Some(Commands::Regen) => {
            regenerate_system_prompt(&config_manager)?;
        }
        Some(Commands::Showsystem) => {
            show_system_prompt(&config_manager)?;
        }
        None => {
            if !config_manager.is_initialized() {
                eprintln!("Configuration not found. Please run: aish init");
                std::process::exit(1);
            }

            let config = config_manager.load_config()?;
            let prompt = cli.prompt.join(" ");

            if cli.interactive {
                let initial_prompt = if prompt.is_empty() {
                    None
                } else {
                    Some(prompt)
                };
                run_interactive_mode(config, cli.accept_all, initial_prompt).await?;
            } else {
                if prompt.is_empty() {
                    eprintln!("Usage: aish <prompt>");
                    eprintln!("Example: aish edit file ./test.mp4 first 5 seconds and last 3 seconds with ffmpeg");
                    std::process::exit(1);
                }

                let mut app = App::new(config, prompt, cli.accept_all)?;
                if let Err(e) = app.run().await {
                    ui::format_error(&e);
                    std::process::exit(1);
                }
            }
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
    render_box("Configuration Settings", Color::BrightWhite);

    // LLM Configuration
    render_section("LLM Configuration", Color::BrightBlue);
    render_section_line("llm.provider:", config.llm.provider.bright_yellow());
    render_section_line("llm.api_url:", config.llm.api_url.bright_cyan());
    render_section_line("llm.model:", config.llm.model.bright_yellow());
    render_section_line(
        "llm.max_tokens:",
        config.llm.max_tokens.to_string().bright_green(),
    );
    render_section_footer();

    // Security Configuration
    render_section("Security Settings", Color::BrightBlue);
    render_section_line(
        "security.allow_absolute_paths:",
        format_bool(config.security.allow_absolute_paths),
    );
    render_section_line(
        "security.allow_config_path_access:",
        format_bool(config.security.allow_config_path_access),
    );
    render_section_footer();

    // Blocked Extensions
    if !config.security.blocked_extensions.is_empty() {
        render_section("Blocked Extensions", Color::BrightBlue);
        for ext in &config.security.blocked_extensions {
            render_section_item(ext.bright_red());
        }
        render_section_footer();
    }

    // Operation Permissions
    render_section("Operation Permissions", Color::BrightBlue);
    render_section_line(
        "fs.makedir:",
        format_bool(config.security.allowed_operations.fs_makedir),
    );
    render_section_line(
        "fs.makefile:",
        format_bool(config.security.allowed_operations.fs_makefile),
    );
    render_section_line(
        "fs.writefile:",
        format_bool(config.security.allowed_operations.fs_writefile),
    );
    render_section_line(
        "fs.readfile:",
        format_bool(config.security.allowed_operations.fs_readfile),
    );
    render_section_line(
        "fs.listdir:",
        format_bool(config.security.allowed_operations.fs_listdir),
    );
    render_section_line(
        "security.shell:",
        format_bool(config.security.allowed_operations.shell),
    );
    render_section_footer();

    // Whitelist
    if !config.whitelist.is_empty() {
        render_section("Whitelist", Color::BrightBlue);
        for item in &config.whitelist {
            render_section_item(item.bright_green());
        }
        render_section_footer();
    }
}

fn format_bool(value: bool) -> ColoredString {
    if value {
        "true".bright_green()
    } else {
        "false".bright_red()
    }
}

fn regenerate_system_prompt(config_manager: &ConfigManager) -> Result<()> {
    use colored::*;

    let default_prompt = include_str!("../data/system_prompt.txt");
    let prompt_path = config_manager.get_system_prompt_path();

    std::fs::create_dir_all(prompt_path.parent().unwrap())?;
    std::fs::write(&prompt_path, default_prompt)?;

    println!(
        "{} {}",
        "✓".green(),
        format!("System prompt regenerated at: {:?}", prompt_path).bright_green()
    );

    Ok(())
}

fn show_system_prompt(config_manager: &ConfigManager) -> Result<()> {
    use colored::*;

    let prompt = config_manager.load_system_prompt()?;
    let prompt_path = config_manager.get_system_prompt_path();

    render_box("System Prompt", Color::BrightWhite);
    println!("{}", format!("Path: {:?}", prompt_path).bright_cyan());
    println!();
    println!("{}", "─".repeat(60).bright_black());
    println!();
    println!("{}", prompt);
    println!();
    println!("{}", "─".repeat(60).bright_black());

    Ok(())
}

async fn run_interactive_mode(
    config: Config,
    accept_all: bool,
    initial_prompt: Option<String>,
) -> Result<()> {
    use colored::*;

    println!(
        "{}",
        "→ Interactive mode started. Type 'quit' or 'exit' to end, or press Ctrl+C".bright_cyan()
    );
    println!();

    // Initialize app
    let mut app = if let Some(prompt) = initial_prompt.clone() {
        // If initial prompt provided, use it
        App::new(config, prompt, accept_all)?
    } else {
        // Otherwise start empty
        App::new_empty(config, accept_all)?
    };

    // If initial prompt was provided, run it first
    if initial_prompt.is_some() {
        app.run().await?;
    }

    // Set up Ctrl+C handler
    let ctrl_c_pressed = Arc::new(AtomicBool::new(false));
    let ctrl_c_flag = ctrl_c_pressed.clone();

    tokio::spawn(async move {
        if let Ok(()) = signal::ctrl_c().await {
            ctrl_c_flag.store(true, Ordering::Relaxed);
        }
    });

    loop {
        // Check if Ctrl+C was pressed
        if ctrl_c_pressed.load(Ordering::Relaxed) {
            println!();
            println!("{}", "→ Exiting interactive mode...".bright_cyan());
            break;
        }

        let prompt = Text::new("aish>")
            .with_help_message("Enter your command or 'quit'/'exit' to exit")
            .prompt();

        match prompt {
            Ok(p) => {
                let p = p.trim();
                if p.is_empty() {
                    continue;
                }

                // Check for exit commands
                let lower = p.to_lowercase();
                if lower == "quit" || lower == "exit" {
                    println!();
                    println!("{}", "→ Exiting interactive mode...".bright_cyan());
                    break;
                }

                // Add user message and run
                app.add_user_message(p.to_string());
                match app.run().await {
                    Ok(_) => {
                        // Continue loop for next prompt
                    }
                    Err(e) => {
                        ui::format_error(&e);
                        // Continue loop even on error
                    }
                }
            }
            Err(_) => {
                // User cancelled with Esc or similar, or Ctrl+C
                if ctrl_c_pressed.load(Ordering::Relaxed) {
                    println!();
                    println!("{}", "→ Exiting interactive mode...".bright_cyan());
                }
                break;
            }
        }
    }

    Ok(())
}
