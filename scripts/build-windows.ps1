param(
    [switch]$NoBundle,
    [string]$Bundles = ""
)

$ErrorActionPreference = "Stop"
$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
Push-Location $RepoRoot

try {
    if (-not (Test-Path "node_modules")) {
        Write-Host "Installing Tauri CLI dependencies..." -ForegroundColor Cyan
        npm install --no-audit --no-fund
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    }

    Write-Host "Running Rust unit tests..." -ForegroundColor Cyan
    cargo test --manifest-path src-tauri\Cargo.toml
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    $tauriArgs = @("tauri", "build")
    if ($NoBundle) {
        $tauriArgs += "--no-bundle"
    } elseif ($Bundles -ne "") {
        $tauriArgs += @("--bundles", $Bundles)
    }

    Write-Host "Building Antigravity Client for Windows..." -ForegroundColor Cyan
    & npx @tauriArgs
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    $exePath = Join-Path $RepoRoot "src-tauri\target\release\antigravity-client.exe"
    Write-Host ""
    Write-Host "Windows build complete!" -ForegroundColor Green
    if (Test-Path $exePath) {
        Write-Host "Executable: $exePath"
    }
    $bundleDir = Join-Path $RepoRoot "src-tauri\target\release\bundle"
    if (Test-Path $bundleDir) {
        Get-ChildItem -Recurse $bundleDir -Include *.exe,*.msi | ForEach-Object {
            Write-Host "Installer:  $($_.FullName)"
        }
    }
} finally {
    Pop-Location
}
