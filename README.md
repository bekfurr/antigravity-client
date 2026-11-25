# Antigravity Client

A lightweight, native cross-platform client for **Google Antigravity Remote Control** (`https://antigravity.google.com`) built with **Rust** and **Tauri v2**, targeting **Windows** and **Android**.

Instead of opening a browser and signing in repeatedly, Antigravity Client wraps the official Antigravity Remote Control web interface in a dedicated native WebView with persistent session storage, saved remote instance bookmarks, and platform-native ergonomics.

## Features

### Shared (Windows & Android)
- **Native WebView Wrapper** — Loads `https://antigravity.google.com` directly using the OS native WebView engine (Edge WebView2 on Windows, Android System WebView on Android).
- **Persistent Sessions** — Cookies, storage, and login state persist across restarts.
- **Smart Navigation Routing** — Keeps Antigravity and Google account authentication flows inside the client while opening external third-party links in your default system browser.
- **Remote Instance Bookmarks** — Save and jump directly to `https://antigravity.google.com/r/<instance-id>` links for your workstations or servers.
- **English Interface** — Clean dark-themed native chrome and dialogs matching Antigravity's visual style.

### Windows
- **Multi-Account Profiles** — Create and switch isolated profiles (`Default`, `Work`, `Personal`, etc.), each backed by its own persistent WebView2 storage folder under `%LOCALAPPDATA%`.
- **Native Menu Bar & Keyboard Shortcuts**:
  - `Ctrl+,` — Settings & Profiles
  - `Ctrl+L` — Connect to Instance URL
  - `Ctrl+R` / `Ctrl+Shift+R` — Reload / Hard Reload
  - `Alt+Left` / `Alt+Right` / `Alt+Home` — Back / Forward / Home
  - `Ctrl+=` / `Ctrl+-` / `Ctrl+0` — Zoom In / Zoom Out / Actual Size
  - `F11` — Toggle Fullscreen
  - `Ctrl+Q` — Quit
- **System Tray** — Quick show/hide, Home, Reload, Settings, plus optional "Minimize to system tray on close".
- **Single Instance & CLI URLs** — Launching again focuses the running window; passing an Antigravity URL (e.g. `antigravity-client.exe https://antigravity.google.com/r/<id>`) navigates directly to that instance.
- **Persistent Window State** — Remembers window position, size, and maximized state.

### Android
- **Edge-to-Edge & Keyboard (IME) Safe Insets** — Dark status/navigation bars with proper cutout and soft-keyboard handling so the chat composer is never obscured.
- **Collapsible Quick Toolbar** — Compact top bar with Back, Home, Instances/Connect dialog (with saved bookmarks & Clear Session), Reload, Open in System Browser, and Hide/Fullscreen toggle.
- **Hardware Back Navigation** — Navigates through WebView history before exiting.
- **Persistent Cookie Flushing** — Explicitly enables third-party cookies and flushes `CookieManager` on pause/stop.
- **Deep Link Support** — Registers intent filters for `https://antigravity.google.com` and `antigravity://` links.
- **Cleaned WebView User-Agent** — Strips `; wv` and `Version/x.x` markers while retaining the device's actual Chromium version.

> **Note on Google Sign-In on Android:** Google enforces strict security policies for `accounts.google.com` inside embedded Android WebViews and may occasionally display a sign-in warning depending on account/device policies. If that occurs, tap the **↗ (Open in system browser)** button in the toolbar to sign in or access Antigravity in Chrome.

## Project Structure

```text
antigravity-client/
├── assets/                  # Source 1024x1024 app icon
├── installer/
│   └── antigravity-client.iss # Inno Setup 6 Windows installer script
├── scripts/
│   ├── build-windows.ps1    # Automated Windows test + release build script
│   ├── build-inno-installer.ps1 # Compiles Windows release .exe + Inno Setup installer
│   └── build-android.ps1    # Automated Android APK build + optional ADB install script
├── ui/
│   ├── index.html           # Local startup fallback screen
│   └── settings.html        # Desktop Settings, Profiles & Quick-Connect UI
└── src-tauri/
    ├── Cargo.toml           # Rust dependencies (desktop & mobile gated)
    ├── tauri.conf.json      # Tauri v2 configuration
    ├── capabilities/        # Tauri v2 permission capabilities
    ├── src/
    │   ├── lib.rs           # Shared Tauri app entry point (Windows + Android)
    │   ├── main.rs          # Desktop binary entry point
    │   ├── config.rs        # Persistent JSON config, URL normalization & profiles
    │   ├── navigation.rs    # In-app vs external link & CLI URL routing
    │   ├── commands.rs      # Tauri IPC commands for Settings & Quick-Connect
    │   └── desktop.rs       # Windows/desktop menu bar, system tray & settings window
    └── gen/android/         # Generated Android project + custom MainActivity.kt
```

## Prerequisites

- **Rust** (`rustup`, `cargo`) with targets:
  - Windows: `x86_64-pc-windows-msvc`
  - Android: `aarch64-linux-android` (and optionally `armv7-linux-androideabi`, `x86_64-linux-android`)
- **Node.js / npm** (used solely to run `@tauri-apps/cli`)
- **Windows**: Microsoft Edge WebView2 Runtime (pre-installed on Windows 10/11)
- **Android**: JDK 21, Android SDK, and Android NDK

## Building & Running

### Run on Windows (Development)
```powershell
npm install
npm run dev
```

### Build for Windows (Inno Setup Installer)
Requires [Inno Setup 6](https://jrsoftware.org/isinfo.php) (`winget install JRSoftware.InnoSetup`):
```powershell
.\scripts\build-inno-installer.ps1
```
Pass `-Rebuild` to recompile the Rust binary before packaging the installer:
```powershell
.\scripts\build-inno-installer.ps1 -Rebuild
```
Output:
- Installer: `installer\output\Antigravity-Client-0.1.0-x64-Setup.exe`

### Build for Windows (Tauri Release `.exe` / MSI / NSIS)
```powershell
.\scripts\build-windows.ps1
```
Build only the standalone executable without MSI/NSIS installers:
```powershell
.\scripts\build-windows.ps1 -NoBundle
```
Outputs:
- Executable: `src-tauri\target\release\antigravity-client.exe`
- Installers: `src-tauri\target\release\bundle\msi\` and `src-tauri\target\release\bundle\nsis\`

### Build for Android (APK)
```powershell
.\scripts\build-android.ps1
```
Build and install directly onto a connected ADB device:
```powershell
.\scripts\build-android.ps1 -Install
```
Build a release APK (unsigned, ready for your own release keystore):
```powershell
.\scripts\build-android.ps1 -Release
```
Output APK:
- `src-tauri\gen\android\app\build\outputs\apk\universal\debug\app-universal-debug.apk`

### Run Unit Tests
```powershell
cargo test --manifest-path src-tauri\Cargo.toml
```
