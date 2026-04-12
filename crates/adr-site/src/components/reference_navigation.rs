use std::collections::BTreeMap;

use adr_core::{AdrDocument, AdrId, AdrIndexEntry};

use super::{PageContext, Render, escape_html, page_href_to_adr};

pub struct ReferenceNavigationComponent<'a> {
    pub entry: &'a AdrIndexEntry,
    pub documents_by_id: &'a BTreeMap<AdrId, &'a AdrDocument>,
}

impl Render for ReferenceNavigationComponent<'_> {
    fn render(&self) -> String {
        let entry = self.entry;
        if entry.outgoing.is_empty() && entry.incoming.is_empty() {
            return String::new();
        }

        let outgoing = if entry.outgoing.is_empty() {
            "<p class=\"empty\">No outgoing references.</p>".to_string()
        } else {
            let mut html = String::from("<ul>");
            for reference in &entry.outgoing {
                if reference.broken {
                    html.push_str(&format!(
                        "<li><span class=\"broken-ref\">{}</span> <span class=\"ref-note\">({} missing)</span></li>",
                        escape_html(&reference.target.to_string()),
                        escape_html(&reference.kind.to_string())
                    ));
                } else if let Some(target) = self.documents_by_id.get(&reference.target.id()) {
                    html.push_str(&format!(
                        "<li><a href=\"{}\">{} — {}</a> <span class=\"ref-note\">({})</span></li>",
                        page_href_to_adr(PageContext::Adr, target.id),
                        escape_html(&target.id.as_ref().to_string()),
                        escape_html(&target.title),
                        escape_html(&reference.kind.to_string())
                    ));
                }
            }
            html.push_str("</ul>");
            html
        };

        let incoming = if entry.incoming.is_empty() {
            "<p class=\"empty\">No incoming references.</p>".to_string()
        } else {
            let mut html = String::from("<ul>");
            for reference in &entry.incoming {
                if let Some(source) = self.documents_by_id.get(&reference.source.id()) {
                    html.push_str(&format!(
                        "<li><a href=\"{}\">{} — {}</a> <span class=\"ref-note\">({})</span></li>",
                        page_href_to_adr(PageContext::Adr, source.id),
                        escape_html(&source.id.as_ref().to_string()),
                        escape_html(&source.title),
                        escape_html(&reference.kind.to_string())
                    ));
                }
            }
            html.push_str("</ul>");
            html
        };

        format!(
            concat!(
                "<section class=\"related\">",
                "<div class=\"related-columns\">",
                "<section><h2>References from this ADR</h2>{}</section>",
                "<section><h2>Referenced by</h2>{}</section>",
                "</div>",
                "</section>"
            ),
            outgoing, incoming
        )
    }
}
