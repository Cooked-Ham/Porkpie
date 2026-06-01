#!/usr/bin/env pwsh
# Build the Porkpie web shell as a static directory of WASM + JS + HTML.
#
# The result is a folder named `dist-web/` next to this script that any
# static file server can serve. The script is intentionally explicit
# about its dependencies so the failure modes are obvious:
#
#   * rustup target add wasm32-unknown-unknown
#   * cargo install wasm-bindgen-cli --version 0.2.88
#
# Usage:
#   ./build-web.ps1            # debug build
#   ./build-web.ps1 -Release   # release build
#   ./build-web.ps1 -Serve     # build then start a local static server
#   ./build-web.ps1 -Clean     # wipe the build output before building

[CmdletBinding()]
param(
    [switch]$Release,
    [switch]$Serve,
    [switch]$Clean,
    [int]$Port = 8000
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$ScriptDir = if ($PSScriptRoot) { $PSScriptRoot } else { Split-Path -Parent $MyInvocation.MyCommand.Definition }
$RepoRoot = (Resolve-Path (Join-Path $ScriptDir "..\..")).Path
$AppDir = Join-Path $RepoRoot "apps\web"
$OutDir = Join-Path $RepoRoot "dist-web"
$Target = "wasm32-unknown-unknown"
$Profile = if ($Release) { "release" } else { "debug" }
$TargetDir = Join-Path $RepoRoot "target\$Target\$Profile"
$WasmArtifact = Join-Path $TargetDir "porkpie-web.wasm"

function Require-Command {
    param([string]$Name, [string]$InstallHint)
    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        Write-Error "Required tool '$Name' is not on PATH. $InstallHint"
    }
}

function Require-Target {
    $installed = & rustup target list --installed 2>$null
    if (-not ($installed -match [regex]::Escape($Target))) {
        Write-Error "Required Rust target '$Target' is not installed. Run: rustup target add $Target"
    }
}

Require-Command -Name "rustup" -InstallHint "Install rustup from https://rustup.rs."
Require-Command -Name "cargo" -InstallHint "Install rustup from https://rustup.rs."
Require-Command -Name "rustc" -InstallHint "Install rustup from https://rustup.rs."
Require-Command -Name "wasm-bindgen" -InstallHint "Run: cargo install wasm-bindgen-cli --version 0.2.122 --locked"
Require-Target

if ($Clean -and (Test-Path $OutDir)) {
    Write-Host "Cleaning $OutDir"
    Remove-Item -Recurse -Force $OutDir
}

if (-not (Test-Path $OutDir)) {
    New-Item -ItemType Directory -Path $OutDir | Out-Null
}

Write-Host "Compiling porkpie-web for $Target ($Profile)"
$env:PORKPIE_WEB_ROOT = "main"
& cargo build --manifest-path (Join-Path $AppDir "Cargo.toml") `
    --target $Target `
    $(if ($Release) { "--release" })
if ($LASTEXITCODE -ne 0) {
    Write-Error "cargo build failed"
}

if (-not (Test-Path $WasmArtifact)) {
    Write-Error "Expected WASM artifact not found: $WasmArtifact"
}

Write-Host "Running wasm-bindgen"
& wasm-bindgen --out-dir $OutDir --target web --no-typescript --omit-default-module-path $WasmArtifact
if ($LASTEXITCODE -ne 0) {
    Write-Error "wasm-bindgen failed"
}

$indexSrc = Join-Path $AppDir "index.html"
$indexDst = Join-Path $OutDir "index.html"
Copy-Item -Path $indexSrc -Destination $indexDst -Force
Write-Host "Wrote $indexDst"

Write-Host ""
Write-Host "Web bundle ready in $OutDir"
Get-ChildItem -LiteralPath $OutDir | ForEach-Object {
    $size = if ($_.PSIsContainer) { "<dir>" } else { $_.Length.ToString() }
    Write-Host ("  {0,-24} {1}" -f $_.Name, $size)
}

if ($Serve) {
    Write-Host ""
    Write-Host "Starting static server at http://127.0.0.1:$Port/ (Ctrl+C to stop)"
    Push-Location $OutDir
    try {
        $listener = New-Object System.Net.HttpListener
        $listener.Prefixes.Add("http://127.0.0.1:$Port/")
        $listener.Start()
        while ($listener.IsListening) {
            $context = $listener.GetContext()
            $request = $context.Request
            $response = $context.Response
            $relativePath = [Uri]::UnescapeDataString($request.Url.AbsolutePath.TrimStart('/'))
            if ([string]::IsNullOrEmpty($relativePath)) { $relativePath = "index.html" }
            $filePath = Join-Path $OutDir $relativePath
            if (Test-Path $filePath -PathType Leaf) {
                $bytes = [System.IO.File]::ReadAllBytes($filePath)
                $ext = [System.IO.Path]::GetExtension($filePath).TrimStart('.').ToLower()
                $mime = switch ($ext) {
                    "html" { "text/html; charset=utf-8" }
                    "js"   { "application/javascript; charset=utf-8" }
                    "wasm" { "application/wasm" }
                    "css"  { "text/css; charset=utf-8" }
                    "json" { "application/json; charset=utf-8" }
                    "svg"  { "image/svg+xml" }
                    default { "application/octet-stream" }
                }
                $response.ContentType = $mime
                $response.ContentLength64 = $bytes.Length
                $response.OutputStream.Write($bytes, 0, $bytes.Length)
            } else {
                $response.StatusCode = 404
                $msg = [System.Text.Encoding]::UTF8.GetBytes("Not found: $relativePath")
                $response.ContentType = "text/plain; charset=utf-8"
                $response.ContentLength64 = $msg.Length
                $response.OutputStream.Write($msg, 0, $msg.Length)
            }
            $response.Close()
        }
    } finally {
        Pop-Location
        if ($listener) { $listener.Stop() }
    }
}
