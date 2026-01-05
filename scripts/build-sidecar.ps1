# Build the mimir-mcp sidecar binary for Tauri bundling (Windows)
# This script builds mimir-mcp and copies it to the binaries directory
# with the target-triple suffix that Tauri expects.

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir
$BinariesDir = Join-Path $ProjectRoot "crates\mimir-dm\binaries"

# Detect target triple
if ($env:TAURI_ENV_TARGET_TRIPLE) {
    $Target = $env:TAURI_ENV_TARGET_TRIPLE
} elseif ($args[0]) {
    $Target = $args[0]
} else {
    # Auto-detect based on platform
    $Arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture
    switch ($Arch) {
        "X64" { $Target = "x86_64-pc-windows-msvc" }
        "Arm64" { $Target = "aarch64-pc-windows-msvc" }
        default {
            Write-Error "Unsupported architecture: $Arch"
            exit 1
        }
    }
}

Write-Host "Building mimir-mcp for target: $Target"

# Build mimir-mcp in release mode
Push-Location $ProjectRoot
try {
    cargo build --release -p mimir-dm-mcp --target $Target
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Cargo build failed"
        exit 1
    }
} finally {
    Pop-Location
}

# Ensure binaries directory exists
if (-not (Test-Path $BinariesDir)) {
    New-Item -ItemType Directory -Path $BinariesDir -Force | Out-Null
}

# Copy binary with target suffix
$Source = Join-Path $ProjectRoot "target\$Target\release\mimir-mcp.exe"
$Dest = Join-Path $BinariesDir "mimir-mcp-$Target.exe"

if (Test-Path $Source) {
    Copy-Item -Path $Source -Destination $Dest -Force
    Write-Host "Copied: $Dest"
} else {
    Write-Error "Error: Binary not found at $Source"
    exit 1
}

Write-Host "Sidecar build complete!"
