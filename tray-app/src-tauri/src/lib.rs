use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Child;
use std::sync::Mutex;
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

struct ProcessState {
    child: Option<Child>,
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

    let contents = fs::read_to_string(&path).map_err(|e| format!("Failed to read file: {}", e))?;

    serde_json::from_str(&contents).map_err(|e| format!("Failed to parse JSON: {}", e))
}

#[tauri::command]
fn save_settings(app: tauri::AppHandle, settings: Settings) -> Result<(), String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;

    fs::create_dir_all(&data_dir).map_err(|e| format!("Failed to create data dir: {}", e))?;

    let path = data_dir.join("settings.json");

    let contents = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize: {}", e))?;

    fs::write(&path, contents).map_err(|e| format!("Failed to write file: {}", e))
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn spawn_process(state: tauri::State<'_, Mutex<ProcessState>>) -> Result<String, String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;

    if state.child.is_some() {
        return Err("A process is already running.".into());
    }

    let child = std::process::Command::new("ping")
        .arg("localhost")
        .spawn()
        .map_err(|e| format!("Failed to spawn: {}", e))?;

    let pid = child.id();
    state.child = Some(child);

    Ok(format!("Spawned process with PID {}", pid))
}

#[tauri::command]
fn process_status(state: tauri::State<'_, Mutex<ProcessState>>) -> Result<String, String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;

    match state.child.as_mut() {
        None => Ok("No process".into()),
        Some(child) => match child.try_wait().map_err(|e| e.to_string())? {
            Some(status) => Ok(format!("Exited with: {}", status)),
            None => Ok(format!("Running (PID {})", child.id())),
        },
    }
}

#[tauri::command]
fn kill_process(state: tauri::State<'_, Mutex<ProcessState>>) -> Result<String, String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;

    match state.child.take() {
        None => Err("No process to kill.".into()),
        Some(mut child) => {
            child.kill().map_err(|e| format!("Failed to kill: {}", e))?;
            child.wait().map_err(|e| e.to_string())?;
            Ok("Process killed.".into())
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Mutex::new(ProcessState { child: None }))
        .invoke_handler(tauri::generate_handler![
            greet,
            load_settings,
            save_settings,
            spawn_process,
            process_status,
            kill_process,
        ])
        .on_window_event(|window, event| {
            match event {
                // Hide the window instead of quitting so the app remains
                // accessible via the tray icon.
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    window.hide().unwrap();
                    api.prevent_close();
                }
                // Kill any spawned child process when the app shuts down.
                // This covers exit paths beyond the tray menu's "Quit" button
                // (e.g. Cmd+Q, force quit, SIGTERM).
                tauri::WindowEvent::Destroyed => {
                    let state = window.state::<Mutex<ProcessState>>();
                    if let Ok(mut guard) = state.lock() {
                        if let Some(mut child) = guard.child.take() {
                            let _ = child.kill();
                            let _ = child.wait();
                        }
                    };
                }
                _ => {}
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
                    // Kill any spawned child process before exiting.
                    // The Destroyed window event also handles this for other
                    // exit paths, but the tray quit fires first and avoids
                    // relying on window teardown order.
                    "quit" => {
                        let state = app.state::<Mutex<ProcessState>>();
                        if let Ok(mut guard) = state.lock() {
                            if let Some(mut child) = guard.child.take() {
                                let _ = child.kill();
                                let _ = child.wait();
                            }
                        }
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
