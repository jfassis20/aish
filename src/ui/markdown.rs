use termimad::MadSkin;

pub fn render_markdown(text: &str) {
    let mut skin = MadSkin::default();

    // Customize colors for better appearance
    skin.set_headers_fg(termimad::crossterm::style::Color::Cyan);
    skin.bold.set_fg(termimad::crossterm::style::Color::Yellow);
    skin.italic
        .set_fg(termimad::crossterm::style::Color::Magenta);
    skin.code_block
        .set_fg(termimad::crossterm::style::Color::Green);
    skin.inline_code
        .set_fg(termimad::crossterm::style::Color::Green);
    skin.quote_mark
        .set_fg(termimad::crossterm::style::Color::Yellow);

    // Render the markdown
    skin.print_text(text);
}
