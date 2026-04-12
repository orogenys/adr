---
status: "accepted"
date: 2026-04-11
decision-makers: ["Orogenys"]
consulted: []
informed: []
---

# Using Rust

## Context and Problem Statement

This repository provides tooling for creating, linting, and publishing Architecture Decision Records. The implementation needs to be fast, portable, easy to distribute, and pleasant to maintain as a command-line tool.

Which implementation language should we use for the ADR toolchain?

## Decision Drivers

* Fast startup and low runtime overhead
* Single-binary distribution
* Strong type safety for parsing and rendering logic
* Good tooling for CLI applications and static site generation

## Considered Options

* Rust
* Python
* Node.js

## Decision Outcome

Chosen option: "Rust", because it provides strong correctness guarantees, excellent CLI ergonomics, and easy distribution as native binaries.

### Consequences

* Good, because the CLI and site generator are fast and self-contained
* Good, because typed models help keep ADR parsing and rendering reliable
* Good, because deployment does not require a separate language runtime
* Bad, because contributors need a Rust toolchain to work on the project

### Confirmation

We confirm this decision by successfully building and running the CLI and static site generator with standard Cargo commands.

## Pros and Cons of the Options

### Rust

Rust is a strong fit for command-line tooling and static site generation.

* Good, because it produces fast native binaries
* Good, because its type system helps reduce bugs in parsing and rendering code
* Good, because the ecosystem includes solid crates for CLI parsing and Markdown handling
* Bad, because compile times are higher than in scripting languages

### Python

Python is productive and widely known.

* Good, because many developers already know it
* Good, because iteration can be quick
* Bad, because packaging standalone tools is less straightforward
* Bad, because runtime performance is lower for heavier processing tasks

### Node.js

Node.js offers a strong ecosystem for content tooling.

* Good, because Markdown and docs tooling is abundant
* Good, because many developers are comfortable with JavaScript and TypeScript
* Bad, because distributing a single self-contained binary is less natural
* Bad, because the project would depend on a JavaScript runtime and package tooling

## More Information

### Mermaid example

```mermaid
flowchart LR
  A[Author writes ADR] --> B[adr-cli validates content]
  B --> C[adr-site builds static pages]
```

### Rust code example

```rust
fn main() {
    println!("adr tooling powered by Rust");
}
```
