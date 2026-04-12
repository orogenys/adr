use std::collections::{BTreeMap, BTreeSet};

use adr_core::{AdrDocument, AdrId, AdrIndexEntry};

use super::{
    AdrMetaComponent, LayoutComponent, MarkdownContentComponent, PageContext, PageKind,
    PageNavComponent, ReferenceNavigationComponent, Render,
};

pub struct AdrPageComponent<'a> {
    pub entry: &'a AdrIndexEntry,
    pub previous: Option<&'a AdrDocument>,
    pub next: Option<&'a AdrDocument>,
    pub documents_by_id: &'a BTreeMap<AdrId, &'a AdrDocument>,
    pub known_ids: &'a BTreeSet<AdrId>,
    pub markdown_body: &'a str,
    pub home_label: &'a str,
    pub footer: &'a str,
}

impl Render for AdrPageComponent<'_> {
    fn render(&self) -> String {
        let document = &self.entry.document;
        let navigation = PageNavComponent {
            previous: self.previous,
            next: self.next,
        }
        .render();
        let meta = AdrMetaComponent { document }.render();
        let content = MarkdownContentComponent {
            body: self.markdown_body,
            page_context: PageContext::Adr,
            known_ids: self.known_ids,
        }
        .render();
        let related = ReferenceNavigationComponent {
            entry: self.entry,
            documents_by_id: self.documents_by_id,
        }
        .render();

        let page_body = format!("{}{}{}", navigation, content, related);
        let page_title = format!("{} — {}", document.id.as_ref(), document.title);
        let heading_html = format!(
            "<span class=\"heading-ref\">[{}]</span> — {}",
            document.id.as_ref(),
            document.title
        );

        LayoutComponent {
            page_kind: PageKind::Adr,
            title: &page_title,
            heading: &page_title,
            heading_html: Some(&heading_html),
            subtitle: None,
            subtitle_html: Some(&meta),
            body: &page_body,
            home_label: self.home_label,
            footer: self.footer,
        }
        .render()
    }
}
