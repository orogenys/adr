use std::{collections::BTreeMap, fmt, fs, path::PathBuf};

use crate::{
    adr::{AdrDate, AdrId, AdrState},
    config::LoadedConfig,
    error::{AdrError, Result},
};

const BUILTIN_TEMPLATES: &[(&str, &str)] = &[
    ("full", include_str!("templates/full.md")),
    ("simple", include_str!("templates/simple.md")),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateSource {
    BuiltIn,
    Local(PathBuf),
}

impl fmt::Display for TemplateSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BuiltIn => write!(f, "built-in"),
            Self::Local(path) => write!(f, "local ({})", path.display()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TemplateDefinition {
    pub name: String,
    pub source: TemplateSource,
}

#[derive(Debug, Clone)]
pub struct TemplateContext {
    pub id: AdrId,
    pub title: String,
    pub status: AdrState,
    pub date: AdrDate,
    pub category: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RenderedTemplate {
    pub name: String,
    pub content: String,
}

pub fn list_templates(config: &LoadedConfig) -> Result<Vec<TemplateDefinition>> {
    let mut templates = BTreeMap::<String, TemplateDefinition>::new();

    for (name, _) in BUILTIN_TEMPLATES {
        templates.insert(
            (*name).to_string(),
            TemplateDefinition {
                name: (*name).to_string(),
                source: TemplateSource::BuiltIn,
            },
        );
    }

    let template_dir = config.template_dir_path();
    if template_dir.exists() {
        for entry in walkdir::WalkDir::new(&template_dir) {
            let entry = entry.map_err(|source| AdrError::WalkDir {
                path: template_dir.clone(),
                source,
            })?;

            if !entry.file_type().is_file() {
                continue;
            }

            if entry.path().extension().and_then(|ext| ext.to_str()) != Some("md") {
                continue;
            }

            let relative = entry
                .path()
                .strip_prefix(&template_dir)
                .expect("template path should be inside template dir");
            let name = relative
                .with_extension("")
                .to_string_lossy()
                .replace(std::path::MAIN_SEPARATOR, "/");

            templates.insert(
                name.clone(),
                TemplateDefinition {
                    name,
                    source: TemplateSource::Local(entry.path().to_path_buf()),
                },
            );
        }
    }

    Ok(templates.into_values().collect())
}

pub fn render_template(
    config: &LoadedConfig,
    template_name: &str,
    context: &TemplateContext,
) -> Result<RenderedTemplate> {
    let template = load_template_content(config, template_name)?;
    let rendered = template
        .replace("{{id}}", &context.id.to_string())
        .replace("{{title}}", &context.title)
        .replace("{{status}}", &context.status.to_string())
        .replace("{{date}}", &context.date.to_string())
        .replace("{{category}}", context.category.as_deref().unwrap_or(""));

    Ok(RenderedTemplate {
        name: template_name.to_string(),
        content: rendered,
    })
}

fn load_template_content(config: &LoadedConfig, template_name: &str) -> Result<String> {
    let template_dir = config.template_dir_path();
    let local_path = template_dir.join(format!("{template_name}.md"));

    if local_path.is_file() {
        return fs::read_to_string(&local_path).map_err(|source| AdrError::FileRead {
            path: local_path,
            source,
        });
    }

    BUILTIN_TEMPLATES
        .iter()
        .find(|(name, _)| *name == template_name)
        .map(|(_, content)| (*content).to_string())
        .ok_or_else(|| AdrError::TemplateNotFound {
            name: template_name.to_string(),
        })
}
