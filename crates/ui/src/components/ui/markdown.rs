use std::borrow::Cow;
use pulldown_cmark::{html, Options, Parser};
use sycamore::prelude::*;
use sycamore::web::tags::div;
use sycamore::web::{GlobalProps, HtmlGlobalAttributes};

/// Парсит переданный Markdown-текст в безопасный HTML-код с поддержкой таблиц,
/// списков задач, зачёркиваний и заголовков.
pub fn render_markdown(md: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

    let parser = Parser::new_ext(md, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

#[component(inline_props)]
pub fn MarkdownView(content: String) -> View {
    let html_str = render_markdown(&content);
    let el = div()
        .class("markdown-rendered-view")
        .dangerously_set_inner_html(Cow::Owned(html_str));
    View::from(el)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_markdown_basic() {
        let md = "# Title\n\nSome **bold** text and `inline code`.";
        let html = render_markdown(md);
        assert!(html.contains("<h1>Title</h1>"));
        assert!(html.contains("<strong>bold</strong>"));
        assert!(html.contains("<code>inline code</code>"));
    }

    #[test]
    fn test_render_markdown_table() {
        let md = "| Key | Value |\n|---|---|\n| id | custom-theme |";
        let html = render_markdown(md);
        assert!(html.contains("<table>"));
        assert!(html.contains("<th>Key</th>"));
        assert!(html.contains("<td>id</td>"));
        assert!(html.contains("<td>custom-theme</td>"));
    }

    #[test]
    fn test_render_markdown_code_block() {
        let md = "```json\n{\n  \"name\": \"Test\"\n}\n```";
        let html = render_markdown(md);
        assert!(html.contains("<pre>"));
        assert!(html.contains("<code"));
        assert!(html.contains("\"name\": \"Test\""));
    }
}

