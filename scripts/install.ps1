# VoxForg Turnkey One-Line Installer for Windows PowerShell
# Usage: irm https://raw.githubusercontent.com/AMG555/VoxForg/main/scripts/install.ps1 | iex

$ErrorActionPreference = "Stop"

$Repo = "AMG555/VoxForg"
$InstallBase = "$env:LOCALAPPDATA\Programs\VoxForg"
$BinDir = "$InstallBase\bin"
$TargetExe = "$BinDir\voxforg.exe"

Write-Host @"
  _    _            ______                 
 | |  | |          |  ____|                
 | |  | | _____  __| |__ ___  _ __ __ _    
 | |  | |/ _ \ \/ /|  __/ _ \| '__/ _` |   
  \ \/ / (_) >  < | | | (_) | | | (_| |   
   \__/ \___/_/\_\|_|  \___/|_|  \__, |   
                                  __/ |   
                                 |___/    
"@ -ForegroundColor Yellow

Write-Host "VoxForg Standalone Workstation Installer for Windows" -ForegroundColor White
Write-Host ""

# 1. Architecture check
$Arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture
if ($Arch -ne "X64" -and $Arch -ne "Arm64") {
    Write-Error "Unsupported CPU architecture: $Arch. VoxForg requires x64 or Arm64 Windows."
    exit 1
}

$BinaryName = "voxforg-windows-x86_64.exe"
Write-Host "Detected Architecture: $Arch" -ForegroundColor Cyan

# 2. Ensure install directory exists
if (-not (Test-Path $BinDir)) {
    New-Item -ItemType Directory -Force -Path $BinDir | Out-Null
}

# 3. Retrieve latest release tag
$LatestTag = "v0.1.0"
try {
    $ReleaseInfo = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest" -Headers @{"User-Agent"="VoxForg-Installer"} -ErrorAction SilentlyContinue
    if ($ReleaseInfo -and $ReleaseInfo.tag_name) {
        $LatestTag = $ReleaseInfo.tag_name
    }
} catch {
    # Keep fallback tag
}

$DownloadUrl = "https://github.com/$Repo/releases/download/$LatestTag/$BinaryName"
Write-Host "Downloading VoxForg $LatestTag..." -ForegroundColor Cyan

try {
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $TargetExe -UseBasicParsing
    Write-Host "Successfully downloaded to $TargetExe" -ForegroundColor Green
} catch {
    Write-Warning "Direct release download unavailable. Checking for local cargo installation..."
    if (Get-Command cargo -ErrorAction SilentlyContinue) {
        cargo install --git "https://github.com/$Repo.git" voxforg-cli --root $InstallBase
        Write-Host "Successfully built and installed via cargo." -ForegroundColor Green
    } else {
        Write-Error "Failed to download binary and cargo was not found. Please install Rust or check release assets."
        exit 1
    }
}

# 4. Persist to User PATH
$UserPath = [Environment]::GetEnvironmentVariable("Path", [EnvironmentVariableTarget]::User)
if ($UserPath -notlike "*$BinDir*") {
    Write-Host "Adding $BinDir to User PATH..." -ForegroundColor Cyan
    $NewPath = "$UserPath;$BinDir".Trim(';')
    [Environment]::SetEnvironmentVariable("Path", $NewPath, [EnvironmentVariableTarget]::User)
    $env:PATH = "$env:PATH;$BinDir"
    Write-Host "PATH updated successfully." -ForegroundColor Green
}

# 5. Hardware validation
Write-Host "`nProbing system hardware..." -ForegroundColor Cyan
& $TargetExe hardware

Write-Host @"

==================================================
  VoxForg Standalone Workstation Ready!
==================================================

Run workstation UI:   voxforg serve --open
Direct synthesis:     voxforg synth "Hello from VoxForg"
Documentation & API:  http://localhost:8080/docs

"@ -ForegroundColor Green
