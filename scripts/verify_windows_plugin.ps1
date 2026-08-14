param(
    [Parameter(Mandatory = $true)]
    [string]$PolicyBinary,

    [Parameter(Mandatory = $true)]
    [string]$OutputRoot
)

$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot

if (Test-Path -LiteralPath $OutputRoot) {
    throw "Verification output already exists: $OutputRoot. Move it to the Recycle Bin before retrying."
}

$env:RECOVERABLE_DELETE_BIN = $PolicyBinary
$pluginRoot = & (Join-Path $PSScriptRoot "package_plugin.ps1") -OutputRoot $OutputRoot

$blockedInput = '{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"Remove-Item -Recurse build\\cache"}}'
$safeInput = '{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"git status --short"}}'
$policy = Join-Path $pluginRoot "bin\recoverable-delete.exe"

$directOutput = $blockedInput | & $policy hook | Out-String
if ($LASTEXITCODE -ne 0 -or $directOutput -notmatch '"permissionDecision":"deny"') {
    throw "Packaged Windows policy binary did not deny permanent deletion."
}

$safeOutput = $safeInput | & $policy hook | Out-String
if ($LASTEXITCODE -ne 0 -or $safeOutput.Trim()) {
    throw "Packaged Windows policy binary rejected a safe command."
}

$env:PLUGIN_ROOT = $pluginRoot
$dispatcher = Join-Path $pluginRoot "hooks\dispatch_hook.ps1"
$dispatchOutput = $blockedInput |
    & powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File $dispatcher |
    Out-String
if ($LASTEXITCODE -ne 0 -or $dispatchOutput -notmatch '"permissionDecision":"deny"') {
    throw "Windows Plugin dispatcher did not forward the denial."
}

Write-Output "Windows Plugin verification passed: $pluginRoot"
