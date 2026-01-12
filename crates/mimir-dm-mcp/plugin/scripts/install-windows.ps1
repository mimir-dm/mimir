# Install Mimir MCP server for Claude Desktop on Windows
# Run with: powershell -ExecutionPolicy Bypass -File install-windows.ps1

param(
    [string]$MimirMcpBin = "",
    [string]$MimirDb = ""
)

$ErrorActionPreference = "Stop"

# Default paths
$DefaultBinLocations = @(
    ".\target\release\mimir-mcp.exe",
    "..\..\..\target\release\mimir-mcp.exe",
    "$env:USERPROFILE\.cargo\bin\mimir-mcp.exe"
)
$DefaultDbPath = "$env:APPDATA\com.mimir.mimir\mimir.db"

# Use parameters, environment variables, or defaults
if (-not $MimirMcpBin) {
    $MimirMcpBin = $env:MIMIR_MCP_BIN
}
if (-not $MimirDb) {
    $MimirDb = if ($env:MIMIR_DATABASE_PATH) { $env:MIMIR_DATABASE_PATH } else { $DefaultDbPath }
}

$ClaudeConfigDir = "$env:APPDATA\Claude"
$ClaudeConfig = "$ClaudeConfigDir\claude_desktop_config.json"

Write-Host "Mimir MCP Installer for Claude Desktop" -ForegroundColor Cyan
Write-Host "=======================================" -ForegroundColor Cyan
Write-Host ""

# Find mimir-mcp binary if not specified
if (-not $MimirMcpBin) {
    foreach ($location in $DefaultBinLocations) {
        if (Test-Path $location) {
            $MimirMcpBin = $location
            break
        }
    }
}

# Check if mimir-mcp binary exists
if (-not $MimirMcpBin -or -not (Test-Path $MimirMcpBin)) {
    Write-Host "Error: mimir-mcp binary not found" -ForegroundColor Red
    Write-Host ""
    Write-Host "Searched locations:"
    foreach ($location in $DefaultBinLocations) {
        Write-Host "  - $location"
    }
    Write-Host ""
    Write-Host "Build it first with: cargo build --release -p mimir-dm-mcp"
    Write-Host "Or specify the path: .\install-windows.ps1 -MimirMcpBin C:\path\to\mimir-mcp.exe"
    exit 1
}

# Convert to absolute path
$MimirMcpBin = (Resolve-Path $MimirMcpBin).Path

# Check if database exists
if (-not (Test-Path $MimirDb)) {
    Write-Host "Warning: Mimir database not found at: $MimirDb" -ForegroundColor Yellow
    Write-Host "The MCP server will fail until a database exists."
    Write-Host ""
}

# Create Claude config directory if needed
if (-not (Test-Path $ClaudeConfigDir)) {
    New-Item -ItemType Directory -Path $ClaudeConfigDir -Force | Out-Null
}

# Generate the config
$ConfigObject = @{
    mcpServers = @{
        mimir = @{
            command = $MimirMcpBin
            args = @()
            env = @{
                MIMIR_DATABASE_PATH = $MimirDb
            }
        }
    }
}

$ConfigJson = $ConfigObject | ConvertTo-Json -Depth 4

# Check if config already exists
if (Test-Path $ClaudeConfig) {
    Write-Host "Existing Claude config found at: $ClaudeConfig" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Add this to your mcpServers section:"
    Write-Host ""
    Write-Host '    "mimir": {'
    Write-Host "      `"command`": `"$($MimirMcpBin -replace '\\', '\\')`","
    Write-Host '      "args": [],'
    Write-Host '      "env": {'
    Write-Host "        `"MIMIR_DATABASE_PATH`": `"$($MimirDb -replace '\\', '\\')`""
    Write-Host '      }'
    Write-Host '    }'
    Write-Host ""
    Write-Host "Or backup and replace with:" -ForegroundColor Yellow
    Write-Host "  Copy-Item `"$ClaudeConfig`" `"$ClaudeConfig.backup`""
    Write-Host ""
} else {
    # Write new config
    $ConfigJson | Out-File -FilePath $ClaudeConfig -Encoding UTF8
    Write-Host "Created Claude config at: $ClaudeConfig" -ForegroundColor Green
}

Write-Host ""
Write-Host "Configuration:"
Write-Host "  Binary: $MimirMcpBin"
Write-Host "  Database: $MimirDb"
Write-Host ""
Write-Host "Done! Restart Claude Desktop to load the MCP server." -ForegroundColor Green
