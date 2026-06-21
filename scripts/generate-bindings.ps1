# Generate Kotlin bindings from UniFFI scaffold
# Requires: uniffi-bindgen CLI

param(
    [string]$Profile = "release"
)

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent $PSScriptRoot

$profileFlag = if ($Profile -eq "release") { "--release" } else { "" }

# Build FFI crate first (needed for scaffolding)
Write-Host "Building phantom-ffi..." -ForegroundColor Yellow
& cargo build --package phantom-ffi $profileFlag `
    --manifest-path (Join-Path $ProjectRoot "ffi\Cargo.toml")

if ($LASTEXITCODE -ne 0) {
    Write-Error "Build failed"
    exit $LASTEXITCODE
}

# Generate Kotlin bindings
$ffiLib = Join-Path $ProjectRoot "ffi\src\lib.rs"
$outDir = Join-Path $ProjectRoot "app\src\main\kotlin"

Write-Host "Generating Kotlin bindings..." -ForegroundColor Yellow
& uniffi-bindgen generate $ffiLib --language kotlin --out-dir $outDir

if ($LASTEXITCODE -ne 0) {
    Write-Error "Binding generation failed"
    exit $LASTEXITCODE
}

Write-Host "Bindings written to $outDir" -ForegroundColor Green
Write-Host "=== Done ===" -ForegroundColor Cyan
