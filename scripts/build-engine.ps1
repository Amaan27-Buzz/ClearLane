$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
# Use the same CEF distribution for this crate and the independently installed
# CEF bundler. A missing CEF_PATH causes get_cef_dir() to fail in the bundler.
if (-not $env:CEF_PATH) {
    $env:CEF_PATH = Join-Path $env:LOCALAPPDATA "ClearLane/cef-sdk"
}
New-Item -ItemType Directory -Force -Path $env:CEF_PATH | Out-Null
Push-Location (Join-Path $root "engine")
try {
    cargo fmt --all -- --check
    cargo test --all-features
    $cefBinary = Join-Path $env:CEF_PATH "156.0.14/cef_windows_x86_64/libcef.dll"
    if (-not (Test-Path -LiteralPath $cefBinary)) {
        throw "CEF distribution missing: $cefBinary"
    }
    # The upstream CEF bundle utility supplies the sandboxed Windows bootstrap.
    # Explicit version pin prevents accidental ABI mismatches.
    if (-not (Get-Command bundle-cef-app -ErrorAction SilentlyContinue)) {
        cargo install cef --version 156.2.0 --features build-util --bin bundle-cef-app --locked
    }
    bundle-cef-app clearlane_engine --release --output "..\target\engine-proof"
    $exe = Join-Path $root "target/engine-proof/clearlane_engine.exe"
    $dll = Join-Path $root "target/engine-proof/clearlane_engine.dll"
    if (-not (Test-Path -LiteralPath $exe) -or -not (Test-Path -LiteralPath $dll)) {
        throw "Incomplete CEF bundle; required clearlane_engine.exe and clearlane_engine.dll"
    }
    Write-Host "Run: $root\target\engine-proof\clearlane_engine.exe"
}
finally {
    Pop-Location
}
