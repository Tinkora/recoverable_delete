# Recoverable Delete

Recoverable Delete is a Codex Plugin that blocks common permanent file-deletion paths and teaches Agents to move exact targets to the operating-system Trash or Recycle Bin.

[中文说明](README.zh_CN.md)

## Why another project?

Existing projects solve adjacent problems:

- [`skill-use-trash`](https://github.com/ar1g/skill-use-trash) is a small prompt-only Skill for macOS and Linux.
- [`ai-trash`](https://github.com/forethought-studio/ai-trash) is a mature MIT-licensed cross-platform Trash CLI and PATH wrapper.

Neither is a Codex Plugin with a `PreToolUse` Hook that observes both shell calls and `apply_patch`. This project supplies that Codex-native integration layer and can coexist with `ai-trash`; it does not reimplement a Trash store.

## Current behavior

- Denies common permanent deletion commands including `rm`, `rmdir`, `unlink`, `find -delete`, `git clean`, `xargs rm`, PowerShell `Remove-Item`, and Windows `del`.
- Detects nested shell execution, common command wrappers, absolute executable paths, and `apply_patch` `Delete File` operations.
- Allows normal commands and known Trash operations.
- Fails closed on malformed Hook input or a missing packaged policy binary.

This is a guardrail, not an operating-system security boundary. Arbitrary native programs, specialized tool paths, or a deliberately bypassed Hook can still delete data.

## Development

The shared Rust toolchain is selected by `rust-toolchain.toml`.

```sh
cargo fmt --all -- --check
cargo test --locked
cargo clippy --all-targets --locked -- -D warnings
```

Build the host policy binary and place it in a local Plugin copy under `bin/recoverable-delete` (`bin/recoverable-delete.exe` on Windows). The Hook intentionally blocks matched tools when that binary is absent.

To test the repository marketplace with Codex:

```sh
codex plugin marketplace add /absolute/path/to/recoverable_delete
codex plugin add recoverable-delete@tinkora
```

Start a new Codex task, open `/hooks`, and review and trust the Plugin Hook before testing it. Hook trust is intentionally not granted by installation.

## Documentation

- [Existing solution review](docs/existing_solutions.md)
- [Architecture and security boundary](docs/architecture.md)
- [Official Codex Hooks documentation](https://learn.chatgpt.com/docs/hooks)
- [Official Plugin packaging documentation](https://developers.openai.com/plugins/build/plugins)

## License

MIT
