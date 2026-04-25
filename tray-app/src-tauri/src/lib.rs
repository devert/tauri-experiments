use serde::{Deserialize, Serialize};
use std::fs;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager,
};

#[derive(Debug, Serialize, Deserialize)]
struct Settings {
    username: String,
    theme: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            username: String::from("guest"),
            theme: String::from("light"),
        }
    }
}

#[tauri::command]
fn load_settings(app: tauri::AppHandle) -> Result<Settings, String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;

    let path = data_dir.join("settings.json");

    if !path.exists() {
        return Ok(Settings::default());
    }

    let contents =
        fs::read_to_string(&path).map_err(|e| format!("Failed to read file: {}", e))?;

    serde_json::from_str(&contents).map_err(|e| format!("Failed to parse JSON: {}", e))
}

#[tauri::command]
fn save_settings(app: tauri::AppHandle, settings: Settings) -> Result<(), String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;

    fs::create_dir_all(&data_dir).map_err(|e| format!("Failed to create data dir: {}", e))?;

    let path = data_dir.join("settings.json");

    let contents =
        serde_json::to_string_pretty(&settings).map_err(|e| format!("Failed to serialize: {}", e))?;

    fs::write(&path, contents).map_err(|e| format!("Failed to write file: {}", e))
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, load_settings, save_settings])
        // When the user closes the window, hide it instead of quitting so the
        // app remains accessible via the tray icon.
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                window.hide().unwrap();
                api.prevent_close();
            }
        })
        .setup(|app| {
            // On macOS, hide the dock icon so only the tray icon is visible.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let show = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;

            TrayIconBuilder::new()
                .icon(tauri::include_image!("icons/tray-icon.png"))
                .tooltip("Tray App")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            window.show().unwrap();
                            window.set_focus().unwrap();
                        }
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
