use colored::*;

const BOX_WIDTH: usize = 56;
const BOX_CONTENT_WIDTH: usize = BOX_WIDTH - 2;

const BOX_TOP_LEFT: &str = "╔";
const BOX_TOP_RIGHT: &str = "╗";
const BOX_BOTTOM_LEFT: &str = "╚";
const BOX_BOTTOM_RIGHT: &str = "╝";
const BOX_HORIZONTAL: &str = "═";
const BOX_VERTICAL: &str = "║";

const SECTION_TOP_LEFT: &str = "┌";
const SECTION_BOTTOM_LEFT: &str = "└";
const SECTION_HORIZONTAL: &str = "─";
const SECTION_VERTICAL: &str = "│";

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
    println!("{}", format_section_footer());
    println!();
}

/// Render a line inside a section with key-value pair
pub fn render_section_line(key: &str, value: ColoredString) {
    println!(
        "{} {} {}",
        SECTION_VERTICAL.bright_black(),
        key.bright_white(),
        value
    );
}

/// Render a list item inside a section
pub fn render_section_item(item: ColoredString) {
    println!("{} {}", SECTION_VERTICAL.bright_black(), item);
}

fn format_box_top(color: Color) -> ColoredString {
    format!(
        "{}{}{}",
        BOX_TOP_LEFT,
        BOX_HORIZONTAL.repeat(BOX_CONTENT_WIDTH),
        BOX_TOP_RIGHT
    )
    .color(color)
}

fn format_box_bottom(color: Color) -> ColoredString {
    format!(
        "{}{}{}",
        BOX_BOTTOM_LEFT,
        BOX_HORIZONTAL.repeat(BOX_CONTENT_WIDTH),
        BOX_BOTTOM_RIGHT
    )
    .color(color)
}

fn format_box_title(title: &str, color: Color) -> ColoredString {
    let title_len = title.chars().count();

    if title_len >= BOX_CONTENT_WIDTH {
        let truncated: String = title.chars().take(BOX_CONTENT_WIDTH).collect();
        return format!("{}{}{}", BOX_VERTICAL, truncated, BOX_VERTICAL)
            .color(color)
            .bold();
    }

    let padding_needed = BOX_CONTENT_WIDTH - title_len;
    let left_padding = padding_needed / 2;
    let right_padding = padding_needed - left_padding;

    format!(
        "{}{}{}{}{}",
        BOX_VERTICAL,
        " ".repeat(left_padding),
        title,
        " ".repeat(right_padding),
        BOX_VERTICAL
    )
    .color(color)
    .bold()
}

fn format_section_header(title: &str, color: Color) -> ColoredString {
    format!("{}{} {}", SECTION_TOP_LEFT, SECTION_HORIZONTAL, title)
        .color(color)
        .bold()
}

fn format_section_footer() -> ColoredString {
    format!(
        "{}{}",
        SECTION_BOTTOM_LEFT,
        SECTION_HORIZONTAL.repeat(BOX_CONTENT_WIDTH - 1)
    )
    .bright_black()
}
