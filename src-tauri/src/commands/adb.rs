use crate::{
    config::LauncherError,
    process::adb::{self, AdbStatus},
};

#[tauri::command]
pub async fn get_adb_devices() -> Result<AdbStatus, LauncherError> {
    tauri::async_runtime::spawn_blocking(adb::status)
        .await
        .map_err(|error| {
            LauncherError::new("internal_error", "Could not inspect ADB devices.")
                .with_details(error.to_string())
        })
}
