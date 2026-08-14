# Architecture and security boundary

## Request flow

```text
Codex local tool call
        |
        v
Plugin PreToolUse matcher (Bash / apply_patch aliases)
        |
        v
Platform dispatcher
        |
        v
Rust policy evaluator ---- allow: no stdout
        |
        +----------------- deny: PreToolUse permissionDecision
```

Codex discovers `plugins/recoverable-delete/hooks/hooks.json` through the official default Plugin Hook path. The dispatcher locates the platform binary under `bin/` and forwards Hook JSON from standard input. A missing binary returns a denial instead of silently disabling protection.

## Policy boundary

The evaluator parses common shell structure without executing the command. It recognizes direct deletion programs, selected nested shells and wrappers, `find` deletion modes, `git clean`, `rsync --delete`, nested `xargs` commands, destructive inline Python/Node/Ruby APIs, and `apply_patch Delete File`.

It deliberately does not claim complete semantic analysis of arbitrary scripts or binaries. Codex documentation notes that specialized tool paths can opt out of local function Hooks. Therefore:

- the Agent Skill remains required as a behavioral layer;
- operating-system permissions and backups remain the final recovery controls;
- enterprise administrators who need mandatory enforcement should deploy a managed Hook through `requirements.toml` and device management;
- release artifacts must include the policy binary for their exact operating system and architecture.

## Packaging plan

Platform-specific archives are complete local Marketplace roots. This matches the official Codex installation model while keeping the native policy binary inside the Plugin:

```text
recoverable_delete/
  .agents/plugins/marketplace.json
  plugins/recoverable-delete/
    .codex-plugin/plugin.json
    hooks/hooks.json
    hooks/dispatch_hook.sh
    hooks/dispatch_hook.ps1
    skills/recoverable-delete/
    bin/recoverable-delete or recoverable-delete.exe
```

The repository source intentionally does not commit compiled binaries. CI release jobs compile the appropriate binary, reject unexpected package files, verify Cargo and Plugin versions, and publish a deterministic archive with its SHA-256 checksum and content manifest.
