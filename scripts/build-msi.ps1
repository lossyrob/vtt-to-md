#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Build MSI installer locally for vtt-to-md

.DESCRIPTION
    This script installs WiX Toolset and cargo-wix if needed, then builds the MSI installer.
    It handles the complete build process including dependencies.

.PARAMETER Version
    Optional version string to set in Cargo.toml (e.g., "0.2.0"). If not specified, uses the version from Cargo.toml.

.PARAMETER SkipWixInstall
    Skip WiX Toolset installation check (useful if already installed)

.EXAMPLE
    .\scripts\build-msi.ps1
    Builds MSI with current version from Cargo.toml

.EXAMPLE
    .\scripts\build-msi.ps1 -Version "0.2.0"
    Builds MSI with version 0.2.0
#>

param(
    [Parameter(Mandatory=$false)]
    [string]$Version,
    
    [Parameter(Mandatory=$false)]
    [switch]$SkipWixInstall
)

$ErrorActionPreference = "Stop"

Write-Host "=== VTT-to-MD MSI Builder ===" -ForegroundColor Cyan

# Check if cargo-wix is installed
Write-Host "`nChecking for cargo-wix..." -ForegroundColor Yellow
if (-not (Get-Command cargo-wix -ErrorAction SilentlyContinue)) {
    Write-Host "cargo-wix not found. Installing..." -ForegroundColor Yellow
    cargo install cargo-wix
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Failed to install cargo-wix"
        exit 1
    }
    Write-Host "cargo-wix installed successfully" -ForegroundColor Green
} else {
    Write-Host "cargo-wix is already installed" -ForegroundColor Green
}

# Check if WiX Toolset is installed
if (-not $SkipWixInstall) {
    Write-Host "`nChecking for WiX Toolset..." -ForegroundColor Yellow
    
    $wixInstalled = $false
    
    # Check if candle.exe is in PATH
    if (Get-Command candle.exe -ErrorAction SilentlyContinue) {
        Write-Host "WiX Toolset found in PATH" -ForegroundColor Green
        $wixInstalled = $true
    }
    
    # Check common WiX installation locations
    $wixPaths = @(
        "${env:ProgramFiles(x86)}\WiX Toolset v3.11\bin",
        "${env:ProgramFiles}\WiX Toolset v3.11\bin",
        "$PSScriptRoot\..\wix-toolset"
    )
    
    foreach ($path in $wixPaths) {
        if (Test-Path "$path\candle.exe") {
            Write-Host "WiX Toolset found at: $path" -ForegroundColor Green
            $env:PATH = "$path;$env:PATH"
            $wixInstalled = $true
            break
        }
    }
    
    if (-not $wixInstalled) {
        Write-Host "WiX Toolset not found. Installing locally..." -ForegroundColor Yellow
        
        $wixDir = Join-Path $PSScriptRoot "..\wix-toolset"
        
        if (-not (Test-Path $wixDir)) {
            # Download WiX 3.11.2 binaries
            $wixZip = Join-Path $PSScriptRoot "..\wix.zip"
            Write-Host "Downloading WiX Toolset 3.11.2..." -ForegroundColor Yellow
            Invoke-WebRequest -Uri "https://github.com/wixtoolset/wix3/releases/download/wix3112rtm/wix311-binaries.zip" -OutFile $wixZip
            
            Write-Host "Extracting WiX Toolset..." -ForegroundColor Yellow
            Expand-Archive -Path $wixZip -DestinationPath $wixDir -Force
            Remove-Item $wixZip
        }
        
        # Add to PATH
        $env:PATH = "$wixDir;$env:PATH"
        Write-Host "WiX Toolset installed locally" -ForegroundColor Green
        
        # Verify installation
        & "$wixDir\candle.exe" -? | Out-Null
        if ($LASTEXITCODE -ne 0) {
            Write-Error "WiX Toolset installation verification failed"
            exit 1
        }
    }
}

# Update version if specified
if ($Version) {
    Write-Host "`nUpdating Cargo.toml version to $Version..." -ForegroundColor Yellow
    $cargoToml = Join-Path $PSScriptRoot "..\Cargo.toml"
    (Get-Content $cargoToml) -replace 'version = "\d+\.\d+\.\d+"', "version = `"$Version`"" | Set-Content $cargoToml
    Write-Host "Version updated" -ForegroundColor Green
} else {
    Write-Host "`nUsing version from Cargo.toml" -ForegroundColor Yellow
}

# Build the MSI
Write-Host "`nBuilding MSI installer..." -ForegroundColor Yellow
Push-Location (Join-Path $PSScriptRoot "..")
try {
    cargo wix --nocapture
    if ($LASTEXITCODE -ne 0) {
        Write-Error "MSI build failed"
        exit 1
    }
} finally {
    Pop-Location
}

# Find the built MSI
$msiFile = Get-ChildItem -Path (Join-Path $PSScriptRoot "..\target\wix\*.msi") | Select-Object -First 1

if ($msiFile) {
    Write-Host "`n=== Build Successful ===" -ForegroundColor Green
    Write-Host "MSI location: $($msiFile.FullName)" -ForegroundColor Cyan
    Write-Host "MSI size: $([math]::Round($msiFile.Length / 1MB, 2)) MB" -ForegroundColor Cyan
} else {
    Write-Error "MSI file not found after build"
    exit 1
}
