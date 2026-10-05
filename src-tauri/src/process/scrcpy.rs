use crate::config::{arguments::build_arguments, LauncherError, ScrcpyConfig};
use serde::Serialize;
use std::{
    env,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScrcpyStatus {
    pub installed: bool,
    pub executable: Option<String>,
    pub version: Option<String>,
    pub error: Option<LauncherError>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandPreview {
    pub executable: String,
    pub arguments: Vec<String>,
    pub display: String,
}

/// Search PATH for a native executable only (never .bat/.cmd or a shell).
fn find_executable_in(path: &std::ffi::OsStr) -> Option<PathBuf> {
    let name = if cfg!(windows) {
        "scrcpy.exe"
    } else {
        "scrcpy"
    };
    env::split_paths(path)
        .filter(|dir| !dir.as_os_str().is_empty())
        .find_map(|dir| {
            let candidate = dir.join(name);
            if !candidate.is_file() {
                return None;
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if candidate.metadata().ok()?.permissions().mode() & 0o111 == 0 {
                    return None;
                }
            }
            candidate.canonicalize().ok()
        })
}

fn find_executable() -> Option<PathBuf> {
    env::var_os("PATH").and_then(|path| find_executable_in(&path))
}

fn command(executable: &Path) -> Command {
    let mut command = Command::new(executable);
    command.stdin(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000); // CREATE_NO_WINDOW; SDL window still works.
    }
    command
}

pub fn status() -> ScrcpyStatus {
    let Some(executable) = find_executable() else {
        return ScrcpyStatus {
            installed: false,
            executable: None,
            version: None,
            error: None,
        };
    };
    let mut status = ScrcpyStatus {
        installed: true,
        executable: Some(executable.to_string_lossy().into()),
        version: None,
        error: None,
    };
    match command(&executable).arg("--version").output() {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            status.version = stdout
                .lines()
                .chain(stderr.lines())
                .find(|line| line.starts_with("scrcpy "))
                .map(str::to_owned);
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
    let label = if executable.chars().any(char::is_whitespace) {
        format!("\"{executable}\"")
    } else {
        executable.clone()
    };
    let display = std::iter::once(label)
        .chain(arguments.iter().cloned())
        .collect::<Vec<_>>()
        .join(" ");
    CommandPreview {
        executable,
        arguments,
        display,
    }
}

pub fn preview(config: &ScrcpyConfig) -> Result<CommandPreview, LauncherError> {
    let arguments = build_arguments(config)?;
    let executable = find_executable()
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
    let executable = find_executable().ok_or_else(|| {
        LauncherError::new(
            "not_installed",
            "scrcpy was not found in PATH. Install official scrcpy and restart the launcher.",
        )
    })?;
    let child = command(&executable)
        .args(&arguments)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| {
            LauncherError::new("spawn_failed", "Could not start scrcpy.")
                .with_details(error.to_string())
        })?;
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
        assert!(find_executable_in(std::ffi::OsStr::new("")).is_none());
    }
}
