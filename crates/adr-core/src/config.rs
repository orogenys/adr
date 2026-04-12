use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;

use crate::error::{AdrError, Result};

#[derive(Debug, Clone, Deserialize)]
pub struct AdrConfig {
    #[serde(default = "default_adr_root")]
    pub adr_root: String,
    #[serde(default = "default_template_dir")]
    pub template_dir: String,
    #[serde(default = "default_template")]
    pub default_template: String,
}

impl Default for AdrConfig {
    fn default() -> Self {
        Self {
            adr_root: default_adr_root(),
            template_dir: default_template_dir(),
            default_template: default_template(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LoadedConfig {
    pub path: PathBuf,
    pub root_dir: PathBuf,
    pub config: AdrConfig,
}

impl LoadedConfig {
    pub fn adr_root_path(&self) -> PathBuf {
        self.root_dir.join(&self.config.adr_root)
    }

    pub fn template_dir_path(&self) -> PathBuf {
        self.root_dir.join(&self.config.template_dir)
    }
}

pub fn load_config(start: impl AsRef<Path>) -> Result<LoadedConfig> {
    let start = start.as_ref();
    let config_path = if start.is_file() {
        start.to_path_buf()
    } else {
        find_config_path(start)?
    };

    let content = fs::read_to_string(&config_path).map_err(|source| AdrError::ConfigRead {
        path: config_path.clone(),
        source,
    })?;

    let config = toml::from_str::<AdrConfig>(&content).map_err(|source| AdrError::ConfigParse {
        path: config_path.clone(),
        source,
    })?;

    let root_dir = config_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));

    Ok(LoadedConfig {
        path: config_path,
        root_dir,
        config,
    })
}

fn find_config_path(start: &Path) -> Result<PathBuf> {
    let start_dir = if start.is_dir() {
        start.to_path_buf()
    } else {
        start
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."))
    };

    for candidate_dir in start_dir.ancestors() {
        let candidate = candidate_dir.join("adr.toml");
        if candidate.is_file() {
            return Ok(candidate);
        }
    }

    Err(AdrError::ConfigNotFound {
        start_dir: start_dir.clone(),
    })
}

fn default_adr_root() -> String {
    "docs/adr".to_string()
}

fn default_template_dir() -> String {
    ".adr/templates".to_string()
}

fn default_template() -> String {
    "full".to_string()
}
