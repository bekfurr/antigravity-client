use crate::commands::AppState;
use crate::config::{normalize_antigravity_url, save_config, DEFAULT_ANTIGRAVITY_URL};
use tauri::menu::{Menu, MenuBuilder, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_opener::OpenerExt;

pub fn build_app_menu(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let settings = MenuItemBuilder::with_id("open_settings", "Settings & Profiles...")
        .accelerator("CmdOrCtrl+,")
        .build(app)?;
    let reload = MenuItemBuilder::with_id("reload", "Reload")
        .accelerator("CmdOrCtrl+R")
        .build(app)?;
    let hard_reload = MenuItemBuilder::with_id("hard_reload", "Hard Reload")
        .accelerator("CmdOrCtrl+Shift+R")
        .build(app)?;
    let open_browser =
        MenuItemBuilder::with_id("open_browser", "Open Current Page in Browser").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "Quit")
        .accelerator("CmdOrCtrl+Q")
        .build(app)?;

    let app_submenu = SubmenuBuilder::new(app, "App")
        .item(&settings)
        .separator()
        .item(&reload)
        .item(&hard_reload)
        .item(&open_browser)
        .separator()
        .item(&quit)
        .build()?;

    let back = MenuItemBuilder::with_id("nav_back", "Back")
        .accelerator("Alt+Left")
        .build(app)?;
    let forward = MenuItemBuilder::with_id("nav_forward", "Forward")
        .accelerator("Alt+Right")
        .build(app)?;
    let home = MenuItemBuilder::with_id("nav_home", "Home (antigravity.google.com)")
        .accelerator("Alt+Home")
        .build(app)?;
    let connect = MenuItemBuilder::with_id("open_connect", "Connect to Instance URL...")
        .accelerator("CmdOrCtrl+L")
        .build(app)?;

    let nav_submenu = SubmenuBuilder::new(app, "Navigate")
        .item(&back)
        .item(&forward)
        .item(&home)
        .separator()
        .item(&connect)
        .build()?;

    let zoom_in = MenuItemBuilder::with_id("zoom_in", "Zoom In")
        .accelerator("CmdOrCtrl+=")
        .build(app)?;
    let zoom_out = MenuItemBuilder::with_id("zoom_out", "Zoom Out")
        .accelerator("CmdOrCtrl+-")
        .build(app)?;
    let zoom_reset = MenuItemBuilder::with_id("zoom_reset", "Actual Size")
        .accelerator("CmdOrCtrl+0")
        .build(app)?;
    let fullscreen = MenuItemBuilder::with_id("toggle_fullscreen", "Toggle Fullscreen")
        .accelerator("F11")
        .build(app)?;

    let view_submenu = SubmenuBuilder::new(app, "View")
        .item(&zoom_in)
        .item(&zoom_out)
        .item(&zoom_reset)
        .separator()
        .item(&fullscreen)
        .build()?;

    let edit_submenu = SubmenuBuilder::new(app, "Edit")
        .item(&PredefinedMenuItem::undo(app, None)?)
        .item(&PredefinedMenuItem::redo(app, None)?)
        .separator()
        .item(&PredefinedMenuItem::cut(app, None)?)
        .item(&PredefinedMenuItem::copy(app, None)?)
        .item(&PredefinedMenuItem::paste(app, None)?)
        .item(&PredefinedMenuItem::select_all(app, None)?)
        .build()?;

    let docs = MenuItemBuilder::with_id("open_docs", "Antigravity Documentation").build(app)?;
    let support = MenuItemBuilder::with_id("open_support", "Antigravity Support").build(app)?;

    let help_submenu = SubmenuBuilder::new(app, "Help")
        .item(&docs)
        .item(&support)
        .build()?;

    MenuBuilder::new(app)
        .items(&[
            &app_submenu,
            &edit_submenu,
            &nav_submenu,
            &view_submenu,
            &help_submenu,
        ])
        .build()
}

pub fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    let show = MenuItemBuilder::with_id("tray_show", "Show Antigravity").build(app)?;
    let home = MenuItemBuilder::with_id("nav_home", "Home").build(app)?;
    let reload = MenuItemBuilder::with_id("reload", "Reload").build(app)?;
    let settings = MenuItemBuilder::with_id("open_settings", "Settings & Profiles...").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;

    let tray_menu = MenuBuilder::new(app)
        .items(&[&show, &home, &reload, &settings])
        .separator()
        .item(&quit)
        .build()?;

    let mut builder = TrayIconBuilder::with_id("main-tray")
        .tooltip("Antigravity Client")
        .menu(&tray_menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            handle_menu_action(app, event.id().as_ref());
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                toggle_main_window(tray.app_handle());
            }
        });

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }

    let _tray = builder.build(app)?;
    Ok(())
}

pub fn handle_menu_action(app: &AppHandle, id: &str) {
    match id {
        "tray_show" => show_main_window(app),
        "open_settings" => {
            let _ = open_settings_window(app, None);
        }
        "open_connect" => {
            let _ = open_settings_window(app, Some("connect"));
        }
        "reload" => {
            if let Some(win) = app.get_webview_window("main") {
                let _ = win.reload();
            }
        }
        "hard_reload" => {
            if let Some(win) = app.get_webview_window("main") {
                let _ = win.eval("window.location.reload();");
            }
        }
        "nav_back" => {
            if let Some(win) = app.get_webview_window("main") {
                let _ = win.eval("window.history.back();");
            }
        }
        "nav_forward" => {
            if let Some(win) = app.get_webview_window("main") {
                let _ = win.eval("window.history.forward();");
            }
        }
        "nav_home" => {
            let target = app
                .try_state::<AppState>()
                .and_then(|s| s.config.lock().ok().map(|c| c.default_url.clone()))
                .unwrap_or_else(|| DEFAULT_ANTIGRAVITY_URL.to_string());
            if let Ok(url) = normalize_antigravity_url(&target) {
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.navigate(url);
                }
            }
            show_main_window(app);
        }
        "zoom_in" => adjust_zoom(app, 0.1),
        "zoom_out" => adjust_zoom(app, -0.1),
        "zoom_reset" => set_zoom(app, 1.0),
        "toggle_fullscreen" => {
            if let Some(win) = app.get_webview_window("main") {
                if let Ok(is_fs) = win.is_fullscreen() {
                    let _ = win.set_fullscreen(!is_fs);
                }
            }
        }
        "open_browser" => {
            let url = app
                .get_webview_window("main")
                .and_then(|w| w.url().ok())
                .map(|u| u.to_string())
                .unwrap_or_else(|| DEFAULT_ANTIGRAVITY_URL.to_string());
            let _ = app.opener().open_url(&url, None::<&str>);
        }
        "open_docs" => {
            let _ = app
                .opener()
                .open_url("https://antigravity.google/docs/getting-started", None::<&str>);
        }
        "open_support" => {
            let _ = app
                .opener()
                .open_url("https://antigravity.google/support", None::<&str>);
        }
        "quit" => {
            app.exit(0);
        }
        _ => {}
    }
}

pub fn show_main_window(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.unminimize();
        let _ = win.show();
        let _ = win.set_focus();
    }
}

pub fn toggle_main_window(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let visible = win.is_visible().unwrap_or(true);
        let minimized = win.is_minimized().unwrap_or(false);
        if visible && !minimized {
            let _ = win.hide();
        } else {
            let _ = win.unminimize();
            let _ = win.show();
            let _ = win.set_focus();
        }
    }
}

pub fn open_settings_window(app: &AppHandle, section: Option<&str>) -> tauri::Result<()> {
    if let Some(existing) = app.get_webview_window("settings") {
        let _ = existing.unminimize();
        let _ = existing.show();
        let _ = existing.set_focus();
        if let Some(sec) = section {
            let _ = existing.eval(&format!("window.focusSection && window.focusSection('{sec}');"));
        }
        return Ok(());
    }

    let route = match section {
        Some(sec) => format!("settings.html#{sec}"),
        None => "settings.html".to_string(),
    };

    WebviewWindowBuilder::new(app, "settings", WebviewUrl::App(route.into()))
        .title("Antigravity Client — Settings & Profiles")
        .inner_size(600.0, 680.0)
        .min_inner_size(460.0, 520.0)
        .resizable(true)
        .center()
        .build()?;

    Ok(())
}

fn adjust_zoom(app: &AppHandle, delta: f64) {
    let next = app
        .try_state::<AppState>()
        .and_then(|s| s.config.lock().ok().map(|c| c.zoom_level + delta))
        .unwrap_or(1.0);
    set_zoom(app, next);
}

fn set_zoom(app: &AppHandle, zoom: f64) {
    let clamped = (zoom.clamp(0.5, 2.0) * 100.0).round() / 100.0;
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.set_zoom(clamped);
    }
    if let Some(state) = app.try_state::<AppState>() {
        if let Ok(mut cfg) = state.config.lock() {
            cfg.zoom_level = clamped;
            let _ = save_config(app, &cfg);
        }
    }
}
