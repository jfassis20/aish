use colored::*;
use serde_json::Value;

use super::ui_utils::render_box;

pub fn format_error(error: &anyhow::Error) {
    let error_str = error.to_string();

    render_box("Error", Color::Red);

    // Detect different types of errors
    if error_str.contains("API error") || error_str.contains("Failed to parse API response") {
        format_api_error(&error_str);
    } else {
        // Generic error formatting
        println!("{}", "Description:".bright_white().bold());
        println!("{}", format!("  {}", error_str).bright_red());
    }
}

fn format_api_error(error_str: &str) {
    let mut message_shown = false;

    // Check if this is a "Failed to parse API response" error with Response body
    if error_str.contains("Failed to parse API response") {
        // Extract the parsing error message
        if let Some(colon_pos) = error_str.find(':') {
            if let Some(response_body_pos) = error_str.find("Response body:") {
                let parse_error = error_str[colon_pos + 1..response_body_pos].trim();
                if !parse_error.is_empty() {
                    println!("{}", "Parse Error:".bright_white().bold());
                    println!("{}", format!("  {}", parse_error).bright_yellow());
                    println!();
                }

                // Extract and parse the response body JSON
                let response_body = error_str[response_body_pos + "Response body:".len()..].trim();
                if let Some(json_start) = response_body.find('{') {
                    if let Some(json_end) = response_body.rfind('}') {
                        let json_str = &response_body[json_start..=json_end];
                        if let Ok(json) = serde_json::from_str::<Value>(json_str) {
                            extract_error_from_json(&json, &mut message_shown);
                            return;
                        }
                    }
                }
            } else {
                // No response body, just show the parse error
                if let Some(colon_pos) = error_str.find(':') {
                    let message = error_str[colon_pos + 1..].trim();
                    if !message.is_empty() {
                        println!("{}", "Message:".bright_white().bold());
                        println!("{}", format!("  {}", message).bright_red());
                        println!();
                    }
                }
                return;
            }
        }
        return;
    }

    // Extract status code from "API error (XXX)" format
    let status = if let Some(start) = error_str.find('(') {
        if let Some(end) = error_str[start..].find(')') {
            Some(&error_str[start + 1..start + end])
        } else {
            None
        }
    } else {
        None
    };

    if let Some(status_code) = status {
        println!("{}", "Status:".bright_white().bold());
        println!("{}", format!("  {}", status_code).bright_yellow());
        println!();
    }

    // Try to find and parse JSON in the error string
    if let Some(json_start) = error_str.find('{') {
        if let Some(json_end) = error_str.rfind('}') {
            let json_str = &error_str[json_start..=json_end];
            if let Ok(json) = serde_json::from_str::<Value>(json_str) {
                extract_error_from_json(&json, &mut message_shown);
            }
        }
    }

    // If no JSON found, extract message after the colon
    if !message_shown {
        if let Some(colon_pos) = error_str.find(':') {
            let message = error_str[colon_pos + 1..].trim();
            // Skip if message is empty or just whitespace
            if !message.is_empty() && !message.chars().all(|c| c.is_whitespace() || c == '\n') {
                println!("{}", "Message:".bright_white().bold());
                println!("{}", format!("  {}", message.lines().next().unwrap_or(message)).bright_red());
                println!();
            }
        }
    }
}

fn extract_error_from_json(json: &Value, message_shown: &mut bool) {
    if let Some(error_obj) = json.get("error") {
        // Extract message
        if let Some(message) = error_obj.get("message").and_then(|m| m.as_str()) {
            println!("{}", "Message:".bright_white().bold());
            println!("{}", format!("  {}", message).bright_red());
            println!();
            *message_shown = true;
        }

        // Extract code if available
        if let Some(code) = error_obj.get("code").and_then(|c| c.as_u64()) {
            println!("{}", "Error Code:".bright_white().bold());
            println!("{}", format!("  {}", code).bright_yellow());
            println!();
        } else if let Some(code) = error_obj.get("code").and_then(|c| c.as_str()) {
            println!("{}", "Error Code:".bright_white().bold());
            println!("{}", format!("  {}", code).bright_yellow());
            println!();
        }

        // Extract type if available
        if let Some(error_type) = error_obj.get("type").and_then(|t| t.as_str()) {
            println!("{}", "Error Type:".bright_white().bold());
            println!("{}", format!("  {}", error_type).bright_yellow());
            println!();
        }

        // Extract metadata if available
        if let Some(metadata) = error_obj.get("metadata") {
            if let Some(provider) = metadata.get("provider_name").and_then(|p| p.as_str()) {
                println!("{}", "Provider:".bright_white().bold());
                println!("{}", format!("  {}", provider).bright_yellow());
                println!();
            }
        }
    } else if !*message_shown {
        // Fallback: show the whole JSON if no error object found
        println!("{}", "Details:".bright_white().bold());
        println!("{}", format!("  {}", json).bright_red());
        println!();
    }
}
