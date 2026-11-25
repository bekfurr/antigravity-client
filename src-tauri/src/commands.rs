use crate::config::{
    normalize_antigravity_url, save_config, slugify_id, AppConfig, Bookmark, Profile,
    DEFAULT_ANTIGRAVITY_URL, DEFAULT_PROFILE_ID,
};
use serde::Serialize;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_opener::OpenerExt;

pub struct AppState {
    pub config: Mutex<AppConfig>,
}

#[derive(Debug, Serialize)]
pub struct ClientSnapshot {
    pub config: AppConfig,
    pub platform: &'static str,
    pub supports_profiles: bool,
    pub supports_tray: bool,
    pub current_url: String,
    pub data_directory: Option<String>,
}

#[tauri::command]
pub fn get_snapshot(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<ClientSnapshot, String> {
    let config = state
        .config
        .lock()
        .map_err(|_| "Failed to lock application state".to_string())?
        .clone();

    let current_url = app
        .get_webview_window("main")
        .and_then(|w| w.url().ok())
        .map(|u| u.to_string())
        .unwrap_or_else(|| config.default_url.clone());

    #[cfg(desktop)]
    let data_directory = crate::config::profile_data_dir(&app, &config.active_profile)
        .ok()
        .map(|p| p.display().to_string());
    #[cfg(not(desktop))]
    let data_directory = None;

    Ok(ClientSnapshot {
        config,
        platform: std::env::consts::OS,
        supports_profiles: cfg!(desktop),
        supports_tray: cfg!(desktop),
        current_url,
        data_directory,
    })
}

#[tauri::command]
pub fn navigate_main(app: AppHandle, url: String) -> Result<String, String> {
    let parsed = normalize_antigravity_url(&url)?;
    let main = app
        .get_webview_window("main")
        .ok_or_else(|| "Main Antigravity window is not available".to_string())?;
    main.navigate(parsed.clone())
        .map_err(|e| format!("Navigation failed: {e}"))?;
    #[cfg(desktop)]
    {
        let _ = main.unminimize();
        let _ = main.show();
        let _ = main.set_focus();
    }
    Ok(parsed.to_string())
}

#[tauri::command]
pub fn save_default_url(
    app: AppHandle,
    state: State<'_, AppState>,
    url: String,
) -> Result<AppConfig, String> {
    let parsed = normalize_antigravity_url(&url)?;
    let mut cfg = state
        .config
        .lock()
        .map_err(|_| "Failed to lock application state".to_string())?;
    cfg.default_url = parsed.to_string();
    save_config(&app, &cfg)?;
    Ok(cfg.clone())
}

#[tauri::command]
pub fn add_bookmark(
    app: AppHandle,
    state: State<'_, AppState>,
    name: String,
    url: String,
) -> Result<AppConfig, String> {
    let clean_name = name.trim();
    if clean_name.is_empty() {
        return Err("Bookmark name cannot be empty.".to_string());
    }
    let parsed = normalize_antigravity_url(&url)?;
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let id = format!("{}-{stamp}", slugify_id(clean_name));

    let mut cfg = state
        .config
        .lock()
        .map_err(|_| "Failed to lock application state".to_string())?;
    cfg.bookmarks.push(Bookmark {
        id,
        name: clean_name.to_string(),
        url: parsed.to_string(),
    });
    save_config(&app, &cfg)?;
    Ok(cfg.clone())
}

#[tauri::command]
pub fn remove_bookmark(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<AppConfig, String> {
    let mut cfg = state
        .config
        .lock()
        .map_err(|_| "Failed to lock application state".to_string())?;
    cfg.bookmarks.retain(|b| b.id != id);
    save_config(&app, &cfg)?;
    Ok(cfg.clone())
}

#[tauri::command]
pub fn set_close_to_tray(
    app: AppHandle,
    state: State<'_, AppState>,
    enabled: bool,
) -> Result<AppConfig, String> {
    let mut cfg = state
        .config
        .lock()
        .map_err(|_| "Failed to lock application state".to_string())?;
    cfg.close_to_tray = enabled;
    save_config(&app, &cfg)?;
    Ok(cfg.clone())
}

#[tauri::command]
pub fn set_custom_user_agent(
    app: AppHandle,
    state: State<'_, AppState>,
    user_agent: Option<String>,
) -> Result<AppConfig, String> {
    let normalized = user_agent
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let mut cfg = state
        .config
        .lock()
        .map_err(|_| "Failed to lock application state".to_string())?;
    cfg.custom_user_agent = normalized;
    save_config(&app, &cfg)?;
    Ok(cfg.clone())
}

#[tauri::command]
pub fn create_profile(
    app: AppHandle,
    state: State<'_, AppState>,
    name: String,
) -> Result<AppConfig, String> {
    let clean_name = name.trim();
    if clean_name.is_empty() {
        return Err("Profile name cannot be empty.".to_string());
    }

    let mut cfg = state
        .config
        .lock()
        .map_err(|_| "Failed to lock application state".to_string())?;

    let base_id = slugify_id(clean_name);
    let mut id = base_id.clone();
    let mut counter = 2;
    while cfg.profiles.iter().any(|p| p.id == id) {
        id = format!("{base_id}-{counter}");
        counter += 1;
    }

    cfg.profiles.push(Profile {
        id,
        name: clean_name.to_string(),
    });
    save_config(&app, &cfg)?;
    Ok(cfg.clone())
}

#[tauri::command]
pub fn switch_profile(
    app: AppHandle,
    state: State<'_, AppState>,
    profile_id: String,
) -> Result<(), String> {
    {
        let mut cfg = state
            .config
            .lock()
            .map_err(|_| "Failed to lock application state".to_string())?;
        if !cfg.profiles.iter().any(|p| p.id == profile_id) {
            return Err(format!("Profile '{profile_id}' does not exist."));
        }
        cfg.active_profile = profile_id;
        save_config(&app, &cfg)?;
    }

    app.restart();
}

#[tauri::command]
pub fn delete_profile(
    app: AppHandle,
    state: State<'_, AppState>,
    profile_id: String,
) -> Result<AppConfig, String> {
    if profile_id == DEFAULT_PROFILE_ID {
        return Err("The Default profile cannot be deleted.".to_string());
    }

    let updated = {
        let mut cfg = state
            .config
            .lock()
            .map_err(|_| "Failed to lock application state".to_string())?;
        if cfg.active_profile == profile_id {
            return Err("Switch to another profile before deleting the active profile.".to_string());
        }
        cfg.profiles.retain(|p| p.id != profile_id);
        save_config(&app, &cfg)?;
        cfg.clone()
    };

    #[cfg(desktop)]
    if let Ok(dir) = crate::config::profile_data_dir(&app, &profile_id) {
        let _ = std::fs::remove_dir_all(dir);
    }

    Ok(updated)
}

#[tauri::command]
pub fn clear_active_session(app: AppHandle) -> Result<(), String> {
    let main = app
        .get_webview_window("main")
        .ok_or_else(|| "Main Antigravity window is not available".to_string())?;
    main.clear_all_browsing_data()
        .map_err(|e| format!("Failed to clear session data: {e}"))?;
    let home = normalize_antigravity_url(DEFAULT_ANTIGRAVITY_URL)?;
    main.navigate(home)
        .map_err(|e| format!("Failed to reload Antigravity: {e}"))?;
    Ok(())
}

#[tauri::command]
pub fn restart_app(app: AppHandle) {
    app.restart();
}

#[tauri::command]
pub fn open_in_browser(app: AppHandle, url: String) -> Result<(), String> {
    let parsed = normalize_antigravity_url(&url)?;
    app.opener()
        .open_url(parsed.as_str(), None::<&str>)
        .map_err(|e| format!("Failed to open browser: {e}"))
}
