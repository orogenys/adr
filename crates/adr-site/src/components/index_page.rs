use adr_core::AdrDocument;

use super::{IndexTableComponent, LayoutComponent, PageKind, Render};

pub struct IndexPageComponent<'a> {
    pub documents: &'a [AdrDocument],
    pub home_label: &'a str,
    pub footer: &'a str,
}

impl Render for IndexPageComponent<'_> {
    fn render(&self) -> String {
        let body = IndexTableComponent {
            documents: self.documents,
        }
        .render();

        LayoutComponent {
            page_kind: PageKind::Index,
            title: "ADR Index",
            heading: "Architectural Decision Records",
            heading_html: None,
            subtitle: Some("Static index of all ADRs in this repository."),
            subtitle_html: None,
            body: &body,
            home_label: self.home_label,
            footer: self.footer,
        }
        .render()
    }
}
