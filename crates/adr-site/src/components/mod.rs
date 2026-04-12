mod adr_meta;
mod adr_page;
mod index_page;
mod index_table;
mod layout;
mod markdown_content;
mod page_nav;
mod reference_navigation;
mod support;
mod theme_toggle;

pub use adr_meta::AdrMetaComponent;
pub use adr_page::AdrPageComponent;
pub use index_page::IndexPageComponent;
pub use index_table::IndexTableComponent;
pub use layout::LayoutComponent;
pub use markdown_content::MarkdownContentComponent;
pub use page_nav::PageNavComponent;
pub use reference_navigation::ReferenceNavigationComponent;
pub use support::{
    PageContext, PageKind, Render, escape_html, index_href_to_adr, page_href_to_adr,
    render_template,
};
pub use theme_toggle::ThemeToggleComponent;
