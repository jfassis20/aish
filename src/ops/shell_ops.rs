use anyhow::Result;
use std::io::{self, BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::thread;

pub struct ShellExecutor;

impl ShellExecutor {
    pub fn execute(command: &str) -> Result<String> {
        let shell = if cfg!(target_os = "windows") {
            "cmd"
        } else {
            "sh"
        };

        let flag = if cfg!(target_os = "windows") {
            "/C"
        } else {
            "-c"
        };

        let mut child = Command::new(shell)
            .arg(flag)
            .arg(command)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::inherit()) // Inherit stdin from parent process
            .spawn()?;

        let stdout = child.stdout.take().expect("Failed to capture stdout");
        let stderr = child.stderr.take().expect("Failed to capture stderr");

        let mut output = String::new();

        // Spawn threads to read stdout and stderr simultaneously
        let stdout_handle = thread::spawn(move || {
            let mut result = String::new();
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                match line {
                    Ok(line) => {
                        println!("{}", line);
                        io::stdout().flush().ok();
                        result.push_str(&line);
                        result.push('\n');
                    }
                    Err(_) => break,
                }
            }
            result
        });

        let stderr_handle = thread::spawn(move || {
            let mut result = String::new();
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                match line {
                    Ok(line) => {
                        eprintln!("{}", line);
                        io::stderr().flush().ok();
                        result.push_str(&line);
                        result.push('\n');
                    }
                    Err(_) => break,
                }
            }
            result
        });

        // Wait for process to finish
        let status = child.wait()?;

        // Wait for threads to finish reading
        let stdout_output = stdout_handle.join().unwrap();
        let stderr_output = stderr_handle.join().unwrap();

        output.push_str(&stdout_output);
        output.push_str(&stderr_output);

        // Add exit code info if non-zero
        if !status.success() {
            if let Some(code) = status.code() {
                println!("\n--- Process exited with code: {} ---", code);
                output.push_str(&format!("\n--- Process exited with code: {} ---\n", code));
            }
        }

        Ok(output)
    }
}
