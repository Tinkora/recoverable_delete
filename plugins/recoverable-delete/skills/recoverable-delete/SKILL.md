---
name: recoverable-delete
description: Use when deleting, removing, cleaning up, or replacing files or directories where accidental loss must remain recoverable, including rm, unlink, rmdir, find -delete, git clean, Remove-Item, del, and apply_patch Delete File operations.
---

# Recoverable Delete

Move deletion targets to the operating system Trash or Recycle Bin. Never substitute permanent deletion merely because the files are generated, ignored, temporary, or easy to recreate.

## Workflow

1. Resolve and inspect every exact target. Do not use broad roots, unresolved variables, or unsafe globs.
2. Choose an available recoverable backend.
3. Move the targets, then verify the original paths no longer exist.
4. Report what moved and where it can be restored.

| Platform | Preferred operation |
| --- | --- |
| macOS | `/usr/bin/trash <exact paths>`; do not add `--`, which this system command treats as another path |
| Linux | `gio trash -- <exact paths>`; otherwise `trash-put -- <exact paths>` |
| Windows | Send exact paths to the Recycle Bin with a trusted tool such as `ai-trash` or the `Microsoft.VisualBasic.FileIO` PowerShell API |

If no recoverable backend is available, stop and explain the missing dependency. Do not fall back to `rm`, `Remove-Item`, `del`, direct filesystem APIs, or `apply_patch` file deletion.

## Hook response

When the plugin Hook blocks a tool call, retry the operation through the platform Trash backend. Do not disable or bypass the Hook, call an absolute delete executable, wrap deletion in another shell, or use a language runtime to achieve the same permanent deletion.

The Hook is a guardrail, not an operating-system security boundary. It recognizes common destructive commands and `apply_patch` deletion, but arbitrary binaries and specialized tool paths may bypass local function Hooks. Keep this Skill active as the behavioral layer.

## Quick example

```sh
# Inspect first, then move only the resolved target.
ls -ld -- build/cache
/usr/bin/trash build/cache
test ! -e build/cache
```

## Common mistakes

| Mistake | Correction |
| --- | --- |
| `rm` is acceptable for generated files | Generated files can contain the only copy of useful work; use Trash. |
| Alias `rm` to a Trash command | Aliases miss absolute paths, child processes, and non-interactive shells; keep the Hook enabled. |
| Use `apply_patch` to delete a file | `apply_patch` deletion is permanent; move the file separately. |
| Empty Trash after cleanup | Leave recovery to the user unless the user explicitly requests permanent emptying. |
