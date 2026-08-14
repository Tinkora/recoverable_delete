$ErrorActionPreference = "Stop"

$pluginRoot = if ($env:PLUGIN_ROOT) {
    $env:PLUGIN_ROOT
} else {
    Split-Path -Parent $PSScriptRoot
}

$candidates = @(
    $env:RECOVERABLE_DELETE_BIN,
    (Join-Path $pluginRoot "bin\recoverable-delete.exe"),
    (Join-Path $pluginRoot "bin\recoverable_delete.exe")
)

foreach ($candidate in $candidates) {
    if ($candidate -and (Test-Path -LiteralPath $candidate -PathType Leaf)) {
        $hookInput = [Console]::In.ReadToEnd()
        $hookInput | & $candidate hook
        exit $LASTEXITCODE
    }
}

Write-Output '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"Recoverable Delete policy binary is unavailable. The matched tool call was blocked instead of failing open."}}'
