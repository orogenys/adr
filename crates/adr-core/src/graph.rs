use std::{
    collections::{BTreeMap, BTreeSet},
    fmt, fs,
    path::{Component, Path, PathBuf},
    str::FromStr,
};

use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use serde::Serialize;

use crate::{
    adr::{AdrDocument, AdrId, AdrRef, collect_adr_file_paths, list_adrs, parse_file_name},
    config::LoadedConfig,
    error::{AdrError, Result},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdrReferenceKind {
    Mention,
    Link,
}

impl fmt::Display for AdrReferenceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mention => write!(f, "mention"),
            Self::Link => write!(f, "link"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdrOutgoingReference {
    pub target: AdrRef,
    pub kind: AdrReferenceKind,
    pub broken: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdrIncomingReference {
    pub source: AdrRef,
    pub kind: AdrReferenceKind,
}

#[derive(Debug, Clone)]
pub struct AdrIndexEntry {
    pub document: AdrDocument,
    pub outgoing: Vec<AdrOutgoingReference>,
    pub incoming: Vec<AdrIncomingReference>,
}

#[derive(Debug, Clone)]
pub struct AdrIndex {
    pub entries: Vec<AdrIndexEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AdrSearchField {
    Id,
    Reference,
    Title,
    Status,
    Category,
    Path,
    Body,
    OutgoingReference,
    IncomingReference,
}

impl AdrSearchField {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::Reference => "reference",
            Self::Title => "title",
            Self::Status => "status",
            Self::Category => "category",
            Self::Path => "path",
            Self::Body => "body",
            Self::OutgoingReference => "outgoing_reference",
            Self::IncomingReference => "incoming_reference",
        }
    }
}

impl fmt::Display for AdrSearchField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdrSearchMatch {
    pub field: AdrSearchField,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct AdrSearchResult {
    pub document: AdrDocument,
    pub matches: Vec<AdrSearchMatch>,
}

impl AdrIndex {
    pub fn entry_by_id(&self, id: AdrId) -> Option<&AdrIndexEntry> {
        self.entries.iter().find(|entry| entry.document.id == id)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AdrGraphExport {
    pub nodes: Vec<AdrGraphNodeExport>,
    pub edges: Vec<AdrGraphEdgeExport>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdrGraphNodeExport {
    pub id: String,
    pub ref_id: String,
    pub title: String,
    pub status: String,
    pub date: String,
    pub category: Option<String>,
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdrGraphEdgeExport {
    pub from: String,
    pub to: String,
    pub kind: AdrReferenceKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReferenceScan {
    pub(crate) id: AdrId,
    pub(crate) path: PathBuf,
    pub(crate) outgoing: Vec<DetectedReference>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct DetectedReference {
    pub(crate) target: AdrRef,
    pub(crate) kind: AdrReferenceKind,
}

pub fn build_adr_index(config: &LoadedConfig) -> Result<AdrIndex> {
    let documents = list_adrs(config)?;
    let reference_scans = scan_references(config)?;
    let scans_by_id = reference_scans
        .into_iter()
        .map(|scan| (scan.id, scan))
        .collect::<BTreeMap<_, _>>();
    let existing_ids = documents
        .iter()
        .map(|document| document.id)
        .collect::<BTreeSet<_>>();

    let mut incoming = BTreeMap::<AdrId, Vec<AdrIncomingReference>>::new();
    for document in &documents {
        if let Some(scan) = scans_by_id.get(&document.id) {
            for reference in &scan.outgoing {
                if existing_ids.contains(&reference.target.id()) {
                    incoming
                        .entry(reference.target.id())
                        .or_default()
                        .push(AdrIncomingReference {
                            source: document.id.as_ref(),
                            kind: reference.kind,
                        });
                }
            }
        }
    }

    let mut entries = Vec::with_capacity(documents.len());
    for document in documents {
        let mut outgoing = scans_by_id
            .get(&document.id)
            .map(|scan| {
                scan.outgoing
                    .iter()
                    .map(|reference| AdrOutgoingReference {
                        target: reference.target,
                        kind: reference.kind,
                        broken: !existing_ids.contains(&reference.target.id()),
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        outgoing.sort_by_key(|reference| (reference.target.id(), reference.kind));

        let mut backlinks = incoming.remove(&document.id).unwrap_or_default();
        backlinks.sort_by_key(|reference| (reference.source.id(), reference.kind));

        entries.push(AdrIndexEntry {
            document,
            outgoing,
            incoming: backlinks,
        });
    }

    entries.sort_by_key(|entry| entry.document.id);
    Ok(AdrIndex { entries })
}

pub fn export_adr_graph(index: &AdrIndex) -> AdrGraphExport {
    let mut nodes = Vec::with_capacity(index.entries.len());
    let mut edges = Vec::new();

    for entry in &index.entries {
        nodes.push(AdrGraphNodeExport {
            id: entry.document.id.to_string(),
            ref_id: entry.document.id.as_ref().to_string(),
            title: entry.document.title.clone(),
            status: entry.document.status.to_string(),
            date: entry.document.date.to_string(),
            category: entry.document.category.clone(),
            path: entry.document.path.display().to_string(),
        });

        for reference in &entry.outgoing {
            if reference.broken {
                continue;
            }

            edges.push(AdrGraphEdgeExport {
                from: entry.document.id.as_ref().to_string(),
                to: reference.target.to_string(),
                kind: reference.kind,
            });
        }
    }

    AdrGraphExport { nodes, edges }
}

pub fn search_adrs(config: &LoadedConfig, query: &str) -> Result<Vec<AdrSearchResult>> {
    let normalized_query = query.trim().to_ascii_lowercase();
    if normalized_query.is_empty() {
        return Ok(Vec::new());
    }

    let index = build_adr_index(config)?;
    let mut results = Vec::new();

    for entry in index.entries {
        let mut matches = Vec::new();
        let id_value = entry.document.id.to_string();
        let reference_value = entry.document.id.as_ref().to_string();
        let status_value = entry.document.status.to_string();
        let path_value = entry.document.path.display().to_string();

        maybe_push_match(
            &mut matches,
            AdrSearchField::Id,
            &id_value,
            &normalized_query,
        );
        maybe_push_match(
            &mut matches,
            AdrSearchField::Reference,
            &reference_value,
            &normalized_query,
        );
        maybe_push_match(
            &mut matches,
            AdrSearchField::Title,
            &entry.document.title,
            &normalized_query,
        );
        maybe_push_match(
            &mut matches,
            AdrSearchField::Status,
            &status_value,
            &normalized_query,
        );
        maybe_push_match(
            &mut matches,
            AdrSearchField::Path,
            &path_value,
            &normalized_query,
        );

        if let Some(category) = entry.document.category.as_deref() {
            maybe_push_match(
                &mut matches,
                AdrSearchField::Category,
                category,
                &normalized_query,
            );
        }

        let body =
            fs::read_to_string(&entry.document.path).map_err(|source| AdrError::FileRead {
                path: entry.document.path.clone(),
                source,
            })?;
        let normalized_body = body.replace("\r\n", "\n");
        let searchable_body = strip_frontmatter(&normalized_body).unwrap_or(&normalized_body);
        if let Some(snippet) = find_body_snippet(searchable_body, &normalized_query) {
            push_match(&mut matches, AdrSearchField::Body, snippet);
        }

        for reference in &entry.outgoing {
            let value = if reference.broken {
                format!("{} [{}] (broken)", reference.target, reference.kind)
            } else {
                format!("{} [{}]", reference.target, reference.kind)
            };
            maybe_push_match(
                &mut matches,
                AdrSearchField::OutgoingReference,
                &value,
                &normalized_query,
            );
        }

        for reference in &entry.incoming {
            let value = format!("{} [{}]", reference.source, reference.kind);
            maybe_push_match(
                &mut matches,
                AdrSearchField::IncomingReference,
                &value,
                &normalized_query,
            );
        }

        if !matches.is_empty() {
            results.push(AdrSearchResult {
                document: entry.document,
                matches,
            });
        }
    }

    results.sort_by_key(|result| result.document.id);
    Ok(results)
}

pub(crate) fn scan_references(config: &LoadedConfig) -> Result<Vec<ReferenceScan>> {
    let mut scans = Vec::new();

    for path in collect_adr_file_paths(config)? {
        let id = parse_file_name(&path)?.id;
        let raw = fs::read_to_string(&path).map_err(|source| AdrError::FileRead {
            path: path.clone(),
            source,
        })?;
        let normalized = raw.replace("\r\n", "\n");
        let body = strip_frontmatter(&normalized).unwrap_or(&normalized);
        let outgoing = extract_references(&path, body);

        scans.push(ReferenceScan { id, path, outgoing });
    }

    scans.sort_by_key(|scan| scan.id);
    Ok(scans)
}

fn strip_frontmatter(content: &str) -> Option<&str> {
    let rest = content.strip_prefix("---\n")?;
    let end = rest.find("\n---\n")?;
    Some(&rest[end + 5..])
}

fn maybe_push_match(
    matches: &mut Vec<AdrSearchMatch>,
    field: AdrSearchField,
    value: &str,
    normalized_query: &str,
) {
    if value.to_ascii_lowercase().contains(normalized_query) {
        push_match(matches, field, value.to_string());
    }
}

fn push_match(matches: &mut Vec<AdrSearchMatch>, field: AdrSearchField, value: String) {
    if matches
        .iter()
        .any(|existing| existing.field == field && existing.value == value)
    {
        return;
    }

    matches.push(AdrSearchMatch { field, value });
}

fn find_body_snippet(body: &str, normalized_query: &str) -> Option<String> {
    body.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .find(|line| line.to_ascii_lowercase().contains(normalized_query))
        .map(compact_body_snippet)
}

fn compact_body_snippet(line: &str) -> String {
    let compact = line.split_whitespace().collect::<Vec<_>>().join(" ");
    const MAX_LEN: usize = 100;
    if compact.chars().count() <= MAX_LEN {
        compact
    } else {
        let truncated = compact.chars().take(MAX_LEN - 1).collect::<String>();
        format!("{}…", truncated)
    }
}

fn extract_references(path: &Path, body: &str) -> Vec<DetectedReference> {
    let mut references = BTreeSet::new();
    let mut link_depth = 0usize;

    for event in Parser::new(body) {
        match event {
            Event::Start(Tag::Link { dest_url, .. }) => {
                link_depth += 1;
                if let Some(reference) = extract_reference_from_link(path, dest_url.as_ref()) {
                    references.insert(DetectedReference {
                        target: reference,
                        kind: AdrReferenceKind::Link,
                    });
                }
            }
            Event::End(TagEnd::Link) => {
                link_depth = link_depth.saturating_sub(1);
            }
            Event::Text(text) | Event::Code(text) | Event::Html(text) if link_depth == 0 => {
                for reference in extract_references_from_text(&text) {
                    references.insert(DetectedReference {
                        target: reference,
                        kind: AdrReferenceKind::Mention,
                    });
                }
            }
            _ => {}
        }
    }

    references.into_iter().collect()
}

fn extract_references_from_text(text: &str) -> Vec<AdrRef> {
    let bytes = text.as_bytes();
    let mut references = Vec::new();
    let mut index = 0usize;

    while index + 10 <= bytes.len() {
        if &bytes[index..index + 4] != b"ADR-" {
            index += 1;
            continue;
        }

        let digits = &bytes[index + 4..index + 10];
        if !digits.iter().all(u8::is_ascii_digit) {
            index += 1;
            continue;
        }

        let before_ok = index == 0 || !bytes[index - 1].is_ascii_alphanumeric();
        let after_ok = index + 10 == bytes.len() || !bytes[index + 10].is_ascii_digit();
        if before_ok && after_ok {
            if let Ok(value) = std::str::from_utf8(digits)
                && let Ok(id) = value.parse::<u32>()
            {
                references.push(AdrRef::new(AdrId::new(id)));
            }
            index += 10;
        } else {
            index += 1;
        }
    }

    references
}

fn extract_reference_from_link(current_path: &Path, destination: &str) -> Option<AdrRef> {
    let destination = destination
        .split(['#', '?'])
        .next()
        .unwrap_or_default()
        .trim();
    if destination.is_empty() || destination.contains("://") || destination.starts_with("mailto:") {
        return None;
    }

    if let Ok(reference) = AdrRef::from_str(destination) {
        return Some(reference);
    }

    let joined = if Path::new(destination).is_absolute() {
        PathBuf::from(destination)
    } else {
        normalize_path(
            current_path
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join(destination),
        )
    };

    parse_file_name(&joined).ok().map(|parts| parts.id.as_ref())
}

fn normalize_path(path: PathBuf) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }

    normalized
}

#[cfg(test)]
mod tests {
    use super::{
        AdrReferenceKind, compact_body_snippet, extract_references, extract_references_from_text,
        find_body_snippet,
    };
    use std::path::Path;

    #[test]
    fn text_references_are_detected() {
        let refs = extract_references_from_text("See ADR-000001 and ADR-000123.");
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0].to_string(), "ADR-000001");
        assert_eq!(refs[1].to_string(), "ADR-000123");
    }

    #[test]
    fn markdown_links_are_detected() {
        let refs = extract_references(
            Path::new("docs/adr/core/000010-sample.md"),
            "See [previous](../shared/000002-other.md).",
        );
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].target.to_string(), "ADR-000002");
        assert_eq!(refs[0].kind, AdrReferenceKind::Link);
    }

    #[test]
    fn body_snippets_are_compacted() {
        let snippet = find_body_snippet(
            "\n  This decision references ADR-000001 for context.\n",
            "adr-000001",
        )
        .unwrap();
        assert_eq!(snippet, "This decision references ADR-000001 for context.");
        assert_eq!(compact_body_snippet(&"x".repeat(120)).chars().count(), 100);
    }
}
