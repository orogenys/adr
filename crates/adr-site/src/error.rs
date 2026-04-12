use std::{io, path::PathBuf};

use adr_core::AdrError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SiteError {
    #[error(transparent)]
    Core(#[from] AdrError),

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

    #[error("failed to remove directory {path}: {source}")]
    DirectoryRemove {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

pub type Result<T> = std::result::Result<T, SiteError>;
