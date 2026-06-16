use cpal::traits::{DeviceTrait, HostTrait};
use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
pub struct AudioDevice {
    pub name: String,
    pub id: String,
}

#[tauri::command]
pub fn list_input_devices() -> Vec<AudioDevice> {
    let host = cpal::default_host();
    host.input_devices()
        .map(|devices| {
            devices
                .filter_map(|d| {
                    let name = d.name().ok()?;
                    Some(AudioDevice { id: name.clone(), name })
                })
                .collect()
        })
        .unwrap_or_default()
}

#[tauri::command]
pub fn list_output_devices() -> Vec<AudioDevice> {
    let host = cpal::default_host();
    host.output_devices()
        .map(|devices| {
            devices
                .filter_map(|d| {
                    let name = d.name().ok()?;
                    Some(AudioDevice { id: name.clone(), name })
                })
                .collect()
        })
        .unwrap_or_default()
}

#[tauri::command]
pub fn default_input_device() -> Option<String> {
    cpal::default_host().default_input_device().and_then(|d| d.name().ok())
}

#[tauri::command]
pub fn default_output_device() -> Option<String> {
    cpal::default_host().default_output_device().and_then(|d| d.name().ok())
}
