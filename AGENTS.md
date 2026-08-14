# Repository Guidelines

## Scope

This repository builds a Codex Plugin that blocks common permanent deletion paths. Keep the project focused on Codex integration and policy detection; reuse operating-system Trash facilities or mature external tools instead of implementing a second Trash store.

## Safety

- Never remove repository files permanently during development. Move cleanup targets to the operating-system Trash.
- Policy failures, malformed Hook input, and missing packaged binaries must fail closed.
- Document bypass limitations honestly. Hooks are guardrails, not an operating-system security boundary.

## Rust checks

Run these before committing Rust behavior changes:

```text
cargo fmt --all -- --check
cargo test --locked
cargo clippy --all-targets --locked -- -D warnings
```

## Plugin checks

- Keep the Plugin identifier and Skill identifier as `recoverable-delete`; hyphenation is required by their external schemas.
- Keep the repository and owned script names underscore-separated where platform conventions permit.
- Validate `.codex-plugin/plugin.json`, `hooks/hooks.json`, and Skill frontmatter before packaging.
- A distributable Plugin archive must include the platform policy binary under `bin/`.
