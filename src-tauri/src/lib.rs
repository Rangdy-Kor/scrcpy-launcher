mod commands;
mod config;
mod process;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::scrcpy::get_scrcpy_status,
            commands::scrcpy::preview_scrcpy,
            commands::scrcpy::launch_scrcpy,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
