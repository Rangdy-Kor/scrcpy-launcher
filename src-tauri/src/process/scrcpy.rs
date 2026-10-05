use super::{adb, find_executable, native_command};
use crate::config::{arguments::build_arguments, LauncherError, ScrcpyConfig};
use crate::scrcpy::{
    capabilities::ScrcpyCapabilities,
    version::{parse_version, ScrcpyVersion},
};
use serde::Serialize;
use std::{
    path::Path,
    process::{Child, Stdio},
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScrcpyStatus {
    pub installed: bool,
    pub executable: Option<String>,
    pub version: Option<ScrcpyVersion>,
    pub version_output: Option<String>,
    pub capabilities: ScrcpyCapabilities,
    pub error: Option<LauncherError>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandPreview {
    pub executable: String,
    pub arguments: Vec<String>,
    pub display: String,
}

pub fn status() -> ScrcpyStatus {
    let Some(executable) = find_executable("scrcpy") else {
        return ScrcpyStatus {
            installed: false,
            executable: None,
            version: None,
            version_output: None,
            capabilities: ScrcpyCapabilities::from_version(None),
            error: None,
        };
    };
    let mut status = ScrcpyStatus {
        installed: true,
        executable: Some(executable.to_string_lossy().into()),
        version: None,
        version_output: None,
        capabilities: ScrcpyCapabilities::from_version(None),
        error: None,
    };
    match native_command(&executable).arg("--version").output() {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let output = format!("{stdout}\n{stderr}");
            status.version = parse_version(&output);
            status.capabilities = ScrcpyCapabilities::from_version(status.version.as_ref());
            status.version_output = Some(output.trim().to_owned());
            if status.version.is_none() {
                status.error = Some(LauncherError::new(
                    "version_failed",
                    "scrcpy returned no recognizable version.",
                ));
            }
        }
        Ok(output) => {
            status.error = Some(
                LauncherError::new("version_failed", "scrcpy --version failed.")
                    .with_details(String::from_utf8_lossy(&output.stderr)),
            )
        }
        Err(error) => {
            status.error = Some(
                LauncherError::new("version_failed", "Could not execute scrcpy --version.")
                    .with_details(error.to_string()),
            )
        }
    }
    status
}

fn make_preview(executable: String, arguments: Vec<String>) -> CommandPreview {
    // Display only: execution uses the original executable and argument vector.
    let label = display_token(&executable);
    let display = std::iter::once(label)
        .chain(arguments.iter().map(|argument| display_token(argument)))
        .collect::<Vec<_>>()
        .join(" ");
    CommandPreview {
        executable,
        arguments,
        display,
    }
}

fn display_token(token: &str) -> String {
    if token
        .chars()
        .any(|c| c.is_whitespace() || matches!(c, '"' | '\'' | ';' | '&' | '|' | '$' | '`'))
    {
        format!("\"{}\"", token.replace('"', "\\\""))
    } else {
        token.to_owned()
    }
}

pub fn preview(config: &ScrcpyConfig) -> Result<CommandPreview, LauncherError> {
    let arguments = build_arguments(config)?;
    let executable = find_executable("scrcpy")
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_else(|| "scrcpy".into());
    Ok(make_preview(executable, arguments))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchResult {
    pub command: CommandPreview,
    pub exit_code: Option<i32>,
}

/// Runs on a blocking worker. Returns when scrcpy closes, so device/ADB and
/// unsupported-option failures are also reported to the frontend.
pub fn launch(config: &ScrcpyConfig) -> Result<LaunchResult, LauncherError> {
    let arguments = build_arguments(config)?;
    let executable = find_executable("scrcpy").ok_or_else(|| {
        LauncherError::new(
            "not_installed",
            "scrcpy was not found in PATH. Install official scrcpy and restart the launcher.",
        )
    })?;
    let adb = adb::discover(Some(&executable))?;
    let devices = adb::query_devices(&adb.path)?;
    adb::validate_selection(config.device.serial.as_deref(), &devices)?;
    let child = spawn_scrcpy(&executable, &adb.path, &arguments)?;
    let output = child.wait_with_output().map_err(|error| {
        LauncherError::new("wait_failed", "Could not collect scrcpy's exit status.")
            .with_details(error.to_string())
    })?;
    if !output.status.success() {
        return Err(LauncherError::new(
            "process_failed",
            format!("scrcpy exited unsuccessfully ({}).", output.status),
        )
        .with_details(
            format!(
                "{}\n{}",
                String::from_utf8_lossy(&output.stderr),
                String::from_utf8_lossy(&output.stdout)
            )
            .trim()
            .to_owned(),
        ));
    }
    Ok(LaunchResult {
        command: make_preview(executable.to_string_lossy().into_owned(), arguments),
        exit_code: output.status.code(),
    })
}

/// A future registry can take ownership of this Child and its pipes without
/// changing version/device validation or the pure argument builder.
fn spawn_scrcpy(
    executable: &Path,
    adb: &Path,
    arguments: &[String],
) -> Result<Child, LauncherError> {
    native_command(executable)
        .args(arguments)
        .env("ADB", adb)
        // Config is the device-selection authority, not an inherited serial.
        .env_remove("ANDROID_SERIAL")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| {
            LauncherError::new("spawn_failed", "Could not start scrcpy.")
                .with_details(error.to_string())
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preview_uses_builder_even_without_installation() {
        let config: ScrcpyConfig = serde_json::from_str(
            r#"{"video":{"codec":"h265","bitrateMbps":20},"input":{"keyboard":"uhid"}}"#,
        )
        .unwrap();
        assert_eq!(
            preview(&config).unwrap().arguments,
            build_arguments(&config).unwrap()
        );
    }
    #[test]
    fn display_quotes_executable_without_changing_arguments() {
        let preview = make_preview(
            "C:/Program Files/scrcpy.exe".into(),
            vec!["--no-audio".into()],
        );
        assert_eq!(
            preview.display,
            "\"C:/Program Files/scrcpy.exe\" --no-audio"
        );
        assert_eq!(preview.arguments, ["--no-audio"]);
    }
    #[test]
    fn missing_path_returns_none() {
        assert!(super::super::find_in_path("scrcpy", std::ffi::OsStr::new("")).is_none());
    }

    #[test]
    fn windows_path_survives_json_without_character_substitution() {
        let path = r"C:\Users\사용자\scrcpy\scrcpy.exe";
        let preview = make_preview(path.to_owned(), vec!["--no-audio".into()]);
        let json = serde_json::to_string(&preview).unwrap();
        let decoded: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded["executable"].as_str().unwrap(), path);
        assert_eq!(
            decoded["display"].as_str().unwrap(),
            format!("{path} --no-audio")
        );
        assert!(decoded["executable"]
            .as_str()
            .unwrap()
            .chars()
            .any(|c| c == '\u{005c}'));
    }

    #[test]
    fn preview_serial_matches_builder_as_one_argument() {
        let config: ScrcpyConfig = serde_json::from_str(
            r#"{"device":{"serial":"adb-example._adb-tls-connect._tcp"},"video":{"codec":"h265"}}"#,
        )
        .unwrap();
        let preview = preview(&config).unwrap();
        assert_eq!(preview.arguments, build_arguments(&config).unwrap());
        assert!(preview
            .display
            .contains("--serial=adb-example._adb-tls-connect._tcp"));
    }

    #[test]
    #[ignore = "requires installed scrcpy and ADB; queries local devices but never launches mirroring"]
    fn live_discovery_smoke() {
        let scrcpy = status();
        println!("scrcpy status: {}", serde_json::to_string(&scrcpy).unwrap());
        assert!(scrcpy.installed && scrcpy.version.is_some() && scrcpy.error.is_none());
        let adb = adb::status();
        println!("adb status: {}", serde_json::to_string(&adb).unwrap());
        assert!(adb.installed && adb.error.is_none());
        for device in &adb.devices {
            if device.state == "device" {
                let config: ScrcpyConfig =
                    serde_json::from_value(serde_json::json!({"device":{"serial":device.serial}}))
                        .unwrap();
                let preview = preview(&config).unwrap();
                assert_eq!(preview.arguments[0], format!("--serial={}", device.serial));
                assert!(
                    adb::validate_selection(config.device.serial.as_deref(), &adb.devices).is_ok()
                );
            }
        }
        if adb.devices.len() > 1 && adb.devices.iter().any(|device| device.state == "device") {
            assert_eq!(
                launch(&ScrcpyConfig::default()).unwrap_err().code,
                "device_selection_required"
            );
        }
    }
}
