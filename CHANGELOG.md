# Changelog

All notable changes to Recoverable Delete are documented here.

## [Unreleased]

### Changed

- Published the repository under the Tinkora organization with explicit ownership and dependency update policy.

## [0.1.0] - 2026-08-14

### Added

- Codex `PreToolUse` protection for common permanent deletion commands and `apply_patch` file deletion.
- A concise Agent Skill that directs cleanup through the operating-system Trash or Recycle Bin.
- Linux ARM64, macOS ARM64, and Windows x64 Marketplace archives with checksums and content manifests.
- Cross-platform policy tests, packaged-plugin tests, and Windows dispatcher verification.

### Security

- The Hook fails closed when its policy binary or input contract is unavailable.
- The project documents that a Hook is a guardrail and not an operating-system security boundary.
