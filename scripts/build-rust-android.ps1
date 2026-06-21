# Build Rust core + FFI for Android targets
# Requirements: Rust + Android NDK + cargo-ndk

param(
    [string]$Profile = "release",
    [string[]]$Targets = @("aarch64-linux-android", "armv7-linux-androideabi", "x86_64-linux-android")
)

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent $PSScriptRoot

Write-Host "=== Building phantom-ffi for Android ===" -ForegroundColor Cyan

# Build for each target
foreach ($target in $Targets) {
    Write-Host "Building for $target..." -ForegroundColor Yellow
    $profileFlag = if ($Profile -eq "release") { "--release" } else { "" }
    
    & cargo ndk `
        --target $target `
        --platform 28 `
        build --package phantom-ffi $profileFlag `
        --manifest-path (Join-Path $ProjectRoot "ffi\Cargo.toml")
    
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Build failed for $target"
        exit $LASTEXITCODE
    }
}

# Copy .so files to app jniLibs
$targetDir = if ($Profile -eq "release") { "release" } else { "debug" }

$abiMap = @{
    "aarch64-linux-android"    = "arm64-v8a"
    "armv7-linux-androideabi"  = "armeabi-v7a"
    "x86_64-linux-android"    = "x86_64"
}

foreach ($target in $Targets) {
    $abi = $abiMap[$target]
    $src = Join-Path $ProjectRoot "target\$target\$targetDir\libphantom_ffi.so"
    $dstDir = Join-Path $ProjectRoot "app\src\main\jniLibs\$abi"
    New-Item -ItemType Directory -Path $dstDir -Force | Out-Null
    Copy-Item -Path $src -Destination (Join-Path $dstDir "libphantom_ffi.so") -Force
    Write-Host "  Copied $abi/libphantom_ffi.so" -ForegroundColor Green
}

Write-Host "=== Build complete ===" -ForegroundColor Cyan
