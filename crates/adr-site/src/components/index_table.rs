use adr_core::AdrDocument;

use super::{Render, escape_html, index_href_to_adr};

pub struct IndexTableComponent<'a> {
    pub documents: &'a [AdrDocument],
}

impl Render for IndexTableComponent<'_> {
    fn render(&self) -> String {
        if self.documents.is_empty() {
            return "<p class=\"empty\">No ADRs found.</p>".to_string();
        }

        let mut html = String::from(
            "<table class=\"index-table\"><thead><tr><th>ID</th><th>Title</th><th>Status</th><th>Category</th><th>Date</th></tr></thead><tbody>",
        );

        for document in self.documents {
            let href = index_href_to_adr(document.id);
            html.push_str("<tr>");
            html.push_str(&format!(
                "<td><a href=\"{}\">{}</a></td>",
                href,
                escape_html(&document.id.as_ref().to_string())
            ));
            html.push_str(&format!(
                "<td><a href=\"{}\">{}</a></td>",
                href,
                escape_html(&document.title)
            ));
            html.push_str(&format!(
                "<td>{}</td>",
                escape_html(&document.status.to_string())
            ));
            html.push_str(&format!(
                "<td>{}</td>",
                escape_html(document.category.as_deref().unwrap_or("-"))
            ));
            html.push_str(&format!(
                "<td>{}</td>",
                escape_html(&document.date.to_string())
            ));
            html.push_str("</tr>");
        }

        html.push_str("</tbody></table>");
        html
    }
}
