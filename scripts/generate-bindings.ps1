# Generate Kotlin bindings from compiled UniFFI library
# Uses uniffi_bindgen as a library (no CLI install needed)

param(
    [string]$Profile = "release"
)

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent $PSScriptRoot

$profileFlag = if ($Profile -eq "release") { "--release" } else { "" }

# Build FFI crate first (produces cdylib with embedded scaffolding)
Write-Host "Building phantom-ffi..." -ForegroundColor Yellow
& cargo build --package phantom-ffi $profileFlag `
    --manifest-path (Join-Path $ProjectRoot "ffi\Cargo.toml")

if ($LASTEXITCODE -ne 0) {
    Write-Error "Build failed"
    exit $LASTEXITCODE
}

# Locate the compiled library
$targetDir = Join-Path $ProjectRoot "target" $Profile
$libFile = $null
foreach ($pattern in @("*.dll", "*.so", "*.dylib")) {
    $candidate = Get-ChildItem (Join-Path $targetDir "phantom_ffi.$pattern") -ErrorAction SilentlyContinue
    if ($candidate) { $libFile = $candidate.FullName; break }
}
if (-not $libFile) {
    # Try lib prefix for Unix
    foreach ($pattern in @("*.so", "*.dylib")) {
        $candidate = Get-ChildItem (Join-Path $targetDir "libphantom_ffi.$pattern") -ErrorAction SilentlyContinue
        if ($candidate) { $libFile = $candidate.FullName; break }
    }
}
if (-not $libFile) {
    Write-Error "Cannot find compiled library in $targetDir"
    exit 1
}

# Generate Kotlin bindings using uniffi_bindgen library API
$tmpDir = Join-Path $env:TEMP "uniffi-gen"
if (Test-Path $tmpDir) { Remove-Item -Recurse -Force $tmpDir }
New-Item -ItemType Directory -Path "$tmpDir\src" | Out-Null
$outDir = Join-Path $ProjectRoot "app\src\main\kotlin"
$cfgFile = Join-Path $ProjectRoot "ffi\uniffi.toml"

@"
[package]
name = "uniffi-gen"
version = "0.1.0"
edition = "2021"
[dependencies]
uniffi_bindgen = "=0.28.1"
camino = "1"
"@ | Out-File -Encoding UTF8 "$tmpDir\Cargo.toml"

@"
use camino::Utf8Path;
fn main() {
    let mut a = std::env::args().skip(1);
    let lib = a.next().expect("usage: gen <lib> <out> <cfg>");
    let out = a.next().expect("usage: gen <lib> <out> <cfg>");
    let cfg = a.next().expect("usage: gen <lib> <out> <cfg>");
    uniffi_bindgen::library_mode::generate_bindings(
        Utf8Path::new(&lib), None,
        &uniffi_bindgen::bindings::kotlin::KotlinBindingGenerator,
        &uniffi_bindgen::EmptyCrateConfigSupplier,
        Some(Utf8Path::new(&cfg)),
        Utf8Path::new(&out), true,
    ).expect("generate_bindings failed");
}
"@ | Out-File -Encoding UTF8 "$tmpDir\src\main.rs"

Write-Host "Generating Kotlin bindings..." -ForegroundColor Yellow
& cargo run --release --manifest-path "$tmpDir\Cargo.toml" -- `
    $libFile $outDir $cfgFile

if ($LASTEXITCODE -ne 0) {
    Write-Error "Binding generation failed"
    exit $LASTEXITCODE
}

Write-Host "Bindings written to $outDir" -ForegroundColor Green
Write-Host "=== Done ===" -ForegroundColor Cyan
