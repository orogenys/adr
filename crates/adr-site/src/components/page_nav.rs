use adr_core::AdrDocument;

use super::{PageContext, Render, escape_html, page_href_to_adr};

pub struct PageNavComponent<'a> {
    pub previous: Option<&'a AdrDocument>,
    pub next: Option<&'a AdrDocument>,
}

impl Render for PageNavComponent<'_> {
    fn render(&self) -> String {
        let mut items = Vec::new();

        if let Some(previous) = self.previous {
            items.push(format!(
                "<li><a href=\"{}\">← {} {}</a></li>",
                page_href_to_adr(PageContext::Adr, previous.id),
                escape_html(&previous.id.as_ref().to_string()),
                escape_html(&previous.title)
            ));
        }

        if let Some(next) = self.next {
            items.push(format!(
                "<li><a href=\"{}\">{} {} →</a></li>",
                page_href_to_adr(PageContext::Adr, next.id),
                escape_html(&next.id.as_ref().to_string()),
                escape_html(&next.title)
            ));
        }

        if items.is_empty() {
            String::new()
        } else {
            format!("<nav class=\"page-nav\"><ul>{}</ul></nav>", items.join(""))
        }
    }
}
