use crate::{
    config::{LauncherError, ScrcpyConfig},
    process::scrcpy::{self, ConfigPreview, LaunchResult, ScrcpyStatus},
};

#[tauri::command]
pub async fn get_scrcpy_status() -> Result<ScrcpyStatus, LauncherError> {
    tauri::async_runtime::spawn_blocking(scrcpy::status)
        .await
        .map_err(|error| {
            LauncherError::new("internal_error", "Could not inspect scrcpy.")
                .with_details(error.to_string())
        })
}

#[tauri::command]
pub async fn preview_scrcpy(config: ScrcpyConfig) -> Result<ConfigPreview, LauncherError> {
    tauri::async_runtime::spawn_blocking(move || scrcpy::inspect_config(&config))
        .await
        .map_err(|error| {
            LauncherError::new("internal_error", "Could not generate preview.")
                .with_details(error.to_string())
        })
}

#[tauri::command]
pub async fn launch_scrcpy(config: ScrcpyConfig) -> Result<LaunchResult, LauncherError> {
    tauri::async_runtime::spawn_blocking(move || scrcpy::launch(&config))
        .await
        .map_err(|error| {
            LauncherError::new("internal_error", "Could not run scrcpy.")
                .with_details(error.to_string())
        })?
}
