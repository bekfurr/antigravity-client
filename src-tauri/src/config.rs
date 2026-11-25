use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use url::Url;

pub const DEFAULT_ANTIGRAVITY_URL: &str = "https://antigravity.google.com";
pub const DEFAULT_PROFILE_ID: &str = "default";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Profile {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Bookmark {
    pub id: String,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub default_url: String,
    pub active_profile: String,
    pub profiles: Vec<Profile>,
    pub bookmarks: Vec<Bookmark>,
    pub close_to_tray: bool,
    pub zoom_level: f64,
    pub custom_user_agent: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            default_url: DEFAULT_ANTIGRAVITY_URL.to_string(),
            active_profile: DEFAULT_PROFILE_ID.to_string(),
            profiles: vec![Profile {
                id: DEFAULT_PROFILE_ID.to_string(),
                name: "Default".to_string(),
            }],
            bookmarks: Vec::new(),
            close_to_tray: false,
            zoom_level: 1.0,
            custom_user_agent: None,
        }
    }
}

impl AppConfig {
    pub fn sanitize(&mut self) {
        if normalize_antigravity_url(&self.default_url).is_err() {
            self.default_url = DEFAULT_ANTIGRAVITY_URL.to_string();
        }
        if !self.profiles.iter().any(|p| p.id == DEFAULT_PROFILE_ID) {
            self.profiles.insert(
                0,
                Profile {
                    id: DEFAULT_PROFILE_ID.to_string(),
                    name: "Default".to_string(),
                },
            );
        }
        if !self.profiles.iter().any(|p| p.id == self.active_profile) {
            self.active_profile = DEFAULT_PROFILE_ID.to_string();
        }
        if !self.zoom_level.is_finite() {
            self.zoom_level = 1.0;
        } else {
            self.zoom_level = self.zoom_level.clamp(0.5, 2.0);
        }
        if let Some(ref ua) = self.custom_user_agent {
            if ua.trim().is_empty() {
                self.custom_user_agent = None;
            }
        }
    }

    #[cfg(desktop)]
    pub fn active_profile_name(&self) -> &str {
        self.profiles
            .iter()
            .find(|p| p.id == self.active_profile)
            .map(|p| p.name.as_str())
            .unwrap_or("Default")
    }
}

pub fn config_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("Failed to resolve config directory: {e}"))?;
    Ok(dir.join("config.json"))
}

pub fn load_config(app: &AppHandle) -> AppConfig {
    let Ok(path) = config_path(app) else {
        return AppConfig::default();
    };
    let Ok(raw) = fs::read_to_string(&path) else {
        return AppConfig::default();
    };
    let mut cfg: AppConfig = serde_json::from_str(&raw).unwrap_or_default();
    cfg.sanitize();
    cfg
}

pub fn save_config(app: &AppHandle, config: &AppConfig) -> Result<(), String> {
    let path = config_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {e}"))?;
    }
    let json = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize configuration: {e}"))?;
    fs::write(&path, json).map_err(|e| format!("Failed to write configuration: {e}"))
}

#[cfg(desktop)]
pub fn profile_data_dir(app: &AppHandle, profile_id: &str) -> Result<PathBuf, String> {
    let base = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to resolve local app data directory: {e}"))?;
    let safe_id = slugify_id(profile_id);
    let dir = base.join("profiles").join(safe_id);
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create profile directory: {e}"))?;
    Ok(dir)
}

pub fn slugify_id(input: &str) -> String {
    let mut out = String::new();
    let mut prev_dash = false;
    for ch in input.trim().chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            prev_dash = false;
        } else if !prev_dash && !out.is_empty() {
            out.push('-');
            prev_dash = true;
        }
    }
    let trimmed = out.trim_matches('-').to_string();
    if trimmed.is_empty() {
        "profile".to_string()
    } else {
        trimmed
    }
}

/// Accepts full https URLs, bare hostnames (`antigravity.google.com/r/abc`),
/// or instance paths (`r/abc` / `/r/abc`).
pub fn normalize_antigravity_url(input: &str) -> Result<Url, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Url::parse(DEFAULT_ANTIGRAVITY_URL)
            .map_err(|e| format!("Invalid default URL: {e}"));
    }

    let candidate = if trimmed.starts_with("https://") || trimmed.starts_with("http://") {
        trimmed.to_string()
    } else if trimmed.contains(':') {
        trimmed.to_string()
    } else if trimmed.starts_with("/r/") {
        format!("{DEFAULT_ANTIGRAVITY_URL}{trimmed}")
    } else if trimmed.starts_with("r/") {
        format!("{DEFAULT_ANTIGRAVITY_URL}/{trimmed}")
    } else if trimmed.contains('.') {
        format!("https://{trimmed}")
    } else {
        format!("{DEFAULT_ANTIGRAVITY_URL}/r/{trimmed}")
    };

    let parsed = Url::parse(&candidate).map_err(|e| format!("Invalid URL '{trimmed}': {e}"))?;
    match parsed.scheme() {
        "https" | "http" => Ok(parsed),
        other => Err(format!("Unsupported URL scheme '{other}'. Use https://.")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_urls_and_instance_ids() {
        assert_eq!(
            normalize_antigravity_url("").unwrap().as_str(),
            "https://antigravity.google.com/"
        );
        assert_eq!(
            normalize_antigravity_url("https://antigravity.google.com/r/dev-box")
                .unwrap()
                .as_str(),
            "https://antigravity.google.com/r/dev-box"
        );
        assert_eq!(
            normalize_antigravity_url("antigravity.google.com/r/box1")
                .unwrap()
                .as_str(),
            "https://antigravity.google.com/r/box1"
        );
        assert_eq!(
            normalize_antigravity_url("r/box2").unwrap().as_str(),
            "https://antigravity.google.com/r/box2"
        );
        assert_eq!(
            normalize_antigravity_url("my-workstation").unwrap().as_str(),
            "https://antigravity.google.com/r/my-workstation"
        );
        assert!(normalize_antigravity_url("javascript:alert(1)").is_err());
    }

    #[test]
    fn slugifies_profile_names() {
        assert_eq!(slugify_id("Work Account"), "work-account");
        assert_eq!(slugify_id("  Personal #1! "), "personal-1");
        assert_eq!(slugify_id("!!!"), "profile");
    }
}
