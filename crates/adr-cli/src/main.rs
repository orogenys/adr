use std::{
    path::{Path, PathBuf},
    process::ExitCode,
    str::FromStr,
};

use adr_core::{
    AdrDocument, AdrError, AdrSearchResult, AdrState, NewAdrRequest, build_adr_index,
    export_adr_graph, find_adr, lint_adrs, list_adrs, list_templates, load_config, search_adrs,
    update_adr_state,
};
use clap::{Args, Parser, Subcommand, ValueEnum};
use dialoguer::{Input, Select};
use thiserror::Error;

#[derive(Debug, Error)]
enum CliError {
    #[error(transparent)]
    Core(#[from] AdrError),

    #[error(transparent)]
    Dialoguer(#[from] dialoguer::Error),

    #[error("{0}")]
    Message(String),
}

#[derive(Debug, Parser)]
#[command(name = "adr")]
#[command(about = "Create and manage markdown ADRs")]
struct Cli {
    #[arg(long, global = true, value_name = "FILE")]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    New(NewArgs),
    List(ListArgs),
    State(StateArgs),
    Lint(LintArgs),
    Search(SearchArgs),
    Refs(RefsArgs),
    Graph(GraphArgs),
    Templates,
}

#[derive(Debug, Args)]
struct NewArgs {
    #[arg(long)]
    title: Option<String>,

    #[arg(long)]
    template: Option<String>,

    #[arg(long)]
    category: Option<String>,

    #[arg(long)]
    status: Option<String>,

    #[arg(long)]
    non_interactive: bool,
}

#[derive(Debug, Args)]
struct ListArgs {
    #[arg(long)]
    category: Option<String>,

    #[arg(long)]
    status: Option<String>,
}

#[derive(Debug, Args)]
struct StateArgs {
    target: String,
    state: String,
}

#[derive(Debug, Args)]
struct LintArgs {
    #[arg(long)]
    quiet: bool,
}

#[derive(Debug, Args)]
struct SearchArgs {
    query: String,
}

#[derive(Debug, Args)]
struct RefsArgs {
    target: String,
}

#[derive(Debug, Args)]
struct GraphArgs {
    #[arg(long, value_enum, default_value_t = GraphFormat::Text)]
    format: GraphFormat,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum GraphFormat {
    Text,
    Json,
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<ExitCode, CliError> {
    let cli = Cli::parse();
    let config = load_config(config_start_path(cli.config.as_deref()))?;

    match cli.command {
        Commands::New(args) => {
            handle_new(&config, args)?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::List(args) => {
            handle_list(&config, args)?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::State(args) => {
            handle_state(&config, args)?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::Lint(args) => handle_lint(&config, args),
        Commands::Search(args) => {
            handle_search(&config, args)?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::Refs(args) => {
            handle_refs(&config, args)?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::Graph(args) => {
            handle_graph(&config, args)?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::Templates => {
            handle_templates(&config)?;
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn handle_new(config: &adr_core::LoadedConfig, args: NewArgs) -> Result<(), CliError> {
    let templates = list_templates(config)?;
    let template_name = match args.template {
        Some(template) => template,
        None if args.non_interactive => config.config.default_template.clone(),
        None => {
            let template_names: Vec<String> = templates
                .iter()
                .map(|template| template.name.clone())
                .collect();
            let default_index = template_names
                .iter()
                .position(|name| name == &config.config.default_template)
                .unwrap_or(0);
            let selected = Select::new()
                .with_prompt("Choose a template")
                .items(&template_names)
                .default(default_index)
                .interact()?;
            template_names[selected].clone()
        }
    };

    let title = match args.title {
        Some(title) => title,
        None if args.non_interactive => {
            return Err(AdrError::MissingField { field: "title" }.into());
        }
        None => Input::<String>::new()
            .with_prompt("ADR title")
            .interact_text()?,
    };

    let category = match args.category {
        Some(category) => normalize_optional(category),
        None if args.non_interactive => None,
        None => normalize_optional(
            Input::<String>::new()
                .allow_empty(true)
                .with_prompt("Category folder (optional)")
                .interact_text()?,
        ),
    };

    let status = match args.status {
        Some(status) => AdrState::from_str(&status)?,
        None => AdrState::Proposed,
    };

    let result = adr_core::create_adr(
        config,
        NewAdrRequest {
            title,
            template: template_name,
            category,
            status,
        },
    )?;

    println!("Created {}", result.document.path.display());
    Ok(())
}

fn handle_list(config: &adr_core::LoadedConfig, args: ListArgs) -> Result<(), CliError> {
    let status_filter = match args.status {
        Some(status) => Some(AdrState::from_str(&status)?),
        None => None,
    };

    let mut documents = list_adrs(config)?;
    if let Some(category) = args.category.as_deref() {
        documents.retain(|document| document.category.as_deref() == Some(category));
    }
    if let Some(status) = status_filter {
        documents.retain(|document| document.status == status);
    }

    if documents.is_empty() {
        println!("No ADRs found.");
        return Ok(());
    }

    print_documents(&documents);
    Ok(())
}

fn handle_state(config: &adr_core::LoadedConfig, args: StateArgs) -> Result<(), CliError> {
    let state = AdrState::from_str(&args.state)?;
    let before = find_adr(config, &args.target)?;
    let updated = update_adr_state(config, &args.target, state)?;

    println!(
        "Updated {}: {} -> {}",
        updated.path.display(),
        before.status,
        updated.status
    );
    Ok(())
}

fn handle_lint(config: &adr_core::LoadedConfig, args: LintArgs) -> Result<ExitCode, CliError> {
    let results = lint_adrs(config)?;
    let failing: Vec<_> = results.iter().filter(|result| !result.is_clean()).collect();

    if failing.is_empty() {
        if !args.quiet {
            println!("All ADRs passed lint.");
        }
        return Ok(ExitCode::SUCCESS);
    }

    for result in failing {
        println!("{}", result.path.display());
        for message in &result.messages {
            println!(
                "  [{}] {}: {}",
                message.level, message.code, message.message
            );
        }
    }

    println!();
    println!("Lint failed.");
    Ok(ExitCode::from(1))
}

fn handle_search(config: &adr_core::LoadedConfig, args: SearchArgs) -> Result<(), CliError> {
    let query = args.query.trim();
    if query.is_empty() {
        return Err(CliError::Message(
            "Search query cannot be empty.".to_string(),
        ));
    }

    let results = search_adrs(config, query)?;
    if results.is_empty() {
        println!("No ADRs matched `{query}`.");
        return Ok(());
    }

    print_search_results(&results);
    Ok(())
}

fn handle_refs(config: &adr_core::LoadedConfig, args: RefsArgs) -> Result<(), CliError> {
    let index = build_adr_index(config)?;
    if index.entries.is_empty() {
        return Err(CliError::Message(
            "No ADRs found. Create one with `adr new`.".to_string(),
        ));
    }

    let document = find_adr(config, &args.target).map_err(|error| match error {
        AdrError::AdrNotFound { .. } => CliError::Message(format!(
            "ADR `{}` not found. Run `adr list` to see available ADRs.",
            args.target
        )),
        other => CliError::Core(other),
    })?;
    let entry = index
        .entry_by_id(document.id)
        .ok_or_else(|| CliError::Message(format!("ADR `{}` not found in index.", args.target)))?;

    println!("{}  {}", document.id.as_ref(), document.title);
    println!("Path: {}", document.path.display());
    println!("Status: {}", document.status);
    println!();

    println!("Outgoing references:");
    if entry.outgoing.is_empty() {
        println!("  (none)");
    } else {
        for reference in &entry.outgoing {
            if reference.broken {
                println!("  {} [{}] (broken)", reference.target, reference.kind);
            } else {
                println!("  {} [{}]", reference.target, reference.kind);
            }
        }
    }

    println!();
    println!("Incoming references:");
    if entry.incoming.is_empty() {
        println!("  (none)");
    } else {
        for reference in &entry.incoming {
            println!("  {} [{}]", reference.source, reference.kind);
        }
    }

    Ok(())
}

fn handle_graph(config: &adr_core::LoadedConfig, args: GraphArgs) -> Result<(), CliError> {
    let index = build_adr_index(config)?;

    match args.format {
        GraphFormat::Text => {
            if index.entries.is_empty() {
                println!("No ADRs found.");
                return Ok(());
            }

            for entry in &index.entries {
                println!("{}  {}", entry.document.id.as_ref(), entry.document.title);
                if entry.outgoing.is_empty() {
                    println!("  -> (none)");
                } else {
                    for reference in &entry.outgoing {
                        if reference.broken {
                            println!("  -> {} [{}] (broken)", reference.target, reference.kind);
                        } else {
                            println!("  -> {} [{}]", reference.target, reference.kind);
                        }
                    }
                }
            }
        }
        GraphFormat::Json => {
            let graph = export_adr_graph(&index);
            println!(
                "{}",
                serde_json::to_string_pretty(&graph).expect("graph json should serialize")
            );
        }
    }

    Ok(())
}

fn handle_templates(config: &adr_core::LoadedConfig) -> Result<(), CliError> {
    for template in list_templates(config)? {
        println!("{}\t{}", template.name, template.source);
    }
    Ok(())
}

fn print_documents(documents: &[AdrDocument]) {
    println!("ID      STATUS       CATEGORY        TITLE");
    for document in documents {
        println!(
            "{:06}  {:<11}  {:<14}  {}",
            document.id,
            document.status,
            document.category.as_deref().unwrap_or("-"),
            document.title
        );
    }
}

fn print_search_results(results: &[AdrSearchResult]) {
    for result in results {
        println!(
            "{}  {:<11}  {:<14}  {}",
            result.document.id.as_ref(),
            result.document.status,
            result.document.category.as_deref().unwrap_or("-"),
            result.document.title
        );
        for search_match in &result.matches {
            println!("  - {}: {}", search_match.field, search_match.value);
        }
        println!();
    }
}

fn normalize_optional(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn config_start_path(path: Option<&Path>) -> &Path {
    path.unwrap_or_else(|| Path::new("."))
}
