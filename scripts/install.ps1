#!/usr/bin/env pwsh
# Contui installer script for Windows
# Usage: powershell -ExecutionPolicy ByPass -c "irm https://raw.githubusercontent.com/aycandv/contui/main/scripts/install.ps1 | iex"

$ErrorActionPreference = 'Stop'

# Configuration
$Repo = "aycandv/contui"
$BinaryName = "contui"
$InstallDir = if ($env:INSTALL_DIR) { $env:INSTALL_DIR } else { "$env:USERPROFILE\.local\bin" }

# Print functions
function Info($Message) {
    Write-Host "info: $Message" -ForegroundColor Green
}

function Warn($Message) {
    Write-Host "warn: $Message" -ForegroundColor Yellow
}

function Error($Message) {
    Write-Host "error: $Message" -ForegroundColor Red
}

# Get latest release version from GitHub
function Get-LatestVersion {
    $url = "https://api.github.com/repos/$Repo/releases/latest"
    try {
        $response = Invoke-RestMethod -Uri $url -Method Get
        return $response.tag_name
    } catch {
        return $null
    }
}

# Download binary
function Download-Binary($Version, $Dest) {
    $arch = if ([Environment]::Is64BitOperatingSystem) { "x86_64" } else { "i686" }
    $target = "x86_64-pc-windows-msvc"
    
    $url = "https://github.com/$Repo/releases/download/$Version/${BinaryName}-${target}.zip"
    
    Info "Downloading $BinaryName $Version for $target..."
    
    try {
        Invoke-WebRequest -Uri $url -OutFile $Dest -UseBasicParsing
    } catch {
        Error "Failed to download from: $url"
        throw
    }
}

# Main installation
function Main {
    Info "Installing $BinaryName..."
    
    # Get latest version
    $version = Get-LatestVersion
    if (-not $version) {
        Error "Failed to get latest version. GitHub API may be rate limited."
        Error "Please try again later or download manually from:"
        Error "  https://github.com/$Repo/releases"
        exit 1
    }
    
    Info "Latest version: $version"
    
    # Create temp directory
    $tmpDir = Join-Path $env:TEMP ([System.Guid]::NewGuid().ToString())
    New-Item -ItemType Directory -Path $tmpDir -Force | Out-Null
    
    try {
        # Download
        $archive = Join-Path $tmpDir "$BinaryName.zip"
        Download-Binary $version $archive
        
        # Extract
        Info "Extracting..."
        Expand-Archive -Path $archive -DestinationPath $tmpDir -Force
        
        # Create install directory
        if (-not (Test-Path $InstallDir)) {
            Info "Creating directory: $InstallDir"
            New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
        }
        
        # Install
        $binaryPath = Join-Path $tmpDir "$BinaryName.exe"
        $destPath = Join-Path $InstallDir "$BinaryName.exe"
        
        Info "Installing to: $destPath"
        Copy-Item -Path $binaryPath -Destination $destPath -Force
        
        # Verify installation
        if (Get-Command $BinaryName -ErrorAction SilentlyContinue) {
            Info "Successfully installed $BinaryName!"
        } elseif (Test-Path $destPath) {
            Info "Successfully installed $BinaryName!"
            Warn "$InstallDir is not in your PATH"
            Warn "Add the following to your PowerShell profile:"
            Warn "  `$env:Path = \"$InstallDir;`$env:Path\""
        } else {
            Error "Installation failed"
            exit 1
        }
        
        # Post-install message
        Write-Host ""
        Info "To get started:"
        Write-Host "  $BinaryName --help"
        Write-Host ""
        Info "For more information, visit:"
        Write-Host "  https://github.com/$Repo"
    } finally {
        # Cleanup
        Remove-Item -Recurse -Force $tmpDir -ErrorAction SilentlyContinue
    }
}

# Run main
Main
