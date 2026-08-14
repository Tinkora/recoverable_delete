# Existing solution review

Research date: 2026-08-14.

## `ar1g/skill-use-trash`

Repository: <https://github.com/ar1g/skill-use-trash>

This project contains a concise `SKILL.md` that tells an Agent to use `trash` on macOS or `trash-put` on Linux. It is useful as a behavioral reminder, but it does not provide a Codex Plugin manifest, a `PreToolUse` Hook, Windows handling, or `apply_patch` interception. No repository license was declared during review, so its text is not copied here.

## `forethought-studio/ai-trash`

Repository: <https://github.com/forethought-studio/ai-trash>

`ai-trash` is a mature MIT-licensed cross-platform implementation. It supports macOS, Linux, and Windows; wraps deletion-related executables through `PATH`; and provides list, restore, and empty operations. It is the preferred optional backend when a consistent Trash CLI is needed.

Its enforcement model is different from a Codex lifecycle Hook:

- PATH wrappers can be bypassed with absolute executable paths or other filesystem APIs.
- The default selective mode depends on detecting an AI process; a full-access Codex environment may not expose the expected sandbox marker.
- A PATH wrapper cannot observe `apply_patch` file deletion.

## Decision

Build a small Codex-native integration layer rather than another Trash implementation:

- bundle a `PreToolUse` Hook and a concise Agent Skill;
- block common permanent shell commands and `apply_patch Delete File`;
- fail closed when policy evaluation is unavailable;
- delegate recoverable storage to the operating system or `ai-trash`;
- document that Hooks remain guardrails, not an unbypassable sandbox.

## Skill behavior evaluation

The same cleanup pressure scenario was run before and after loading the Skill:

- Baseline: the Agent proposed `rm -rf` for cache directories and `find -delete` for temporary files.
- With `recoverable-delete`: the Agent resolved and inspected exact paths, moved them with `/usr/bin/trash`, verified the original paths were absent, and explicitly kept the Trash recoverable.

This confirms that the Skill changes the default cleanup behavior while the Hook provides deterministic coverage for common violations.
