#!/usr/bin/env pwsh
# Smoke-test the web bundle by serving it and fetching the key assets.
# Used by the Phase 07 acceptance gate and by CI in later phases.

[CmdletBinding()]
param(
    [int]$Port = 0
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$ScriptDir = if ($PSScriptRoot) { $PSScriptRoot } else { Split-Path -Parent $MyInvocation.MyCommand.Definition }
$BundleDir = (Resolve-Path (Join-Path $ScriptDir "..\..\dist-web")).Path

if (-not (Test-Path $BundleDir)) {
    Write-Error "Bundle directory not found: $BundleDir. Run build-web.ps1 first."
}

if ($Port -eq 0) {
    for ($p = 8765; $p -lt 8800; $p++) {
        $listener = New-Object System.Net.HttpListener
        try {
            $listener.Prefixes.Add("http://127.0.0.1:$p/")
            $listener.Start()
            $Port = $p
            break
        } catch {
            $listener.Close()
        }
    }
    if ($Port -eq 0) { Write-Error "Could not bind to a free port in 8765-8800" }
} else {
    $listener = New-Object System.Net.HttpListener
    $listener.Prefixes.Add("http://127.0.0.1:$Port/")
    $listener.Start()
}

Write-Host "Test server bound to http://127.0.0.1:$Port/"

$failures = New-Object System.Collections.Generic.List[string]

function Run-Check {
    param([string]$Name, [scriptblock]$Block)
    Write-Host "  [$Name]"
    try {
        & $Block
        Write-Host "    OK"
    } catch {
        $msg = $_.Exception.Message
        $failures.Add("$Name : $msg")
        Write-Host "    FAILED: $msg"
    }
}

$listenerTask = Start-ThreadJob -ScriptBlock {
    param($L, $B)
    while ($L.IsListening) {
        $ctx = $L.GetContext()
        $resp = $ctx.Response
        $rel = [Uri]::UnescapeDataString($ctx.Request.Url.AbsolutePath.TrimStart('/'))
        if ([string]::IsNullOrEmpty($rel)) { $rel = "index.html" }
        $file = Join-Path $B $rel
        if (Test-Path $file -PathType Leaf) {
            $bytes = [System.IO.File]::ReadAllBytes($file)
            $ext = [System.IO.Path]::GetExtension($file).TrimStart('.').ToLower()
            $mime = switch ($ext) {
                "html" { "text/html; charset=utf-8" }
                "js"   { "application/javascript; charset=utf-8" }
                "wasm" { "application/wasm" }
                "css"  { "text/css; charset=utf-8" }
                "json" { "application/json; charset=utf-8" }
                "svg"  { "image/svg+xml" }
                default { "application/octet-stream" }
            }
            $resp.ContentType = $mime
            $resp.ContentLength64 = $bytes.Length
            $resp.OutputStream.Write($bytes, 0, $bytes.Length)
        } else {
            $resp.StatusCode = 404
            $msg = [Text.Encoding]::UTF8.GetBytes("not found: $rel")
            $resp.ContentType = "text/plain"
            $resp.ContentLength64 = $msg.Length
            $resp.OutputStream.Write($msg, 0, $msg.Length)
        }
        $resp.Close()
    }
} -ArgumentList $listener, $BundleDir

Start-Sleep -Seconds 1

try {
    Write-Host "Smoke testing http://127.0.0.1:$Port/"
    Run-Check "index.html" {
        $r = Invoke-WebRequest -Uri "http://127.0.0.1:$Port/index.html" -UseBasicParsing -TimeoutSec 10
        if ($r.StatusCode -ne 200) { throw "status $($r.StatusCode)" }
        if ($r.Content -notmatch 'porkpie-web\.js') { throw "index.html does not reference porkpie-web.js" }
        if ($r.Content -notmatch 'porkpie-web_bg\.wasm') { throw "index.html does not reference porkpie-web_bg.wasm" }
        if ($r.Content -notmatch 'id="main"') { throw "index.html is missing the #main mount point" }
    }
    Run-Check "porkpie-web.js" {
        $r = Invoke-WebRequest -Uri "http://127.0.0.1:$Port/porkpie-web.js" -UseBasicParsing -TimeoutSec 10
        if ($r.StatusCode -ne 200) { throw "status $($r.StatusCode)" }
        if ($r.Content -notmatch '__wbindgen') { throw "porkpie-web.js does not look like a wasm-bindgen bundle" }
    }
    Run-Check "porkpie-web_bg.wasm" {
        $r = Invoke-WebRequest -Uri "http://127.0.0.1:$Port/porkpie-web_bg.wasm" -UseBasicParsing -TimeoutSec 10
        if ($r.StatusCode -ne 200) { throw "status $($r.StatusCode)" }
        if ($r.RawContentLength -lt 100000) { throw "wasm bundle suspiciously small: $($r.RawContentLength) bytes" }
    }
    Run-Check "snippets" {
        $interpDir = Get-ChildItem -LiteralPath $BundleDir -Recurse -Directory -Filter "dioxus-interpreter*" -ErrorAction SilentlyContinue | Select-Object -First 1
        if (-not $interpDir) { throw "no dioxus-interpreter snippet directory under $BundleDir" }
        $inlinePath = Join-Path $interpDir.FullName "inline0.js"
        $relPath = $inlinePath.Substring($BundleDir.Length).TrimStart('\', '/').Replace('\', '/')
        $r = Invoke-WebRequest -Uri "http://127.0.0.1:$Port/$relPath" -UseBasicParsing -TimeoutSec 10
        if ($r.StatusCode -ne 200) { throw "interpreter snippet missing: status $($r.StatusCode)" }
    }

    if ($failures.Count -gt 0) {
        Write-Host "FAILED checks:"
        foreach ($f in $failures) { Write-Host "  - $f" }
        exit 1
    }
    Write-Host "PASS"
}
finally {
    $listener.Stop()
    $listener.Close()
    Stop-Job -Job $listenerTask -ErrorAction SilentlyContinue
    Remove-Job -Job $listenerTask -ErrorAction SilentlyContinue
}
