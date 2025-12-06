# AISH


> AI-powered shell assistant that helps you execute commands and file operations through natural language.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)

`aish` is an intelligent command-line assistant that uses Large Language Models (LLMs) to understand your natural language requests and execute shell commands and file operations safely and interactively.

## Features

- 🤖 **AI-Powered**: Uses LLMs to understand natural language and execute tasks
- 🔒 **Security First**: Built-in security features with path validation, operation permissions, and whitelist support
- 🎯 **Interactive**: Beautiful CLI interface with approval prompts for each operation
- 💬 **Conversation Mode**: Keep conversations open with `-i` flag for continuous interaction
- 🧠 **Context Aware**: Automatically detects workspace context including:
  - Programming languages and versions (Node.js, Python, Java, Rust, Go, PHP, Ruby, .NET)
  - Package managers and build tools
  - Web and backend frameworks
  - Testing frameworks and linters
  - Docker, Kubernetes, and container registries
  - CI/CD platforms (GitHub Actions, GitLab CI, Jenkins, etc.)
  - Cloud providers (AWS, GCP, Azure)
  - Infrastructure as Code tools (Terraform, Pulumi, CDK)
  - Databases and development tools
- 🔧 **Flexible**: Supports multiple LLM providers (OpenAI, OpenRouter, Custom)
- 📁 **File Operations**: Read, write, create directories, and list files
- 🚀 **Shell Commands**: Execute shell commands with real-time output
- ⚙️ **Configurable**: Easy-to-use configuration system with interactive wizard

## Installation

### From Source

```bash
git clone https://github.com/sammwyy/aish.git
cd aish
cargo build --release
```

The binary will be available at `target/release/aish`.

### Requirements

- Rust 1.70 or later
- An API key from an LLM provider (OpenAI, OpenRouter, etc.)

## Quick Start

1. **Initialize configuration:**
   ```bash
   aish init
   ```
   This will guide you through setting up your LLM provider, API key, and preferences.

2. **Start using aish:**
   ```bash
   aish "list all Python files in the current directory"
   ```

3. **Try interactive mode:**
   ```bash
   aish -i "show me disk usage"
   ```
   This starts an interactive session where you can continue the conversation.

4. **View configuration:**
   ```bash
   aish config
   ```

## Usage

### Basic Commands

```bash
# Execute a simple task
aish "show me disk usage"

# File operations
aish "create a new directory called 'projects'"

# Complex workflows
aish "find all .log files older than 30 days and delete them"

# Auto-approve all operations (use with caution)
aish --accept-all "run tests and commit if they pass"
```

### Interactive Mode

Interactive mode (`-i` or `--interactive`) keeps the conversation open, allowing you to have a continuous dialogue with the AI assistant. The full conversation history is maintained, enabling references to previous commands and results.

```bash
# Start interactive mode (prompts for commands)
aish -i

# Start interactive mode with an initial command
aish -i "list all Python files"

# Interactive mode with auto-approve
aish -i --accept-all "run tests"

# Exit interactive mode by typing 'quit' or 'exit', or press Ctrl+C
```

**Features:**
- Full conversation history maintained across all prompts
- Type `quit` or `exit` to end the session
- Press `Ctrl+C` to exit at any time
- Empty prompts are ignored (allows you to skip)
- Errors don't terminate the session, allowing you to continue

### Configuration Commands

```bash
# View all configuration
aish config

# Get a specific value
aish config llm.model

# Set a configuration value
aish config llm.max_tokens 8192
aish config security.allow_absolute_paths true

# View the current system prompt (with placeholders replaced)
aish showsystem

# Regenerate the system prompt template
aish regen
```

## Configuration

Configuration is stored in `~/.aish/config.toml` and API keys in `~/.aish/tokens.env`.

### LLM Configuration

```toml
[llm]
provider = "OpenAI"
api_url = "https://api.openai.com/v1"
model = "gpt-4"
max_tokens = 4096
```

**Supported Providers:**
- OpenAI
- OpenRouter
- Custom (any OpenAI-compatible API)

### Security Configuration

```toml
[security]
allow_absolute_paths = false
allow_config_path_access = false
blocked_extensions = [".env"]

[security.allowed_operations]
"fs.makedir" = true
"fs.makefile" = true
"fs.writefile" = true
"fs.readfile" = true
"fs.listdir" = true
shell = true
```

### Whitelist

The whitelist is used for auto-execution when `--accept-all` is enabled. Commands matching whitelist patterns will be automatically approved.

```toml
whitelist = [
  "^ls.*",
  "^cat.*\\.txt$",
  "^git status$"
]
```

**Note:** The whitelist only applies when using `--accept-all`. When running normally, all operations require user approval regardless of whitelist status.

### .aishignore

Create a `.aishignore` file in `~/.aish/` to block access to specific file patterns (similar to `.gitignore`):

```
*.env
*.key
secrets/*
.git/*
node_modules/*
```

## Examples

### File Operations

```bash
# Create pro
ect structure
aish "create a Python project with src, tests, and docs folders"

# Read and analyze files
aish "read all Python files in src/ and summarize the code structure"

# Batch operations
aish "rename all .txt files to .md in the current directory"
```

### Shell Commands

```bash
# System information
aish "show me disk usage and the top 5 largest directories"

# Development tasks
aish "run the tests and if they pass, commit with message 'fix: tests passing'"

# Media processing
aish "convert all .mov files to .mp4 with H.264 encoding"
```

### Complex Workflows

```bash
# Data analysis
aish "analyze server.log, find errors in the last hour, and create a summary"

# Deployment
aish "backup the database, build the Docker image, and push to registry"

# Cleanup
aish "find and delete all .log files older than 30 days"
```

## Security Features

1. **Approval Required**: Every operation requires explicit approval (unless whitelisted with `--accept-all`)
2. **Path Validation**: Prevents access to config directory and absolute paths by default
3. **Extension Blocking**: Blocks sensitive file extensions like `.env`
4. **Operation Permissions**: Granular control over what operations are allowed
5. **Pattern Matching**: `.aishignore` file prevents access to sensitive patterns
6. **Whitelist**: Auto-approve specific command patterns (only with `--accept-all`)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Author

**Sammwy**

- GitHub: [@sammwyy](https://github.com/sammwyy)

## Acknowledgments

- Built with [Rust](https://www.rust-lang.org/)
- Uses [inquire](https://github.com/mikaelmello/inquire) for beautiful CLI prompts
- Uses [colored](https://github.com/mackwic/colored) for terminal colors

## Disclaimer

This tool executes shell commands and file operations based on AI interpretation. Always review proposed operations before approval. Use `--accept-all` with extreme caution and only with trusted whitelisted commands.

