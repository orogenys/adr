use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    str::FromStr,
};

use adr_core::{AdrId, AdrRef};
use pulldown_cmark::{
    CodeBlockKind, CowStr, Event, HeadingLevel, LinkType, Parser, Tag, TagEnd, html::push_html,
};

use super::{PageContext, Render, escape_html, page_href_to_adr};

pub struct MarkdownContentComponent<'a> {
    pub body: &'a str,
    pub page_context: PageContext,
    pub known_ids: &'a BTreeSet<AdrId>,
}

impl Render for MarkdownContentComponent<'_> {
    fn render(&self) -> String {
        let parser = Parser::new(self.body);
        let events = transform_events(parser, self.page_context, self.known_ids);
        let mut html = String::new();
        push_html(&mut html, events.into_iter());
        format!("<article class=\"content\">{html}</article>")
    }
}

struct HeadingCapture {
    level: HeadingLevel,
    events: Vec<Event<'static>>,
    text: String,
}

fn transform_events<'a>(
    parser: Parser<'a>,
    page: PageContext,
    known_ids: &BTreeSet<AdrId>,
) -> Vec<Event<'static>> {
    let mut transformed = Vec::new();
    let mut inside_link = false;
    let mut inside_code_block = false;
    let mut mermaid_block: Option<String> = None;
    let mut heading_capture: Option<HeadingCapture> = None;
    let mut heading_ids = BTreeMap::<String, usize>::new();
    let mut skipping_first_h1 = matches!(page, PageContext::Adr);
    let mut skip_depth = 0usize;

    for event in parser {
        if skip_depth > 0 {
            match event {
                Event::Start(_) => {
                    skip_depth += 1;
                }
                Event::End(TagEnd::Heading(_)) => {
                    skip_depth -= 1;
                }
                Event::End(_) => {
                    skip_depth -= 1;
                }
                _ => {}
            }
            continue;
        }

        if let Some(content) = mermaid_block.as_mut() {
            match event {
                Event::Text(text) | Event::Code(text) | Event::Html(text) => {
                    content.push_str(text.as_ref());
                }
                Event::SoftBreak | Event::HardBreak => content.push('\n'),
                Event::End(TagEnd::CodeBlock) => {
                    transformed.push(Event::Html(CowStr::Boxed(
                        render_mermaid_block(content).into_boxed_str(),
                    )));
                    mermaid_block = None;
                    inside_code_block = false;
                }
                _ => {}
            }
            continue;
        }

        if let Some(heading) = heading_capture.as_mut() {
            match event {
                Event::End(TagEnd::Heading(_)) => {
                    transformed.push(Event::Html(CowStr::Boxed(
                        render_heading(heading, &mut heading_ids).into_boxed_str(),
                    )));
                    heading_capture = None;
                }
                Event::Text(text) => {
                    heading.text.push_str(text.as_ref());
                    heading.events.push(Event::Text(text.into_static()));
                }
                Event::Code(text) => {
                    heading.text.push_str(text.as_ref());
                    heading.events.push(Event::Code(text.into_static()));
                }
                Event::SoftBreak | Event::HardBreak => {
                    heading.text.push(' ');
                    heading.events.push(Event::Text(CowStr::Borrowed(" ")));
                }
                Event::Start(Tag::Link {
                    link_type,
                    dest_url,
                    title,
                    id,
                }) => {
                    let rewritten = rewrite_link_destination(dest_url.as_ref(), page, known_ids)
                        .unwrap_or_else(|| dest_url.into_static());
                    heading.events.push(Event::Start(Tag::Link {
                        link_type,
                        dest_url: rewritten,
                        title: title.into_static(),
                        id: id.into_static(),
                    }));
                }
                other => heading.events.push(other.into_static()),
            }
            continue;
        }

        match event {
            Event::Start(Tag::Heading { level, .. })
                if skipping_first_h1 && level == HeadingLevel::H1 =>
            {
                skipping_first_h1 = false;
                skip_depth = 1;
            }
            Event::Start(Tag::Heading { level, .. }) => {
                skipping_first_h1 = false;
                heading_capture = Some(HeadingCapture {
                    level,
                    events: Vec::new(),
                    text: String::new(),
                });
            }
            Event::Start(Tag::Link {
                link_type,
                dest_url,
                title,
                id,
            }) => {
                inside_link = true;
                let rewritten = rewrite_link_destination(dest_url.as_ref(), page, known_ids)
                    .unwrap_or_else(|| dest_url.into_static());
                transformed.push(Event::Start(Tag::Link {
                    link_type,
                    dest_url: rewritten,
                    title: title.into_static(),
                    id: id.into_static(),
                }));
            }
            Event::End(TagEnd::Link) => {
                inside_link = false;
                transformed.push(Event::End(TagEnd::Link));
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                inside_code_block = true;
                if is_mermaid_block(&kind) {
                    mermaid_block = Some(String::new());
                } else {
                    transformed.push(Event::Start(Tag::CodeBlock(kind.into_static())));
                }
            }
            Event::End(TagEnd::CodeBlock) => {
                inside_code_block = false;
                transformed.push(Event::End(TagEnd::CodeBlock));
            }
            Event::Text(text) if !inside_link && !inside_code_block => {
                skipping_first_h1 = false;
                push_text_with_auto_links(&mut transformed, text.as_ref(), page, known_ids);
            }
            other => {
                if !matches!(other, Event::SoftBreak | Event::HardBreak) {
                    skipping_first_h1 = false;
                }
                transformed.push(other.into_static())
            }
        }
    }

    transformed
}

fn is_mermaid_block(kind: &CodeBlockKind<'_>) -> bool {
    match kind {
        CodeBlockKind::Fenced(info) => info
            .split_ascii_whitespace()
            .next()
            .is_some_and(|language| language.eq_ignore_ascii_case("mermaid")),
        CodeBlockKind::Indented => false,
    }
}

fn render_mermaid_block(content: &str) -> String {
    format!(
        "<pre class=\"mermaid\">{}</pre>",
        escape_html(content.trim())
    )
}

fn render_heading(heading: &HeadingCapture, used_ids: &mut BTreeMap<String, usize>) -> String {
    let mut inner_html = String::new();
    push_html(&mut inner_html, heading.events.clone().into_iter());

    let id = unique_heading_id(slugify(&heading.text), used_ids);
    let level = heading_level_number(heading.level);

    format!(
        "<h{level} id=\"{id}\" class=\"section-heading\"><a class=\"heading-anchor\" href=\"#{id}\" aria-label=\"Copy link to this section\" title=\"Copy link to this section\">#</a>{inner_html}</h{level}>"
    )
}

fn unique_heading_id(base: String, used_ids: &mut BTreeMap<String, usize>) -> String {
    let next = used_ids.entry(base.clone()).or_insert(0);
    *next += 1;

    if *next == 1 {
        base
    } else {
        format!("{}-{}", base, *next)
    }
}

fn slugify(text: &str) -> String {
    let mut slug = String::new();
    let mut last_was_dash = false;

    for ch in text.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch);
            last_was_dash = false;
        } else if !last_was_dash && !slug.is_empty() {
            slug.push('-');
            last_was_dash = true;
        }
    }

    slug.trim_end_matches('-').to_string().if_empty("section")
}

fn heading_level_number(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

trait DefaultIfEmpty {
    fn if_empty(self, fallback: &str) -> String;
}

impl DefaultIfEmpty for String {
    fn if_empty(self, fallback: &str) -> String {
        if self.is_empty() {
            fallback.to_string()
        } else {
            self
        }
    }
}

fn rewrite_link_destination(
    destination: &str,
    page: PageContext,
    known_ids: &BTreeSet<AdrId>,
) -> Option<CowStr<'static>> {
    let (base, suffix) = split_destination(destination);

    let reference = AdrRef::from_str(base)
        .ok()
        .or_else(|| extract_adr_ref_from_path(base));
    let reference = reference.filter(|reference| known_ids.contains(&reference.id()))?;

    Some(CowStr::Boxed(
        format!("{}{}", page_href_to_adr(page, reference.id()), suffix).into_boxed_str(),
    ))
}

fn push_text_with_auto_links(
    events: &mut Vec<Event<'static>>,
    text: &str,
    page: PageContext,
    known_ids: &BTreeSet<AdrId>,
) {
    let bytes = text.as_bytes();
    let mut cursor = 0usize;
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
        if !(before_ok && after_ok) {
            index += 1;
            continue;
        }

        let matched = &text[index..index + 10];
        let Some(reference) = AdrRef::from_str(matched).ok() else {
            index += 1;
            continue;
        };

        if !known_ids.contains(&reference.id()) {
            index += 10;
            continue;
        }

        if cursor < index {
            push_text_event(events, &text[cursor..index]);
        }

        let href = page_href_to_adr(page, reference.id());
        events.push(Event::Start(Tag::Link {
            link_type: LinkType::Inline,
            dest_url: CowStr::Boxed(href.into_boxed_str()),
            title: CowStr::Borrowed(""),
            id: CowStr::Borrowed(""),
        }));
        events.push(Event::Text(CowStr::Boxed(
            matched.to_string().into_boxed_str(),
        )));
        events.push(Event::End(TagEnd::Link));

        cursor = index + 10;
        index += 10;
    }

    if cursor < text.len() {
        push_text_event(events, &text[cursor..]);
    }
}

fn push_text_event(events: &mut Vec<Event<'static>>, value: &str) {
    if value.is_empty() {
        return;
    }

    events.push(Event::Text(CowStr::Boxed(
        value.to_string().into_boxed_str(),
    )));
}

fn extract_adr_ref_from_path(path: &str) -> Option<AdrRef> {
    let file_name = PathBuf::from(path).file_name()?.to_owned();
    adr_core::adr::parse_file_name(Path::new(&file_name))
        .ok()
        .map(|parts| parts.id.as_ref())
}

fn split_destination(destination: &str) -> (&str, &str) {
    destination
        .find(['#', '?'])
        .map(|idx| (&destination[..idx], &destination[idx..]))
        .unwrap_or((destination, ""))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use adr_core::{AdrId, AdrRef};

    use super::{
        MarkdownContentComponent, extract_adr_ref_from_path, render_mermaid_block,
        rewrite_link_destination, slugify, split_destination,
    };
    use crate::components::{PageContext, Render, index_href_to_adr, page_href_to_adr};

    #[test]
    fn markdown_text_references_become_links() {
        let known = BTreeSet::from([AdrId::new(1)]);
        let html = MarkdownContentComponent {
            body: "See ADR-000001.",
            page_context: PageContext::Adr,
            known_ids: &known,
        }
        .render();
        assert!(html.contains("<a href=\"../000001/index.html\">ADR-000001</a>"));
    }

    #[test]
    fn markdown_links_to_adr_files_are_rewritten() {
        let known = BTreeSet::from([AdrId::new(1)]);
        let rewritten = rewrite_link_destination(
            "../docs/adr/000001-using-rust.md#context",
            PageContext::Adr,
            &known,
        )
        .unwrap();
        assert_eq!(rewritten.as_ref(), "../000001/index.html#context");
    }

    #[test]
    fn adr_path_filenames_resolve_to_refs() {
        assert_eq!(
            extract_adr_ref_from_path("../docs/adr/000001-using-rust.md").unwrap(),
            AdrRef::new(AdrId::new(1))
        );
    }

    #[test]
    fn hrefs_use_explicit_html_files() {
        assert_eq!(index_href_to_adr(AdrId::new(1)), "adr/000001/index.html");
        assert_eq!(
            page_href_to_adr(PageContext::Adr, AdrId::new(1)),
            "../000001/index.html"
        );
        assert_eq!(
            split_destination("ADR-000001#more"),
            ("ADR-000001", "#more")
        );
    }

    #[test]
    fn first_h1_is_removed_on_adr_pages() {
        let known = BTreeSet::new();
        let html = MarkdownContentComponent {
            body: "# Using Rust\n\n## Context and Problem Statement\n\nBody.",
            page_context: PageContext::Adr,
            known_ids: &known,
        }
        .render();

        assert!(!html.contains("<h1>Using Rust</h1>"));
        assert!(
            html.contains("<h2 id=\"context-and-problem-statement\" class=\"section-heading\">")
        );
    }

    #[test]
    fn mermaid_code_blocks_render_as_mermaid_previews() {
        let known = BTreeSet::from([AdrId::new(1)]);
        let html = MarkdownContentComponent {
            body: "```mermaid\nflowchart TD\n  A[ADR-000001] --> B[Done]\n```",
            page_context: PageContext::Adr,
            known_ids: &known,
        }
        .render();

        assert!(
            html.contains(
                "<pre class=\"mermaid\">flowchart TD\n  A[ADR-000001] --&gt; B[Done]</pre>"
            )
        );
        assert!(!html.contains("<a href="));
    }

    #[test]
    fn mermaid_blocks_are_html_escaped() {
        assert_eq!(
            render_mermaid_block("flowchart TD\nA[<unsafe>] --> B"),
            "<pre class=\"mermaid\">flowchart TD\nA[&lt;unsafe&gt;] --&gt; B</pre>"
        );
    }

    #[test]
    fn headings_get_stable_unique_ids_and_anchor_links() {
        let known = BTreeSet::new();
        let html = MarkdownContentComponent {
            body: "## Decision Outcome\n\nText\n\n## Decision Outcome\n\nMore text",
            page_context: PageContext::Adr,
            known_ids: &known,
        }
        .render();

        assert!(html.contains("id=\"decision-outcome\""));
        assert!(html.contains("href=\"#decision-outcome\""));
        assert!(html.contains("id=\"decision-outcome-2\""));
    }

    #[test]
    fn heading_slugs_are_normalized() {
        assert_eq!(slugify("Decision Outcome?"), "decision-outcome");
        assert_eq!(slugify("Rust & TOML"), "rust-toml");
        assert_eq!(slugify("!!!"), "section");
    }
}
