pub mod adr;
pub mod config;
pub mod error;
pub mod graph;
pub mod template;

pub use adr::{
    AdrDate, AdrDocument, AdrId, AdrLint, AdrLintCode, AdrLintLevel, AdrLintMessage, AdrRef,
    AdrSlug, AdrState, FileNameParts, NewAdrRequest, NewAdrResult, create_adr, find_adr, lint_adrs,
    list_adrs, next_adr_id, update_adr_state,
};
pub use config::{AdrConfig, LoadedConfig, load_config};
pub use error::{AdrError, Result};
pub use graph::{
    AdrGraphEdgeExport, AdrGraphExport, AdrGraphNodeExport, AdrIncomingReference, AdrIndex,
    AdrIndexEntry, AdrOutgoingReference, AdrReferenceKind, AdrSearchField, AdrSearchMatch,
    AdrSearchResult, build_adr_index, export_adr_graph, search_adrs,
};
pub use template::{
    RenderedTemplate, TemplateContext, TemplateDefinition, TemplateSource, list_templates,
    render_template,
};
