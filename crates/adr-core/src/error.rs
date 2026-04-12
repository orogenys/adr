use std::{io, path::PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AdrError {
    #[error("could not find `adr.toml` starting from {start_dir}")]
    ConfigNotFound { start_dir: PathBuf },

    #[error("failed to read config file {path}: {source}")]
    ConfigRead {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("failed to parse config file {path}: {source}")]
    ConfigParse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },

    #[error("failed to read file {path}: {source}")]
    FileRead {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("failed to write file {path}: {source}")]
    FileWrite {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("failed to create directory {path}: {source}")]
    DirectoryCreate {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("failed to walk directory {path}: {source}")]
    WalkDir {
        path: PathBuf,
        #[source]
        source: walkdir::Error,
    },

    #[error("invalid ADR file name: {path}")]
    InvalidFileName { path: PathBuf },

    #[error("ADR not found: {query}")]
    AdrNotFound { query: String },

    #[error("ADR already exists: {path}")]
    AdrAlreadyExists { path: PathBuf },

    #[error("template not found: {name}")]
    TemplateNotFound { name: String },

    #[error("invalid ADR status: {value}")]
    InvalidStatus { value: String },

    #[error("missing frontmatter in {path}")]
    MissingFrontmatter { path: PathBuf },

    #[error("invalid frontmatter in {path}: {source}")]
    FrontmatterParse {
        path: PathBuf,
        #[source]
        source: serde_yaml::Error,
    },

    #[error("invalid date `{value}` in {path}")]
    InvalidDate { path: PathBuf, value: String },

    #[error("missing required field: {field}")]
    MissingField { field: &'static str },
}

pub type Result<T> = std::result::Result<T, AdrError>;
