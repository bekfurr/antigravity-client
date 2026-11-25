param(
    [switch]$Release,
    [ValidateSet("aarch64", "armv7", "x86_64", "all")]
    [string]$Target = "aarch64",
    [switch]$Install
)

$ErrorActionPreference = "Stop"
$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
Push-Location $RepoRoot

try {
    # Locate JDK 21 (recommended for Android Gradle Plugin)
    if (-not $env:JAVA_HOME -or -not (Test-Path (Join-Path $env:JAVA_HOME "bin\java.exe")) -or ($env:JAVA_HOME -match "jdk-27")) {
        $jdkCandidates = @(
            (Get-ChildItem "C:\Program Files\Java" -Directory -Filter "jdk-21*" -ErrorAction SilentlyContinue | Select-Object -First 1 -ExpandProperty FullName),
            "C:\Program Files\Android\Android Studio\jbr"
        ) | Where-Object { $_ -and (Test-Path (Join-Path $_ "bin\java.exe")) }

        if ($jdkCandidates.Count -gt 0) {
            $env:JAVA_HOME = $jdkCandidates[0]
            $env:PATH = "$($env:JAVA_HOME)\bin;$env:PATH"
        }
    }

    # Locate Android SDK
    if (-not $env:ANDROID_HOME) {
        $defaultSdk = Join-Path $env:LOCALAPPDATA "Android\Sdk"
        if (Test-Path $defaultSdk) {
            $env:ANDROID_HOME = $defaultSdk
        } else {
            throw "ANDROID_HOME is not set and default Android SDK was not found at $defaultSdk"
        }
    }

    # Locate Android NDK
    if (-not $env:NDK_HOME) {
        $ndkRoot = Join-Path $env:ANDROID_HOME "ndk"
        if (Test-Path $ndkRoot) {
            $ndk = Get-ChildItem $ndkRoot -Directory | Sort-Object Name | Select-Object -First 1
            if ($ndk) {
                $env:NDK_HOME = $ndk.FullName
            }
        }
        if (-not $env:NDK_HOME) {
            throw "NDK_HOME is not set and no NDK was found under $ndkRoot"
        }
    }

    Write-Host "JAVA_HOME:    $env:JAVA_HOME" -ForegroundColor DarkGray
    Write-Host "ANDROID_HOME: $env:ANDROID_HOME" -ForegroundColor DarkGray
    Write-Host "NDK_HOME:     $env:NDK_HOME" -ForegroundColor DarkGray

    if (-not (Test-Path "node_modules")) {
        Write-Host "Installing Tauri CLI dependencies..." -ForegroundColor Cyan
        npm install --no-audit --no-fund
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    }

    $tauriArgs = @("tauri", "android", "build", "--apk")
    if (-not $Release) {
        $tauriArgs += "--debug"
    }
    if ($Target -ne "all") {
        $tauriArgs += @("--target", $Target)
    }

    Write-Host "Building Antigravity Client for Android ($Target, $(if ($Release) { 'release' } else { 'debug' }))..." -ForegroundColor Cyan
    & npx @tauriArgs
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    $apkDir = Join-Path $RepoRoot "src-tauri\gen\android\app\build\outputs\apk"
    $apks = Get-ChildItem -Recurse $apkDir -Filter "*.apk" -ErrorAction SilentlyContinue | Sort-Object LastWriteTime -Descending
    Write-Host ""
    Write-Host "Android build complete!" -ForegroundColor Green
    foreach ($apk in $apks) {
        Write-Host "APK: $($apk.FullName)"
    }

    if ($Install -and $apks.Count -gt 0) {
        $adb = Join-Path $env:ANDROID_HOME "platform-tools\adb.exe"
        if (Test-Path $adb) {
            $apkToInstall = $apks[0].FullName
            Write-Host "Installing $apkToInstall via ADB..." -ForegroundColor Cyan
            & $adb install -r $apkToInstall
            if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
        } else {
            Write-Warning "adb.exe not found at $adb"
        }
    }
} finally {
    Pop-Location
}
