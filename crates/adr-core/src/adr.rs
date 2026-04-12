use std::{
    collections::BTreeSet,
    fmt, fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use chrono::{Local, NaiveDate};
use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};
use serde::{Deserialize, Serialize};

use crate::{
    config::LoadedConfig,
    error::{AdrError, Result},
    graph::scan_references,
    template::{TemplateContext, render_template},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AdrId(u32);

impl AdrId {
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    pub fn value(self) -> u32 {
        self.0
    }

    pub fn next(self) -> Self {
        Self(self.0 + 1)
    }

    pub fn as_ref(self) -> AdrRef {
        AdrRef::new(self)
    }
}

impl fmt::Display for AdrId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:06}", self.0)
    }
}

impl FromStr for AdrId {
    type Err = AdrError;

    fn from_str(value: &str) -> Result<Self> {
        let parsed = value
            .parse::<u32>()
            .map_err(|_| AdrError::InvalidFileName {
                path: PathBuf::from(value),
            })?;
        Ok(Self::new(parsed))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AdrRef {
    id: AdrId,
}

impl AdrRef {
    pub fn new(id: AdrId) -> Self {
        Self { id }
    }

    pub fn id(self) -> AdrId {
        self.id
    }
}

impl fmt::Display for AdrRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ADR-{}", self.id)
    }
}

impl FromStr for AdrRef {
    type Err = AdrError;

    fn from_str(value: &str) -> Result<Self> {
        let normalized = value.trim();
        let digits = normalized
            .strip_prefix("ADR-")
            .or_else(|| normalized.strip_prefix("adr-"))
            .ok_or_else(|| AdrError::AdrNotFound {
                query: value.to_string(),
            })?;

        if digits.len() != 6 || !digits.chars().all(|character| character.is_ascii_digit()) {
            return Err(AdrError::AdrNotFound {
                query: value.to_string(),
            });
        }

        Ok(Self::new(AdrId::new(digits.parse::<u32>().map_err(
            |_| AdrError::AdrNotFound {
                query: value.to_string(),
            },
        )?)))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AdrSlug(String);

impl AdrSlug {
    pub fn new(value: String) -> Result<Self> {
        if value.is_empty()
            || !value.chars().all(|character| {
                character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
            })
        {
            return Err(AdrError::InvalidFileName {
                path: PathBuf::from(value),
            });
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AdrSlug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AdrDate(NaiveDate);

impl AdrDate {
    pub fn today() -> Self {
        Self(Local::now().date_naive())
    }
}

impl fmt::Display for AdrDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.format("%Y-%m-%d"))
    }
}

impl FromStr for AdrDate {
    type Err = AdrError;

    fn from_str(value: &str) -> Result<Self> {
        let parsed =
            NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|_| AdrError::InvalidDate {
                path: PathBuf::new(),
                value: value.to_string(),
            })?;
        Ok(Self(parsed))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AdrState {
    Proposed,
    Rejected,
    Accepted,
    Deprecated,
    Superseded,
}

impl AdrState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Rejected => "rejected",
            Self::Accepted => "accepted",
            Self::Deprecated => "deprecated",
            Self::Superseded => "superseded",
        }
    }

    pub fn parse_frontmatter(value: &str) -> Result<Self> {
        let normalized = value.trim().to_ascii_lowercase();
        if normalized.starts_with("superseded by ") {
            return Ok(Self::Superseded);
        }

        Self::from_str(&normalized)
    }
}

impl fmt::Display for AdrState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for AdrState {
    type Err = AdrError;

    fn from_str(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "proposed" | "draft" => Ok(Self::Proposed),
            "rejected" => Ok(Self::Rejected),
            "accepted" => Ok(Self::Accepted),
            "deprecated" => Ok(Self::Deprecated),
            "superseded" => Ok(Self::Superseded),
            other => Err(AdrError::InvalidStatus {
                value: other.to_string(),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileNameParts {
    pub id: AdrId,
    pub slug: AdrSlug,
}

#[derive(Debug, Clone)]
pub struct AdrDocument {
    pub id: AdrId,
    pub slug: AdrSlug,
    pub title: String,
    pub status: AdrState,
    pub date: AdrDate,
    pub category: Option<String>,
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct NewAdrRequest {
    pub title: String,
    pub template: String,
    pub category: Option<String>,
    pub status: AdrState,
}

#[derive(Debug, Clone)]
pub struct NewAdrResult {
    pub document: AdrDocument,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdrLintLevel {
    Error,
    Warning,
}

impl fmt::Display for AdrLintLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Error => write!(f, "error"),
            Self::Warning => write!(f, "warning"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdrLintCode {
    FrontmatterMissing,
    FrontmatterParse,
    FrontmatterStatusMissing,
    FrontmatterStatusInvalid,
    FrontmatterDateMissing,
    FrontmatterDateInvalid,
    HeadingTitleMissing,
    HeadingRequired,
    ReferenceBroken,
}

impl AdrLintCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::FrontmatterMissing => "frontmatter.missing",
            Self::FrontmatterParse => "frontmatter.parse",
            Self::FrontmatterStatusMissing => "frontmatter.status.missing",
            Self::FrontmatterStatusInvalid => "frontmatter.status.invalid",
            Self::FrontmatterDateMissing => "frontmatter.date.missing",
            Self::FrontmatterDateInvalid => "frontmatter.date.invalid",
            Self::HeadingTitleMissing => "heading.title.missing",
            Self::HeadingRequired => "heading.required",
            Self::ReferenceBroken => "reference.broken",
        }
    }
}

impl fmt::Display for AdrLintCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdrLintMessage {
    pub level: AdrLintLevel,
    pub code: AdrLintCode,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct AdrLint {
    pub path: PathBuf,
    pub messages: Vec<AdrLintMessage>,
}

impl AdrLint {
    pub fn is_clean(&self) -> bool {
        self.messages.is_empty()
    }
}

#[derive(Debug, Clone)]
struct AdrFrontmatter {
    status: AdrState,
    date: AdrDate,
    decision_makers: Vec<String>,
    consulted: Vec<String>,
    informed: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RawAdrFrontmatter {
    status: Option<String>,
    date: Option<String>,
    #[serde(rename = "decision-makers", default)]
    decision_makers: Vec<String>,
    #[serde(default)]
    consulted: Vec<String>,
    #[serde(default)]
    informed: Vec<String>,
}

impl TryFrom<RawAdrFrontmatter> for AdrFrontmatter {
    type Error = AdrError;

    fn try_from(value: RawAdrFrontmatter) -> Result<Self> {
        let status = value
            .status
            .ok_or(AdrError::MissingField { field: "status" })
            .and_then(|status| AdrState::parse_frontmatter(&status))?;
        let date = value
            .date
            .ok_or(AdrError::MissingField { field: "date" })
            .and_then(|date| AdrDate::from_str(&date))?;

        Ok(Self {
            status,
            date,
            decision_makers: value.decision_makers,
            consulted: value.consulted,
            informed: value.informed,
        })
    }
}

impl From<&AdrFrontmatter> for RawAdrFrontmatter {
    fn from(value: &AdrFrontmatter) -> Self {
        Self {
            status: Some(value.status.to_string()),
            date: Some(value.date.to_string()),
            decision_makers: value.decision_makers.clone(),
            consulted: value.consulted.clone(),
            informed: value.informed.clone(),
        }
    }
}

#[derive(Debug, Clone)]
struct MarkdownDocument {
    title: Option<String>,
    sections: BTreeSet<RequiredSection>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum RequiredSection {
    ContextAndProblemStatement,
    DecisionOutcome,
}

impl RequiredSection {
    fn heading(self) -> &'static str {
        match self {
            Self::ContextAndProblemStatement => "Context and Problem Statement",
            Self::DecisionOutcome => "Decision Outcome",
        }
    }

    fn from_heading(level: u8, heading: &str) -> Option<Self> {
        if level != 2 {
            return None;
        }

        match heading {
            "Context and Problem Statement" => Some(Self::ContextAndProblemStatement),
            "Decision Outcome" => Some(Self::DecisionOutcome),
            _ => None,
        }
    }
}

pub fn create_adr(config: &LoadedConfig, request: NewAdrRequest) -> Result<NewAdrResult> {
    let id = next_adr_id(config)?;
    let slug = slugify(&request.title);
    let file_name = format!("{id}-{slug}.md");

    let mut target_dir = config.adr_root_path();
    if let Some(category) = request
        .category
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        target_dir.push(category.trim());
    }

    fs::create_dir_all(&target_dir).map_err(|source| AdrError::DirectoryCreate {
        path: target_dir.clone(),
        source,
    })?;

    let path = target_dir.join(file_name);
    if path.exists() {
        return Err(AdrError::AdrAlreadyExists { path });
    }

    let date = AdrDate::today();
    let rendered = render_template(
        config,
        &request.template,
        &TemplateContext {
            id,
            title: request.title.clone(),
            status: request.status,
            date,
            category: request.category.clone(),
        },
    )?;

    fs::write(&path, rendered.content).map_err(|source| AdrError::FileWrite {
        path: path.clone(),
        source,
    })?;

    Ok(NewAdrResult {
        document: read_adr_document(config, &path)?,
    })
}

pub fn list_adrs(config: &LoadedConfig) -> Result<Vec<AdrDocument>> {
    let mut documents = Vec::new();
    for path in collect_adr_file_paths(config)? {
        documents.push(read_adr_document(config, &path)?);
    }
    documents.sort_by_key(|document| document.id);
    Ok(documents)
}

pub fn next_adr_id(config: &LoadedConfig) -> Result<AdrId> {
    let max_id = collect_adr_file_paths(config)?
        .into_iter()
        .filter_map(|path| parse_file_name(&path).ok().map(|parts| parts.id))
        .max()
        .unwrap_or_else(|| AdrId::new(0));

    Ok(max_id.next())
}

pub fn find_adr(config: &LoadedConfig, query: &str) -> Result<AdrDocument> {
    let query_path = PathBuf::from(query);
    if query_path.is_file() {
        return read_adr_document(config, &query_path);
    }

    let documents = list_adrs(config)?;

    if let Ok(id) = query.parse::<u32>()
        && let Some(document) = documents.iter().find(|document| document.id.value() == id)
    {
        return Ok(document.clone());
    }

    if let Ok(reference) = AdrRef::from_str(query)
        && let Some(document) = documents
            .iter()
            .find(|document| document.id == reference.id())
    {
        return Ok(document.clone());
    }

    if let Some(document) = documents.into_iter().find(|document| {
        document
            .path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name == query)
            || document.path.to_string_lossy().ends_with(query)
    }) {
        return Ok(document);
    }

    Err(AdrError::AdrNotFound {
        query: query.to_string(),
    })
}

pub fn update_adr_state(
    config: &LoadedConfig,
    query: &str,
    state: AdrState,
) -> Result<AdrDocument> {
    let document = find_adr(config, query)?;
    let raw = read_file(&document.path)?;
    let normalized = normalize_newlines(&raw);
    let (frontmatter, body) = split_frontmatter(&normalized, &document.path)?;
    let raw_frontmatter = parse_frontmatter(&document.path, frontmatter)?;
    let mut typed_frontmatter = AdrFrontmatter::try_from(raw_frontmatter)?;

    typed_frontmatter.status = state;
    typed_frontmatter.date = AdrDate::today();

    let serialized =
        serde_yaml::to_string(&RawAdrFrontmatter::from(&typed_frontmatter)).map_err(|source| {
            AdrError::FrontmatterParse {
                path: document.path.clone(),
                source,
            }
        })?;
    let updated = format!("---\n{}---\n{}", serialized, body);

    fs::write(&document.path, updated).map_err(|source| AdrError::FileWrite {
        path: document.path.clone(),
        source,
    })?;

    read_adr_document(config, &document.path)
}

pub fn lint_adrs(config: &LoadedConfig) -> Result<Vec<AdrLint>> {
    let mut results = Vec::new();
    let reference_scans = scan_references(config)?;
    let existing_ids = reference_scans
        .iter()
        .map(|scan| scan.id)
        .collect::<BTreeSet<_>>();
    let reference_scan_by_path = reference_scans
        .iter()
        .map(|scan| (scan.path.clone(), scan))
        .collect::<std::collections::BTreeMap<_, _>>();

    for path in collect_adr_file_paths(config)? {
        let mut messages = Vec::new();
        let raw = read_file(&path)?;
        let normalized = normalize_newlines(&raw);

        let (frontmatter, body) = match split_frontmatter(&normalized, &path) {
            Ok(parts) => parts,
            Err(_) => {
                messages.push(AdrLintMessage {
                    level: AdrLintLevel::Error,
                    code: AdrLintCode::FrontmatterMissing,
                    message: "missing YAML frontmatter block".to_string(),
                });
                results.push(AdrLint { path, messages });
                continue;
            }
        };

        match parse_frontmatter(&path, frontmatter) {
            Ok(raw_frontmatter) => {
                if let Some(status) = raw_frontmatter.status.as_deref() {
                    if AdrState::parse_frontmatter(status).is_err() {
                        messages.push(AdrLintMessage {
                            level: AdrLintLevel::Error,
                            code: AdrLintCode::FrontmatterStatusInvalid,
                            message: format!("invalid status `{status}`"),
                        });
                    }
                } else {
                    messages.push(AdrLintMessage {
                        level: AdrLintLevel::Error,
                        code: AdrLintCode::FrontmatterStatusMissing,
                        message: "missing `status` in frontmatter".to_string(),
                    });
                }

                if let Some(date) = raw_frontmatter.date.as_deref() {
                    if AdrDate::from_str(date).is_err() {
                        messages.push(AdrLintMessage {
                            level: AdrLintLevel::Error,
                            code: AdrLintCode::FrontmatterDateInvalid,
                            message: format!("invalid ISO date `{date}`"),
                        });
                    }
                } else {
                    messages.push(AdrLintMessage {
                        level: AdrLintLevel::Error,
                        code: AdrLintCode::FrontmatterDateMissing,
                        message: "missing `date` in frontmatter".to_string(),
                    });
                }
            }
            Err(_) => messages.push(AdrLintMessage {
                level: AdrLintLevel::Error,
                code: AdrLintCode::FrontmatterParse,
                message: "frontmatter is not valid YAML".to_string(),
            }),
        }

        let markdown = parse_markdown(body);
        if markdown.title.is_none() {
            messages.push(AdrLintMessage {
                level: AdrLintLevel::Error,
                code: AdrLintCode::HeadingTitleMissing,
                message: "missing top-level `# Title` heading".to_string(),
            });
        }

        for required in [
            RequiredSection::ContextAndProblemStatement,
            RequiredSection::DecisionOutcome,
        ] {
            if !markdown.sections.contains(&required) {
                messages.push(AdrLintMessage {
                    level: AdrLintLevel::Error,
                    code: AdrLintCode::HeadingRequired,
                    message: format!("missing required heading `## {}``", required.heading())
                        .replace("``", "`"),
                });
            }
        }

        if let Some(scan) = reference_scan_by_path.get(&path) {
            for reference in &scan.outgoing {
                if !existing_ids.contains(&reference.target.id()) {
                    messages.push(AdrLintMessage {
                        level: AdrLintLevel::Error,
                        code: AdrLintCode::ReferenceBroken,
                        message: format!("broken ADR reference `{}`", reference.target),
                    });
                }
            }
        }

        results.push(AdrLint { path, messages });
    }

    Ok(results)
}

pub fn parse_file_name(path: &Path) -> Result<FileNameParts> {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| AdrError::InvalidFileName {
            path: path.to_path_buf(),
        })?;

    let stem = file_name
        .strip_suffix(".md")
        .ok_or_else(|| AdrError::InvalidFileName {
            path: path.to_path_buf(),
        })?;

    let (id_part, slug_part) = stem
        .split_once('-')
        .ok_or_else(|| AdrError::InvalidFileName {
            path: path.to_path_buf(),
        })?;

    if id_part.len() != 6 || !id_part.chars().all(|character| character.is_ascii_digit()) {
        return Err(AdrError::InvalidFileName {
            path: path.to_path_buf(),
        });
    }

    let id = AdrId::new(
        id_part
            .parse::<u32>()
            .map_err(|_| AdrError::InvalidFileName {
                path: path.to_path_buf(),
            })?,
    );
    let slug = AdrSlug::new(slug_part.to_string()).map_err(|_| AdrError::InvalidFileName {
        path: path.to_path_buf(),
    })?;

    Ok(FileNameParts { id, slug })
}

pub(crate) fn collect_adr_file_paths(config: &LoadedConfig) -> Result<Vec<PathBuf>> {
    let root = config.adr_root_path();
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut paths = Vec::new();
    for entry in walkdir::WalkDir::new(&root) {
        let entry = entry.map_err(|source| AdrError::WalkDir {
            path: root.clone(),
            source,
        })?;

        if !entry.file_type().is_file() {
            continue;
        }

        if entry.path().extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }

        if parse_file_name(entry.path()).is_ok() {
            paths.push(entry.path().to_path_buf());
        }
    }

    paths.sort();
    Ok(paths)
}

fn read_adr_document(config: &LoadedConfig, path: &Path) -> Result<AdrDocument> {
    let parts = parse_file_name(path)?;
    let raw = read_file(path)?;
    let normalized = normalize_newlines(&raw);
    let (frontmatter, body) = split_frontmatter(&normalized, path)?;
    let raw_frontmatter = parse_frontmatter(path, frontmatter)?;
    let typed_frontmatter = AdrFrontmatter::try_from(raw_frontmatter)?;
    let markdown = parse_markdown(body);

    let title = markdown
        .title
        .unwrap_or_else(|| parts.slug.as_str().replace('-', " "));
    let adr_root = config.adr_root_path();
    let category = path
        .parent()
        .and_then(|parent| parent.strip_prefix(&adr_root).ok())
        .map(|relative| relative.to_string_lossy().to_string())
        .and_then(|relative| {
            if relative.is_empty() {
                None
            } else {
                Some(relative)
            }
        });

    Ok(AdrDocument {
        id: parts.id,
        slug: parts.slug,
        title,
        status: typed_frontmatter.status,
        date: typed_frontmatter.date,
        category,
        path: path.to_path_buf(),
    })
}

fn read_file(path: &Path) -> Result<String> {
    fs::read_to_string(path).map_err(|source| AdrError::FileRead {
        path: path.to_path_buf(),
        source,
    })
}

fn split_frontmatter<'a>(content: &'a str, path: &Path) -> Result<(&'a str, &'a str)> {
    let rest = content
        .strip_prefix("---\n")
        .ok_or_else(|| AdrError::MissingFrontmatter {
            path: path.to_path_buf(),
        })?;
    let end = rest
        .find("\n---\n")
        .ok_or_else(|| AdrError::MissingFrontmatter {
            path: path.to_path_buf(),
        })?;

    let frontmatter = &rest[..end];
    let body = &rest[end + 5..];
    Ok((frontmatter, body))
}

fn parse_frontmatter(path: &Path, frontmatter: &str) -> Result<RawAdrFrontmatter> {
    serde_yaml::from_str(frontmatter).map_err(|source| AdrError::FrontmatterParse {
        path: path.to_path_buf(),
        source,
    })
}

fn parse_markdown(body: &str) -> MarkdownDocument {
    let mut title = None;
    let mut sections = BTreeSet::new();
    let mut current_heading: Option<(u8, String)> = None;

    for event in Parser::new(body) {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                current_heading = Some((heading_level_to_u8(level), String::new()));
            }
            Event::Text(text) | Event::Code(text) | Event::Html(text)
                if current_heading.is_some() =>
            {
                current_heading
                    .as_mut()
                    .expect("heading state should exist")
                    .1
                    .push_str(&text);
            }
            Event::SoftBreak | Event::HardBreak if current_heading.is_some() => {
                current_heading
                    .as_mut()
                    .expect("heading state should exist")
                    .1
                    .push(' ');
            }
            Event::End(TagEnd::Heading(_)) => {
                if let Some((level, text)) = current_heading.take() {
                    let heading = normalize_heading_text(&text);
                    if level == 1 && title.is_none() && !heading.is_empty() {
                        title = Some(heading.clone());
                    }
                    if let Some(section) = RequiredSection::from_heading(level, &heading) {
                        sections.insert(section);
                    }
                }
            }
            _ => {}
        }
    }

    MarkdownDocument { title, sections }
}

fn heading_level_to_u8(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

fn normalize_heading_text(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn normalize_newlines(content: &str) -> String {
    content.replace("\r\n", "\n")
}

fn slugify(title: &str) -> AdrSlug {
    let mut slug = String::with_capacity(title.len());
    let mut last_was_dash = false;

    for character in title.chars() {
        let mapped = match character {
            'a'..='z' | '0'..='9' => Some(character),
            'A'..='Z' => Some(character.to_ascii_lowercase()),
            _ => None,
        };

        if let Some(character) = mapped {
            slug.push(character);
            last_was_dash = false;
        } else if !last_was_dash && !slug.is_empty() {
            slug.push('-');
            last_was_dash = true;
        }
    }

    while slug.ends_with('-') {
        slug.pop();
    }

    AdrSlug::new(if slug.is_empty() {
        "untitled".to_string()
    } else {
        slug
    })
    .expect("slugify should always produce a valid slug")
}

#[cfg(test)]
mod tests {
    use super::{AdrId, AdrRef, AdrState, parse_file_name, slugify};
    use std::path::Path;

    #[test]
    fn slugify_normalizes_titles() {
        assert_eq!(
            slugify("Use Rust for ADR CLI").as_str(),
            "use-rust-for-adr-cli"
        );
        assert_eq!(slugify("  !!!  ").as_str(), "untitled");
    }

    #[test]
    fn file_names_must_start_with_six_digits() {
        let parts = parse_file_name(Path::new("000123-my-decision.md")).unwrap();
        assert_eq!(parts.id, AdrId::new(123));
        assert_eq!(parts.slug.as_str(), "my-decision");
        assert!(parse_file_name(Path::new("ADR-0001.md")).is_err());
    }

    #[test]
    fn superseded_by_is_accepted_in_frontmatter() {
        let state = AdrState::parse_frontmatter("superseded by ADR-0123").unwrap();
        assert_eq!(state, AdrState::Superseded);
    }

    #[test]
    fn canonical_adr_refs_are_parsed() {
        let reference = "ADR-000321".parse::<AdrRef>().unwrap();
        assert_eq!(reference.id(), AdrId::new(321));
        assert_eq!(reference.to_string(), "ADR-000321");
    }
}
