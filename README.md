# adr

`adr` is a small Rust workspace for writing, maintaining, and publishing **Architecture Decision Records (ADRs)**.

ADRs are lightweight documents that capture important technical decisions:
- the **context** behind a decision
- the **options** considered
- the **decision** that was made
- the **consequences** of that decision

They help teams explain *why* a system looks the way it does, not just *what* was built.

## What this project provides

This workspace contains three crates:

- **`crates/adr-core`**: shared ADR domain logic, parsing, config loading, references, search, graphing, and linting
- **`crates/adr-cli`**: the `adr` command-line tool for creating and managing ADRs
- **`crates/adr-site`**: a static site generator for publishing ADRs as HTML

## Repository layout

Typical structure:

```text
adr.toml
.adr/templates/
docs/adr/
crates/
```

- `adr.toml` defines where ADRs and templates live
- `docs/adr/` contains ADR markdown files
- `.adr/templates/` can contain custom templates

## Configuration

Example `adr.toml`:

```toml
adr_root = "docs/adr"
template_dir = ".adr/templates"
default_template = "full"
```

## Common commands

Create and manage ADRs:

```bash
cargo run -p adr-cli -- new
cargo run -p adr-cli -- list
cargo run -p adr-cli -- state 1 accepted
cargo run -p adr-cli -- search rust
cargo run -p adr-cli -- refs ADR-000001
cargo run -p adr-cli -- graph
cargo run -p adr-cli -- lint
```

## Adding a new ADR

A simple workflow:

```bash
cargo run -p adr-cli -- new
cargo run -p adr-cli -- lint
cargo run -p adr-site -- build
```

Then open the generated site in `dist/` and review the new ADR page.

Build the static site:

```bash
cargo run -p adr-site -- build
```

## Static site features

`adr-site build` generates a site in `dist/` with:

- an ADR index page
- one page per ADR
- explicit `index.html` links for local file browsing
- Markdown rendering
- automatic links for plain references like `ADR-000001`
- Mermaid diagrams from fenced `mermaid` blocks
- syntax-highlighted fenced code blocks
- stable section anchors for headings
- clickable heading links that copy deep links to sections
- light/dark theme toggle
- previous/next navigation and reference navigation

## Development

Useful checks:

```bash
cargo fmt --all
cargo test
cargo clippy --all-targets -- -D warnings
```

## GitHub workflows

This repository includes:

- **CI**: runs `cargo test` and `cargo clippy`
- **GitHub Pages deploy**: runs ADR linting, builds the site, and publishes `dist/`

## Local git hook

A pre-commit hook is available at `.githooks/pre-commit` and runs:

```bash
cargo fmt --all
```

Enable it locally with:

```bash
git config core.hooksPath .githooks
```
