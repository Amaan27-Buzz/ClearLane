param(
    [switch]$SkipTests
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot

if ($env:OS -ne "Windows_NT") {
    throw "The CEF engine proof is currently supported only on Windows."
}

# MSVC cannot reliably create the long intermediate paths produced by CEF when
# the checkout lives deep under directories such as D:\Programming\...\ClearLane.
# Keep Cargo's intermediate output at the drive root. This does not relocate
# the source tree, modify the CEF SDK, or disable Windows security.
if (-not $env:CARGO_TARGET_DIR) {
    $driveRoot = [System.IO.Path]::GetPathRoot($root)
    $env:CARGO_TARGET_DIR = Join-Path $driveRoot "clearlane-build"
}
New-Item -ItemType Directory -Force -Path $env:CARGO_TARGET_DIR | Out-Null
Write-Host "Cargo intermediate build directory: $env:CARGO_TARGET_DIR"

# The independent cef-rs bundler and application must share the same CEF SDK.
if (-not $env:CEF_PATH) {
    $env:CEF_PATH = Join-Path $env:LOCALAPPDATA "ClearLane/cef-sdk"
}
New-Item -ItemType Directory -Force -Path $env:CEF_PATH | Out-Null
Write-Host "CEF SDK: $env:CEF_PATH"

$bundleDir = Join-Path $root "target/engine-proof"
$exe = Join-Path $bundleDir "clearlane_engine.exe"
$dll = Join-Path $bundleDir "clearlane_engine.dll"

# Do not leave an old executable that looks like a successful current build.
if (Test-Path -LiteralPath $bundleDir) {
    Remove-Item -LiteralPath $bundleDir -Recurse -Force
}

Push-Location (Join-Path $root "engine")
try {
    cargo fmt --all -- --check
    if ($LASTEXITCODE -ne 0) {
        throw "cargo fmt failed (exit $LASTEXITCODE)"
    }

    if (-not $SkipTests) {
        cargo test --all-features
        if ($LASTEXITCODE -ne 0) {
            throw "cargo test failed (exit $LASTEXITCODE)"
        }
    }

    $cefBinary = Join-Path $env:CEF_PATH "156.0.14/cef_windows_x86_64/libcef.dll"
    if (-not (Test-Path -LiteralPath $cefBinary)) {
        throw "CEF SDK binary missing after Cargo build: $cefBinary"
    }

    # Upstream provides the Windows sandbox bootstrap; this is not an
    # unsandboxed cargo-run executable.
    if (-not (Get-Command bundle-cef-app -ErrorAction SilentlyContinue)) {
        cargo install cef --version 156.2.0 --features build-util --bin bundle-cef-app --locked
        if ($LASTEXITCODE -ne 0) {
            throw "Installing cef-rs bundle-cef-app failed (exit $LASTEXITCODE)"
        }
    }

    bundle-cef-app clearlane_engine --release --output $bundleDir
    if ($LASTEXITCODE -ne 0) {
        throw "CEF native compilation/bundling failed (exit $LASTEXITCODE). Read the FIRST compiler error above; no executable is ready."
    }

    if (-not (Test-Path -LiteralPath $exe) -or -not (Test-Path -LiteralPath $dll)) {
        throw "CEF bundler succeeded but required EXE/DLL are missing: $exe ; $dll"
    }

    Write-Host ""
    Write-Host "CEF sandbox bundle complete: $exe"
    Write-Host "Run: & '$exe' --url=https://example.com/"
}
finally {
    Pop-Location
}
