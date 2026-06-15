use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VstPluginInfo {
    pub path: String,
    pub name: String,
    pub vendor: String,
    pub version: String,
    pub num_inputs: i32,
    pub num_outputs: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VstPluginState {
    pub path: String,
    pub enabled: bool,
    pub parameters: HashMap<String, f32>,
}

/// Загружает информацию о VST плагине
#[tauri::command]
pub fn load_vst_info(path: String) -> Result<VstPluginInfo, String> {
    let ext = std::path::Path::new(&path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let name = std::path::Path::new(&path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string();

    match ext.as_str() {
        "dll" | "so" | "dylib" | "vst3" => Ok(VstPluginInfo {
            path,
            name,
            vendor: "Unknown".into(),
            version: "0".into(),
            num_inputs: 2,
            num_outputs: 2,
        }),
        _ => Err(format!("unsupported plugin format: .{ext}")),
    }
}

/// Открывает нативное GUI окно плагина (заглушка — реализуем через vst3-sys)
#[tauri::command]
pub fn open_vst_gui(path: String) -> Result<(), String> {
    eprintln!("open VST GUI: {path}");
    Ok(())
}
