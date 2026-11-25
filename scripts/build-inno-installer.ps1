param(
    [switch]$Rebuild
)

$ErrorActionPreference = "Stop"
$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
Push-Location $RepoRoot

try {
    $exePath = Join-Path $RepoRoot "src-tauri\target\release\antigravity-client.exe"

    if ($Rebuild -or -not (Test-Path $exePath)) {
        Write-Host "Building Windows release binary first..." -ForegroundColor Cyan
        & powershell -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot "build-windows.ps1") -NoBundle
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    }

    if (-not (Test-Path $exePath)) {
        throw "Release executable not found at $exePath"
    }

    # Read version from src-tauri/tauri.conf.json
    $tauriConf = Get-Content (Join-Path $RepoRoot "src-tauri\tauri.conf.json") -Raw | ConvertFrom-Json
    $appVersion = $tauriConf.version
    if (-not $appVersion) { $appVersion = "0.1.0" }

    # Locate Inno Setup Compiler (ISCC.exe)
    $iscc = $null
    $cmd = Get-Command "ISCC.exe" -ErrorAction SilentlyContinue
    if ($cmd) {
        $iscc = $cmd.Source
    } else {
        $candidates = @(
            "$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe",
            "C:\Program Files (x86)\Inno Setup 6\ISCC.exe",
            "C:\Program Files\Inno Setup 6\ISCC.exe"
        )
        $iscc = $candidates | Where-Object { Test-Path $_ } | Select-Object -First 1
    }

    if (-not $iscc) {
        throw "Inno Setup 6 (ISCC.exe) not found. Install it with: winget install JRSoftware.InnoSetup"
    }

    $issScript = Join-Path $RepoRoot "installer\antigravity-client.iss"
    Write-Host "Using Inno Setup Compiler: $iscc" -ForegroundColor DarkGray
    Write-Host "Compiling installer for Antigravity Client v$appVersion..." -ForegroundColor Cyan

    & $iscc "/DMyAppVersion=$appVersion" $issScript
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    $outFile = Join-Path $RepoRoot "installer\output\Antigravity-Client-$appVersion-x64-Setup.exe"
    if (Test-Path $outFile) {
        $mb = [math]::Round((Get-Item $outFile).Length / 1MB, 2)
        Write-Host ""
        Write-Host "Inno Setup installer created successfully!" -ForegroundColor Green
        Write-Host "Installer: $outFile ($mb MB)"
    }
} finally {
    Pop-Location
}
