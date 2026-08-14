# v0.1.0 release and installation

## Release artifacts

`v0.1.0` publishes one complete local Marketplace archive per supported architecture:

| Host | Archive |
| --- | --- |
| macOS ARM64 | `recoverable-delete-macos-arm64.tar.gz` |
| Linux ARM64 | `recoverable-delete-linux-arm64.tar.gz` |
| Windows x64 | `recoverable-delete-windows-x86_64.zip` |

macOS x64, Linux x64, and Windows ARM64 are not included in this release candidate. Installing an archive for another operating system or architecture leaves the Hook unable to run its policy binary, so the dispatcher fails closed.

Each archive has two companion files:

- `.sha256` contains the archive SHA-256 in standard checksum format.
- `.contents.txt` records the version, Marketplace name, archive hash, and exact file list.

The release workflow rejects extra files, missing files, symlinks, and Cargo/Plugin version mismatches before creating the archive.

## Verify the download

On macOS:

```sh
shasum -a 256 -c recoverable-delete-macos-arm64.tar.gz.sha256
```

On Linux:

```sh
sha256sum -c recoverable-delete-linux-arm64.tar.gz.sha256
```

On Windows PowerShell:

```powershell
$archive = "recoverable-delete-windows-x86_64.zip"
$expected = ((Get-Content -LiteralPath "$archive.sha256") -split "\s+")[0]
$actual = (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant()
if ($actual -ne $expected) { throw "SHA-256 mismatch" }
```

Do not install an archive when its checksum or content manifest is missing or does not match.

## Install

The extracted `recoverable_delete` directory is a local Marketplace root, not only a Plugin folder. Keep it in a durable path because Codex tracks that Marketplace source.

macOS and Linux example:

```sh
install_root="$HOME/.local/share/recoverable_delete/v0.1.0"
mkdir -p "$install_root"
tar -xzf recoverable-delete-macos-arm64.tar.gz -C "$install_root"
codex plugin marketplace add "$install_root/recoverable_delete"
codex plugin add recoverable-delete@tinkora
```

Use the Linux archive name on Linux.

Windows PowerShell example:

```powershell
$installRoot = Join-Path $env:LOCALAPPDATA "recoverable_delete\v0.1.0"
New-Item -ItemType Directory -Path $installRoot -Force | Out-Null
Expand-Archive -LiteralPath "recoverable-delete-windows-x86_64.zip" -DestinationPath $installRoot
codex plugin marketplace add "$installRoot\recoverable_delete"
codex plugin add recoverable-delete@tinkora
```

Use a new, empty version directory. Do not overwrite an older extracted release.

Restart the ChatGPT/Codex desktop app, start a new task, open `/hooks`, inspect the `recoverable-delete` `PreToolUse` command, and explicitly trust it. Installation does not automatically grant Hook trust.

## Recovery behavior

The Hook blocks recognized permanent deletion paths; it does not keep a private backup or move files itself. The Agent must retry through the operating-system Trash or Recycle Bin.

- macOS: use `/usr/bin/trash <exact paths>`. The system command does not accept `--` as an option terminator.
- Linux: prefer `gio trash <exact paths>`, otherwise use `trash-put <exact paths>`.
- Windows: use a trusted Recycle Bin tool or the `Microsoft.VisualBasic.FileIO` API.

Restore an item through Finder Trash, the Linux desktop Trash, or Windows Recycle Bin. Do not empty that location automatically.

## Upgrade

1. Verify and extract the new release into a new version directory.
2. Run `codex plugin remove recoverable-delete@tinkora`.
3. Run `codex plugin marketplace remove tinkora`.
4. Add the new extracted Marketplace path and install the Plugin again.
5. Restart the app, review Hook trust, and verify the new version.
6. After verification, move the old extracted version directory to the operating-system Trash or Recycle Bin.

## Uninstall

```sh
codex plugin remove recoverable-delete@tinkora
codex plugin marketplace remove tinkora
```

Restart the app, confirm the Hook is absent, then move the extracted Marketplace directory to Trash or the Recycle Bin. Removing the Plugin does not empty the operating-system Trash and does not affect previously moved files.

## Security boundary

This Plugin is a guardrail, not an operating-system security boundary. It recognizes common shell commands, wrappers, inline runtime deletion APIs, and `apply_patch` file deletion. Arbitrary native programs, specialized tool paths, Hook opt-outs, operations outside Codex, or an intentionally bypassed Hook can still permanently delete data. Keep backups and normal operating-system permissions in place.

See the [official OpenAI Plugin packaging documentation](https://developers.openai.com/plugins/build/plugins) for the Marketplace-based local installation model.
