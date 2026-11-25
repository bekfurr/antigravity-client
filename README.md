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

