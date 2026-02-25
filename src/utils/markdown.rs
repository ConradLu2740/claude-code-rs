use pulldown_cmark::{Parser, html, Options};
use termimad::MadSkin;

pub fn render_markdown(content: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(content, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

pub fn print_markdown(content: &str) {
    let skin = MadSkin::default_dark();
    skin.print_text(content);
}

pub fn strip_markdown(content: &str) -> String {
    let parser = Parser::new(content);
    let mut plain = String::new();
    
    for event in parser {
        use pulldown_cmark::Event;
        match event {
            Event::Text(text) | Event::Code(text) => {
                plain.push_str(&text);
            }
            Event::SoftBreak | Event::HardBreak => {
                plain.push('\n');
            }
            _ => {}
        }
    }
    
    plain
}
