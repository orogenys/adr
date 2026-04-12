use adr_core::AdrDocument;

use super::{Render, escape_html};

pub struct AdrMetaComponent<'a> {
    pub document: &'a AdrDocument,
}

impl Render for AdrMetaComponent<'_> {
    fn render(&self) -> String {
        let document = self.document;
        let status = format!(
            "<span class=\"meta-label\">Status:</span> <span class=\"pill pill-status pill-status-{}\">{}</span>",
            escape_html(&document.status.to_string()),
            escape_html(&document.status.to_string())
        );

        let category = document.category.as_deref().map(|category| {
            format!(
                "<span class=\"meta-label\">Category:</span> <span class=\"pill pill-category\">{}</span>",
                escape_html(category)
            )
        });

        let mut parts = vec![status];
        if let Some(category) = category {
            parts.push(category);
        }
        parts.push(format!(
            "<span class=\"meta-date\">{}</span>",
            escape_html(&document.date.to_string())
        ));

        format!(
            "<p class=\"adr-meta-inline\">{}</p>",
            parts.join(" <span class=\"meta-sep\">·</span> ")
        )
    }
}
