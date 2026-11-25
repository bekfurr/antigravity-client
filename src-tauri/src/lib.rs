mod commands;
mod config;
#[cfg(desktop)]
mod desktop;
mod navigation;

use commands::AppState;
#[cfg(desktop)]
use config::DEFAULT_PROFILE_ID;
use config::{load_config, normalize_antigravity_url, DEFAULT_ANTIGRAVITY_URL};
use navigation::{extract_cli_url, is_in_app_url};
use std::sync::Mutex;
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_opener::OpenerExt;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default().plugin(tauri_plugin_opener::init());

    #[cfg(desktop)]
    let builder = builder
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            if let Some(url) = extract_cli_url(&args) {
                if let Some(main) = app.get_webview_window("main") {
                    let _ = main.navigate(url);
                }
            }
            desktop::show_main_window(app);
        }));

    builder
        .invoke_handler(tauri::generate_handler![
            commands::get_snapshot,
            commands::navigate_main,
            commands::save_default_url,
            commands::add_bookmark,
            commands::remove_bookmark,
            commands::set_close_to_tray,
            commands::set_custom_user_agent,
            commands::create_profile,
            commands::switch_profile,
            commands::delete_profile,
            commands::clear_active_session,
            commands::restart_app,
            commands::open_in_browser,
        ])
        .setup(|app| {
            let cfg = load_config(app.handle());
            let cli_args: Vec<String> = std::env::args().collect();
            let startup_url = extract_cli_url(&cli_args)
                .or_else(|| normalize_antigravity_url(&cfg.default_url).ok())
                .unwrap_or_else(|| {
                    normalize_antigravity_url(DEFAULT_ANTIGRAVITY_URL)
                        .expect("default URL must be valid")
                });

            #[cfg(desktop)]
            let window_title = if cfg.active_profile == DEFAULT_PROFILE_ID {
                "Antigravity Client".to_string()
            } else {
                format!("Antigravity Client — {}", cfg.active_profile_name())
            };

            #[cfg(desktop)]
            let saved_zoom = cfg.zoom_level;
            #[cfg(desktop)]
            let profile_id = cfg.active_profile.clone();
            let custom_ua = cfg.custom_user_agent.clone();

            app.manage(AppState {
                config: Mutex::new(cfg),
            });

            let nav_handle = app.handle().clone();
            let mut win_builder = WebviewWindowBuilder::new(
                app,
                "main",
                WebviewUrl::External(startup_url),
            )
            .on_navigation(move |url| {
                if is_in_app_url(url) {
                    true
                } else {
                    let _ = nav_handle.opener().open_url(url.as_str(), None::<&str>);
                    false
                }
            });

            if let Some(ua) = custom_ua {
                win_builder = win_builder.user_agent(&ua);
            }

            #[cfg(desktop)]
            {
                let popup_handle = app.handle().clone();
                win_builder = win_builder
                    .title(&window_title)
                    .inner_size(1280.0, 840.0)
                    .min_inner_size(420.0, 540.0)
                    .center()
                    .resizable(true)
                    .on_new_window(move |url, _features| {
                        if is_in_app_url(&url) {
                            tauri::webview::NewWindowResponse::Allow
                        } else {
                            let _ = popup_handle.opener().open_url(url.as_str(), None::<&str>);
                            tauri::webview::NewWindowResponse::Deny
                        }
                    });

                if let Ok(data_dir) = config::profile_data_dir(app.handle(), &profile_id) {
                    win_builder = win_builder.data_directory(data_dir);
                }
            }

            let main_window = win_builder.build()?;

            #[cfg(desktop)]
            {
                if (saved_zoom - 1.0).abs() > f64::EPSILON {
                    let _ = main_window.set_zoom(saved_zoom);
                }

                let menu = desktop::build_app_menu(app.handle())?;
                app.set_menu(menu)?;
                app.on_menu_event(|app_handle, event| {
                    desktop::handle_menu_action(app_handle, event.id().as_ref());
                });

                let _ = desktop::setup_tray(app.handle());

                let close_handle = app.handle().clone();
                let main_for_close = main_window.clone();
                main_window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        let hide_to_tray = close_handle
                            .try_state::<AppState>()
                            .and_then(|s| s.config.lock().ok().map(|c| c.close_to_tray))
                            .unwrap_or(false);
                        if hide_to_tray {
                            api.prevent_close();
                            let _ = main_for_close.hide();
                        }
                    }
                });
            }

            #[cfg(not(desktop))]
            let _ = main_window;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Antigravity Client");
}
