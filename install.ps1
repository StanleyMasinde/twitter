#!/usr/bin/env pwsh

param(
    [string]$Version = "latest"
)

$ErrorActionPreference = "Stop"

$Repo = "StanleyMasinde/twitter"
$InstallDir = if ($env:TWITTER_INSTALL) { $env:TWITTER_INSTALL } else { Join-Path $env:USERPROFILE "bin" }

function Get-Platform {
    $isWindowsOs = $false
    if ($env:OS -eq "Windows_NT") {
        $isWindowsOs = $true
    }
    else {
        try {
            $isWindowsOs = [System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform(
                [System.Runtime.InteropServices.OSPlatform]::Windows
            )
        }
        catch {
            $isWindowsOs = $false
        }
    }

    if (-not $isWindowsOs) {
        throw "Unsupported OS. install.ps1 is for Windows only."
    }

    $arch = $null
    try {
        $arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()
    }
    catch {
        $procArch = $env:PROCESSOR_ARCHITEW6432
        if (-not $procArch) {
            $procArch = $env:PROCESSOR_ARCHITECTURE
        }
        switch -Regex ($procArch) {
            "^(AMD64|X86_64)$" { $arch = "X64" }
            "^(ARM64)$" { $arch = "Arm64" }
            default { $arch = $procArch }
        }
    }

    switch ($arch) {
        "X64" { return "windows-x86_64" }
        "Arm64" { return "windows-aarch64" }
        default { throw "Unsupported architecture: $arch" }
    }
}

function Get-ReleaseData([string]$RequestedVersion) {
    if ($RequestedVersion -eq "latest") {
        $apiUrl = "https://api.github.com/repos/$Repo/releases/latest"
    }
    else {
        $apiUrl = "https://api.github.com/repos/$Repo/releases/tags/$RequestedVersion"
    }

    try {
        return Invoke-RestMethod -Uri $apiUrl -Headers @{ "User-Agent" = "twitter-installer" }
    }
    catch {
        throw "Could not fetch release data from GitHub API."
    }
}

function Verify-Checksum([string]$FilePath, [string]$Digest) {
    if ([string]::IsNullOrWhiteSpace($Digest)) {
        Write-Warning "No checksum available for this release; skipping verification (expected for older releases)."
        return
    }

    $expected = $Digest -replace "^sha256:", ""
    Write-Host "Verifying checksum..."

    $actual = (Get-FileHash -Algorithm SHA256 -Path $FilePath).Hash.ToLowerInvariant()
    if ($actual -ne $expected.ToLowerInvariant()) {
        throw "Checksum verification failed. Expected $expected, got $actual"
    }

    Write-Host "Checksum verified: $expected"
}

function Add-ToPath([string]$Dir) {
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if (-not $userPath) {
        $userPath = ""
    }

    $needle = $Dir.TrimEnd("\\")
    $alreadyInUserPath = $false

    foreach ($entry in ($userPath -split ";" | Where-Object { $_ -and $_.Trim() })) {
        if ($entry.TrimEnd("\\") -ieq $needle) {
            $alreadyInUserPath = $true
            break
        }
    }

    if (-not $alreadyInUserPath) {
        $newUserPath = if ([string]::IsNullOrWhiteSpace($userPath)) { $Dir } else { "$userPath;$Dir" }
        [Environment]::SetEnvironmentVariable("Path", $newUserPath, "User")
        Write-Host "Added $Dir to user PATH"
    }

    $sessionEntries = $env:Path -split ";"
    $alreadyInSessionPath = $false
    foreach ($entry in $sessionEntries) {
        if ($entry -and $entry.TrimEnd("\\") -ieq $needle) {
            $alreadyInSessionPath = $true
            break
        }
    }

    if (-not $alreadyInSessionPath) {
        $env:Path = if ([string]::IsNullOrWhiteSpace($env:Path)) { $Dir } else { "$env:Path;$Dir" }
    }
}

function Install-Twitter([string]$RequestedVersion) {
    $platform = Get-Platform
    $filename = "twitter-$platform.zip"

    Write-Host "Twitter CLI Installer"
    Write-Host ""
    Write-Host "Fetching release information..."

    $release = Get-ReleaseData -RequestedVersion $RequestedVersion
    $resolvedVersion = if ($RequestedVersion -eq "latest") { $release.tag_name } else { $RequestedVersion }

    if (-not $resolvedVersion) {
        throw "Could not determine release version from API response."
    }

    Write-Host "Version:  $resolvedVersion"
    Write-Host "Platform: $platform"
    Write-Host ""

    $asset = $release.assets | Where-Object { $_.name -eq $filename } | Select-Object -First 1
    if (-not $asset) {
        $available = @($release.assets | Where-Object { $_.name -like "twitter-*" } | ForEach-Object { $_.name })
        $msg = "Could not find asset '$filename' in release."
        if ($available.Count -gt 0) {
            $msg += " Available assets: $($available -join ', ')"
        }
        throw $msg
    }

    $downloadUrl = $asset.browser_download_url
    if (-not $downloadUrl) {
        throw "Could not extract download URL from release data."
    }

    $tempDir = Join-Path ([System.IO.Path]::GetTempPath()) ("twitter-install-" + [System.Guid]::NewGuid().ToString("N"))
    New-Item -ItemType Directory -Path $tempDir -Force | Out-Null

    try {
        $archivePath = Join-Path $tempDir $filename

        Write-Host "Downloading from: $downloadUrl"
        Invoke-WebRequest -Uri $downloadUrl -OutFile $archivePath
        Write-Host ""

        Verify-Checksum -FilePath $archivePath -Digest $asset.digest
        Write-Host ""

        Write-Host "Extracting..."
        Expand-Archive -Path $archivePath -DestinationPath $tempDir -Force

        $exePath = Join-Path $tempDir "twitter.exe"
        if (-not (Test-Path -Path $exePath -PathType Leaf)) {
            throw "Binary not found after extraction."
        }

        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
        $destination = Join-Path $InstallDir "twitter.exe"

        Write-Host "Installing to $destination..."
        Copy-Item -Path $exePath -Destination $destination -Force

        Add-ToPath -Dir $InstallDir

        Write-Host ""
        Write-Host "Twitter CLI was installed successfully to $destination"
        Write-Host "Run 'twitter --help' to get started"
    }
    finally {
        if (Test-Path $tempDir) {
            Remove-Item -Path $tempDir -Recurse -Force -ErrorAction SilentlyContinue
        }
    }
}

if ($Version -in @("-h", "--help", "help")) {
    Write-Host @"
Twitter CLI Installer (Windows)

Usage:
  powershell -ExecutionPolicy Bypass -File .\install.ps1
  powershell -ExecutionPolicy Bypass -File .\install.ps1 v1.5.0
  pwsh -File .\install.ps1
  pwsh -File .\install.ps1 v1.5.0

Environment Variables:
  TWITTER_INSTALL    Installation directory (default: $InstallDir)
"@
    exit 0
}

Install-Twitter -RequestedVersion $Version
