use adr_core::AdrId;

pub trait Render {
    fn render(&self) -> String;
}

#[derive(Debug, Clone, Copy)]
pub enum PageKind {
    Index,
    Adr,
}

#[derive(Debug, Clone, Copy)]
pub enum PageContext {
    Adr,
}

pub fn index_href_to_adr(target: AdrId) -> String {
    format!("adr/{target}/index.html")
}

pub fn page_href_to_adr(page: PageContext, target: AdrId) -> String {
    match page {
        PageContext::Adr => format!("../{target}/index.html"),
    }
}

pub fn render_template(template: &str, replacements: &[(&str, String)]) -> String {
    let mut rendered = template.to_string();
    for (placeholder, value) in replacements {
        rendered = rendered.replace(placeholder, value);
    }
    rendered
}

pub fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
