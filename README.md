# Recoverable Delete

Recoverable Delete is a Codex Plugin that blocks common permanent file-deletion paths and teaches Agents to move exact targets to the operating-system Trash or Recycle Bin.

[中文说明](README.zh_CN.md)

## Why another project?

Existing projects solve adjacent problems:

- [`skill-use-trash`](https://github.com/ar1g/skill-use-trash) is a small prompt-only Skill for macOS and Linux.
- [`ai-trash`](https://github.com/forethought-studio/ai-trash) is a mature MIT-licensed cross-platform Trash CLI and PATH wrapper.

Neither is a Codex Plugin with a `PreToolUse` Hook that observes both shell calls and `apply_patch`. This project supplies that Codex-native integration layer and can coexist with `ai-trash`; it does not reimplement a Trash store.

## Current behavior

- Denies common permanent deletion commands including `rm`, `rmdir`, `unlink`, `find -delete`, `git clean`, `rsync --delete`, PowerShell `Remove-Item`, and Windows `del`.
- Detects nested shell execution, `xargs` commands, common command wrappers, absolute executable paths, destructive inline Python/Node/Ruby APIs, and `apply_patch` `Delete File` operations.
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

Build a host-specific Plugin directory:

```sh
sh scripts/package_plugin.sh
```

On Windows:

```powershell
./scripts/package_plugin.ps1
```

Run the packaged Windows policy binary and PowerShell Hook dispatcher end to end:

```powershell
cargo build --release --locked
$outputRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("recoverable_delete_verify_" + [guid]::NewGuid())
./scripts/verify_windows_plugin.ps1 `
  -PolicyBinary .\target\release\recoverable-delete.exe `
  -OutputRoot $outputRoot
```

Both commands refuse to overwrite an existing package. Move the old package to Trash or the Recycle Bin before rebuilding. The Hook intentionally blocks matched tools when its packaged policy binary is absent.

## Install a release

Each platform archive extracts to a complete local Marketplace root named `recoverable_delete`. Verify its SHA-256 file, extract it into a durable directory, add that directory with `codex plugin marketplace add`, and install `recoverable-delete@tinkora`.

See the [v0.1.0 release and installation guide](docs/release_v0_1_0.md) for platform commands, Hook trust, recovery, upgrades, uninstallation, and architecture limits.

To test the repository marketplace with Codex:

```sh
codex plugin marketplace add /absolute/path/to/recoverable_delete
codex plugin add recoverable-delete@tinkora
```

Start a new Codex task, open `/hooks`, and review and trust the Plugin Hook before testing it. Hook trust is intentionally not granted by installation.

Pushing a `v*` tag runs the release workflow, builds native Linux ARM64, macOS ARM64, and Windows x64 Marketplace archives, and attaches each archive, checksum, and content manifest to a GitHub release.

## Documentation

- [Existing solution review](docs/existing_solutions.md)
- [Architecture and security boundary](docs/architecture.md)
- [v0.1.0 release and installation](docs/release_v0_1_0.md)
- [Official Codex Hooks documentation](https://learn.chatgpt.com/docs/hooks)
- [Official Plugin packaging documentation](https://developers.openai.com/plugins/build/plugins)

## License

MIT
