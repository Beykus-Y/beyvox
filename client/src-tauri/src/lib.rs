mod audio;
mod vst;

#[cfg(debug_assertions)]
use tauri::Manager;

#[cfg(target_os = "windows")]
fn grant_media_permissions(app: &tauri::App) {
    use webview2_com::Microsoft::Web::WebView2::Win32::*;
    use windows::Win32::Foundation::BOOL;
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.with_webview(|webview| {
            unsafe {
                let core = webview.inner();
                if let Ok(handler) = webview2_com::PermissionRequestedEventHandler::create(
                    Box::new(|_, args: Option<ICoreWebView2PermissionRequestedEventArgs>| {
                        if let Some(args) = args {
                            let _ = args.SetState(COREWEBVIEW2_PERMISSION_STATE_ALLOW);
                            let _ = args.put_Handled(BOOL(1));
                        }
                        Ok(())
                    }),
                ) {
                    let _ = core.add_PermissionRequested(&handler);
                }
            }
        });
    }
}

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
            #[cfg(target_os = "windows")]
            grant_media_permissions(app);
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
