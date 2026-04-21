# install.ps1 — Build and install fed on Windows
# Run from the project root:  .\install.ps1
# Optional: .\install.ps1 -InstallDir "C:\Tools"

param(
    [string]$InstallDir = "$env:USERPROFILE\.cargo\bin"
)

$ErrorActionPreference = "Stop"
$BinaryName = "fed.exe"
$ReleasePath = "target\release\$BinaryName"

function Check-Command($cmd) {
    return [bool](Get-Command $cmd -ErrorAction SilentlyContinue)
}

# ── Prerequisites ──────────────────────────────────────────────────────────────

if (-not (Check-Command "cargo")) {
    Write-Error "Rust / cargo not found. Install from https://rustup.rs and re-run."
    exit 1
}

Write-Host ""
Write-Host "  fed — install script for Windows" -ForegroundColor Cyan
Write-Host "  Install directory: $InstallDir" -ForegroundColor Gray
Write-Host ""

# ── Build ──────────────────────────────────────────────────────────────────────

Write-Host "[1/3] Building release binary..." -ForegroundColor Yellow
cargo build --release

if ($LASTEXITCODE -ne 0) {
    Write-Error "Build failed (cargo exit code $LASTEXITCODE)."
    exit 1
}

Write-Host "      Build OK" -ForegroundColor Green

# ── Create install directory if needed ────────────────────────────────────────

if (-not (Test-Path $InstallDir)) {
    Write-Host "[2/3] Creating install directory..." -ForegroundColor Yellow
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
} else {
    Write-Host "[2/3] Install directory exists, skipping." -ForegroundColor Gray
}

# ── Copy binary ───────────────────────────────────────────────────────────────

Write-Host "[3/3] Copying $BinaryName to $InstallDir..." -ForegroundColor Yellow
Copy-Item -Path $ReleasePath -Destination (Join-Path $InstallDir $BinaryName) -Force
Write-Host "      Installed OK" -ForegroundColor Green

# ── PATH check ────────────────────────────────────────────────────────────────

$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$InstallDir*") {
    Write-Host ""
    Write-Host "  NOTE: '$InstallDir' is not in your PATH." -ForegroundColor Magenta
    $add = Read-Host "  Add it now? (y/N)"
    if ($add -match "^[Yy]$") {
        [Environment]::SetEnvironmentVariable(
            "Path",
            "$userPath;$InstallDir",
            "User"
        )
        Write-Host "  PATH updated. Restart your terminal to apply." -ForegroundColor Green
    }
}

Write-Host ""
Write-Host "  Done! Try:  fed yourfile.pdf" -ForegroundColor Cyan
Write-Host ""
