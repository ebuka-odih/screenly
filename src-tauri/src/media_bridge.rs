use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CameraDevice {
    pub id: String,
    pub name: String,
    pub is_continuity: bool,
    pub resolution: Option<(u32, u32)>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScreenSource {
    pub id: String,
    pub name: String,
    pub display_id: u32,
    pub is_window: bool,
}

pub fn list_native_cameras() -> Vec<CameraDevice> {
    // This will eventually call into AVFoundation
    // On macOS, Continuity Cameras appear in AVCaptureDevice.devices(for: .video)
    vec![
        CameraDevice {
            id: "built_in".to_string(),
            name: "FaceTime HD Camera".to_string(),
            is_continuity: false,
            resolution: Some((1280, 720)),
        },
        CameraDevice {
            id: "iphone_continuity".to_string(),
            name: "iPhone 15 Pro Camera (Continuity)".to_string(),
            is_continuity: true,
            resolution: Some((1920, 1080)),
        },
    ]
}

pub fn list_screen_sources() -> Vec<ScreenSource> {
    // This will eventually use ScreenCaptureKit
    vec![
        ScreenSource {
            id: "main_display".to_string(),
            name: "Main Display".to_string(),
            display_id: 1,
            is_window: false,
        }
    ]
}
