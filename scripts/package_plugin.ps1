param(
    [string]$OutputRoot
)

$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
if (-not $OutputRoot) {
    $OutputRoot = Join-Path $repoRoot "dist"
}

$pluginSource = Join-Path $repoRoot "plugins\recoverable-delete"
$packageRoot = Join-Path $OutputRoot "recoverable-delete"
if (Test-Path -LiteralPath $packageRoot) {
    throw "Package destination already exists: $packageRoot. Move it to the Recycle Bin before packaging again."
}

$policyBinary = $env:RECOVERABLE_DELETE_BIN
if (-not $policyBinary) {
    cargo build --manifest-path (Join-Path $repoRoot "Cargo.toml") --release --locked
    if ($LASTEXITCODE -ne 0) {
        throw "cargo build failed with exit code $LASTEXITCODE"
    }
    $policyBinary = Join-Path $repoRoot "target\release\recoverable-delete.exe"
}

if (-not (Test-Path -LiteralPath $policyBinary -PathType Leaf)) {
    throw "Policy binary not found: $policyBinary"
}

New-Item -ItemType Directory -Path (Join-Path $packageRoot "bin") -Force | Out-Null
Get-ChildItem -LiteralPath $pluginSource -Force |
    Copy-Item -Destination $packageRoot -Recurse
Copy-Item -LiteralPath $policyBinary -Destination (Join-Path $packageRoot "bin\recoverable-delete.exe")

Write-Output $packageRoot
