mod audio;
mod vst;

#[cfg(debug_assertions)]
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            let _ = app;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            audio::list_input_devices,
            audio::list_output_devices,
            audio::default_input_device,
            audio::default_output_device,
            vst::load_vst_info,
            vst::open_vst_gui,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
