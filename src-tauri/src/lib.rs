mod media_bridge;

use std::sync::Mutex;
use tauri::{State, Manager, Emitter};
use serde::Serialize;
use rdev::{listen, Event, EventType};

#[derive(Debug, Serialize, Clone)]
struct ClickEvent {
    x: f64,
    y: f64,
    timestamp: u64,
}

pub struct Engine {
    pub is_recording: Mutex<bool>,
    pub start_time: Mutex<Option<u64>>,
}

#[tauri::command]
fn list_cameras() -> Vec<media_bridge::CameraDevice> {
    media_bridge::list_native_cameras()
}

#[tauri::command]
fn list_screens() -> Vec<media_bridge::ScreenSource> {
    media_bridge::list_screen_sources()
}

#[tauri::command]
fn check_permissions() -> bool {
    true
}

#[tauri::command]
fn start_recording(state: State<Engine>) -> Result<(), String> {
    let mut recording = state.is_recording.lock().unwrap();
    if *recording {
        return Err("Already recording".to_string());
    }
    *recording = true;
    Ok(())
}

#[tauri::command]
fn stop_recording(state: State<Engine>) -> Result<(), String> {
    let mut recording = state.is_recording.lock().unwrap();
    if !*recording {
        return Err("Not recording".to_string());
    }
    *recording = false;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(Engine {
            is_recording: Mutex::new(false),
            start_time: Mutex::new(None),
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            check_permissions,
            list_cameras,
            list_screens,
            start_recording,
            stop_recording
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            
            // Spawn a thread for global mouse event listening
            thread::spawn(move || {
                let mut last_x = 0.0;
                let mut last_y = 0.0;
                let callback = move |event: Event| {
                    match event.event_type {
                        EventType::MouseMove { x, y } => {
                            last_x = x;
                            last_y = y;
                        }
                        EventType::ButtonPress(rdev::Button::Left) => {
                            // Only emit if we are actually recording
                            let state = handle.state::<Engine>();
                            if *state.is_recording.lock().unwrap() {
                                let _ = handle.emit("mouse-click", ClickEvent {
                                    x: last_x,
                                    y: last_y,
                                    timestamp: std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap()
                                        .as_millis() as u64,
                                });
                            }
                        }
                        _ => {}
                    }
                };

                if let Err(error) = listen(callback) {
                    println!("Error listening to mouse events: {:?}", error);
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

use std::thread;
