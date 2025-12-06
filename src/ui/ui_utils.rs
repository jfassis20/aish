use colored::*;

const BOX_WIDTH: usize = 56;

/// Render a centered box with custom text and color
pub fn render_box(title: &str, color: Color) {
    println!();
    println!("{}", format_box_top(color));
    println!("{}", format_box_title(title, color));
    println!("{}", format_box_bottom(color));
    println!();
}

/// Render a section header with custom text and color
pub fn render_section(title: &str, color: Color) {
    println!("{}", format_section_header(title, color));
}

/// Render a section footer
pub fn render_section_footer() {
    println!(
        "{}",
        "└─────────────────────────────────────────────────────".bright_black()
    );
    println!();
}

/// Render a line inside a section
pub fn render_section_line(key: &str, value: ColoredString) {
    println!("{} {} {}", "│".bright_black(), key.bright_white(), value);
}

/// Render a list item inside a section
pub fn render_section_item(item: ColoredString) {
    println!("{} {}", "│".bright_black(), item);
}

fn format_box_top(color: Color) -> ColoredString {
    "╔════════════════════════════════════════════════════════╗".color(color)
}

fn format_box_bottom(color: Color) -> ColoredString {
    "╚════════════════════════════════════════════════════════╝".color(color)
}

fn format_box_title(title: &str, color: Color) -> ColoredString {
    let title_len = title.chars().count();
    let is_title_even = title_len % 2 == 0;
    let total_width = BOX_WIDTH - 2; // Subtract the two border characters (║ and ║)

    if title_len >= total_width {
        // If title is too long, truncate it
        let truncated: String = title.chars().take(total_width).collect();
        return format!("║{}║", truncated).color(color).bold();
    }

    let padding_needed = total_width - title_len;
    let left_padding = padding_needed / 2;
    let right_padding = (padding_needed - left_padding) + if is_title_even { 2 } else { 0 };

    let left_spaces = " ".repeat(left_padding);
    let right_spaces = " ".repeat(right_padding);

    format!("║{}{}{}║", left_spaces, title, right_spaces)
        .color(color)
        .bold()
}

fn format_section_header(title: &str, color: Color) -> ColoredString {
    format!("┌─ {}", title).color(color).bold()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_box_title_centering() {
        let title = "Error";
        let formatted = format_box_title(title, Color::Red);
        let formatted_str = format!("{}", formatted);
        assert!(formatted_str.starts_with("║"));
        assert!(formatted_str.ends_with("║"));
    }
}
