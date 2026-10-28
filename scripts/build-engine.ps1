$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
Push-Location (Join-Path $root "engine")
try {
    cargo fmt --all -- --check
    cargo test --all-features
    # The upstream CEF bundle utility supplies the sandboxed Windows bootstrap.
    # Explicit version pin prevents accidental ABI mismatches.
    if (-not (Get-Command bundle-cef-app -ErrorAction SilentlyContinue)) {
        cargo install cef --version 156.2.0 --features build-util --bin bundle-cef-app --locked
    }
    bundle-cef-app clearlane-engine --release --output "..\target\engine-proof"
    Write-Host "Run: $root\target\engine-proof\clearlane-engine.exe"
}
finally {
    Pop-Location
}
