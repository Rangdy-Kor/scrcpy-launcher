use super::{executable_name, find_in_path, native_command, native_executable};
use crate::config::LauncherError;
use serde::Serialize;
use std::{
    env,
    ffi::OsStr,
    net::IpAddr,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AdbConnection {
    Usb,
    TcpIp,
    WirelessDebugging,
    Emulator,
    Unknown,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdbDevice {
    pub serial: String,
    /// Preserve all ADB states; only exact "device" is launchable here.
    pub state: String,
    pub model: Option<String>,
    pub product: Option<String>,
    pub device: Option<String>,
    pub transport_id: Option<u64>,
    pub usb: Option<String>,
    pub connection: AdbConnection,
}

fn connection(serial: &str, usb: Option<&str>) -> AdbConnection {
    if usb.is_some() {
        return AdbConnection::Usb;
    }
    if serial.starts_with("emulator-") {
        return AdbConnection::Emulator;
    }
    if serial.contains("._adb-tls-connect._tcp") {
        return AdbConnection::WirelessDebugging;
    }
    if let Some((host, port)) = serial.rsplit_once(':') {
        if host.trim_matches(['[', ']']).parse::<IpAddr>().is_ok() && port.parse::<u16>().is_ok() {
            return AdbConnection::TcpIp;
        }
    }
    // A serial without an IP/mDNS suffix is not proof of USB connectivity.
    AdbConnection::Unknown
}

/// Pure parser. Preserve separate serials/transports even with identical models.
pub fn parse_devices(output: &str) -> Result<Vec<AdbDevice>, LauncherError> {
    let mut lines = output.lines().map(str::trim);
    if !lines.any(|line| line == "List of devices attached") {
        return Err(LauncherError::new(
            "adb_parse_failed",
            "ADB output has no device-list header.",
        )
        .with_details(output));
    }
    let mut devices = Vec::new();
    for line in lines.filter(|line| !line.is_empty() && !line.starts_with('*')) {
        let mut tokens = line.split_whitespace();
        let serial = tokens.next().unwrap_or_default();
        let state = tokens.next().ok_or_else(|| {
            LauncherError::new("adb_parse_failed", "Malformed ADB device row.").with_details(line)
        })?;
        let mut metadata: Vec<_> = tokens.collect();
        // Linux ADB may emit the multiword state "no permissions (...)".
        let state = if state == "no" && metadata.first() == Some(&"permissions") {
            metadata.remove(0);
            "no permissions"
        } else {
            state
        };
        let value = |key: &str| {
            metadata.iter().find_map(|token| {
                let (name, value) = token.split_once(':')?;
                (name == key && !value.is_empty()).then(|| value.to_owned())
            })
        };
        let usb = value("usb");
        devices.push(AdbDevice {
            serial: serial.to_owned(),
            state: state.to_owned(),
            model: value("model"),
            product: value("product"),
            device: value("device"),
            transport_id: value("transport_id").and_then(|value| value.parse().ok()),
            connection: connection(serial, usb.as_deref()),
            usb,
        });
    }
    Ok(devices)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AdbSource {
    Path,
    ScrcpyDirectory,
}

pub(crate) struct AdbExecutable {
    pub path: PathBuf,
    pub source: AdbSource,
}

fn discover_in(path: Option<&OsStr>, scrcpy: Option<&Path>) -> Option<AdbExecutable> {
    if let Some(executable) = path.and_then(|path| find_in_path("adb", path)) {
        return Some(AdbExecutable {
            path: executable,
            source: AdbSource::Path,
        });
    }
    let candidate = scrcpy?.parent()?.join(executable_name("adb"));
    native_executable(&candidate).map(|path| AdbExecutable {
        path,
        source: AdbSource::ScrcpyDirectory,
    })
}

pub(crate) fn discover(scrcpy: Option<&Path>) -> Result<AdbExecutable, LauncherError> {
    discover_in(env::var_os("PATH").as_deref(), scrcpy).ok_or_else(||
        LauncherError::new("adb_not_installed", "ADB was not found in PATH or beside scrcpy. Install Android platform-tools or use official scrcpy's bundled ADB."))
}

pub(crate) fn query_devices(executable: &Path) -> Result<Vec<AdbDevice>, LauncherError> {
    let output = native_command(executable)
        .args(["devices", "-l"])
        .output()
        .map_err(|error| {
            LauncherError::new("adb_query_failed", "Could not execute adb devices -l.")
                .with_details(error.to_string())
        })?;
    if !output.status.success() {
        return Err(LauncherError::new(
            "adb_query_failed",
            format!("adb devices -l failed ({}).", output.status),
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
    let stdout = String::from_utf8(output.stdout).map_err(|error| {
        LauncherError::new("adb_parse_failed", "ADB output was not valid UTF-8.")
            .with_details(error.to_string())
    })?;
    parse_devices(&stdout)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdbStatus {
    pub installed: bool,
    pub executable: Option<String>,
    pub display_executable: Option<String>,
    pub source: Option<AdbSource>,
    pub devices: Vec<AdbDevice>,
    pub error: Option<LauncherError>,
}

pub fn status() -> AdbStatus {
    let scrcpy = super::find_executable("scrcpy");
    match discover(scrcpy.as_deref()) {
        Ok(executable) => {
            let (devices, error) = match query_devices(&executable.path) {
                Ok(devices) => (devices, None),
                Err(error) => (Vec::new(), Some(error)),
            };
            AdbStatus {
                installed: true,
                executable: Some(executable.path.to_string_lossy().into_owned()),
                display_executable: Some(super::paths::display_path(&executable.path)),
                source: Some(executable.source),
                devices,
                error,
            }
        }
        Err(error) => AdbStatus {
            installed: false,
            executable: None,
            display_executable: None,
            source: None,
            devices: Vec::new(),
            error: Some(error),
        },
    }
}

/// Runtime snapshot validation, separate from config/argument validation.
pub fn validate_selection(
    serial: Option<&str>,
    devices: &[AdbDevice],
) -> Result<(), LauncherError> {
    if let Some(serial) = serial {
        let matches: Vec<_> = devices
            .iter()
            .filter(|device| device.serial == serial)
            .collect();
        let device = matches.first().ok_or_else(|| {
            LauncherError::new(
                "device_not_found",
                "The selected device is no longer in the ADB list. Refresh and select a device.",
            )
        })?;
        if matches.len() > 1 {
            return Err(LauncherError::new("ambiguous_device", "Multiple ADB transports share the selected serial; --serial cannot distinguish them."));
        }
        if device.state != "device" {
            return Err(LauncherError::new(
                "device_unavailable",
                format!(
                    "Selected device is {}. Authorize/reconnect it and refresh.",
                    device.state
                ),
            ));
        }
        return Ok(());
    }
    if !devices.iter().any(|device| device.state == "device") {
        return Err(LauncherError::new(
            "no_usable_devices",
            "No ready ADB device. Connect and authorize a device, then refresh.",
        ));
    }
    // Even one usable device plus an offline row needs --serial: scrcpy would
    // otherwise encounter multiple transports. Do not change the config/preview.
    if devices.len() > 1 {
        return Err(LauncherError::new(
            "device_selection_required",
            "Multiple ADB transports are present. Select one device before launching.",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn list(rows: &str) -> Vec<AdbDevice> {
        parse_devices(&format!("List of devices attached\r\n{rows}\r\n")).unwrap()
    }

    #[test]
    fn single_device_with_metadata() {
        let devices =
            list("ABC\tdevice usb:2-1 product:pixel model:Pixel_9 device:tokay transport_id:7");
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].serial, "ABC");
        assert_eq!(devices[0].state, "device");
        assert_eq!(devices[0].model.as_deref(), Some("Pixel_9"));
        assert_eq!(devices[0].product.as_deref(), Some("pixel"));
        assert_eq!(devices[0].device.as_deref(), Some("tokay"));
        assert_eq!(devices[0].transport_id, Some(7));
        assert_eq!(devices[0].connection, AdbConnection::Usb);
    }
    #[test]
    fn multiple_transports_of_same_model_are_preserved() {
        let devices = list("192.168.1.188:42999 device product:r13sksx model:SM_S731N device:r13s transport_id:4\nadb-example._adb-tls-connect._tcp device product:r13sksx model:SM_S731N device:r13s transport_id:2");
        assert_eq!(devices.len(), 2);
        assert_ne!(devices[0].serial, devices[1].serial);
        assert_eq!(devices[0].connection, AdbConnection::TcpIp);
        assert_eq!(devices[1].connection, AdbConnection::WirelessDebugging);
    }
    #[test]
    fn tcpip_and_ipv6_serials() {
        let devices = list("192.168.0.2:5555 device\n[::1]:5555 device");
        assert!(devices
            .iter()
            .all(|device| device.connection == AdbConnection::TcpIp));
    }
    #[test]
    fn wireless_debugging_serial() {
        let serial = "adb-example-123._adb-tls-connect._tcp";
        let devices = list(&format!("{serial} device"));
        assert_eq!(devices[0].serial, serial);
        assert_eq!(devices[0].connection, AdbConnection::WirelessDebugging);
    }
    #[test]
    fn unauthorized_device() {
        let devices = list("ABC unauthorized transport_id:3");
        assert_eq!(devices[0].state, "unauthorized");
        assert_eq!(
            validate_selection(Some("ABC"), &devices).unwrap_err().code,
            "device_unavailable"
        );
    }
    #[test]
    fn offline_device() {
        let devices = list("ABC offline");
        assert_eq!(devices[0].state, "offline");
        assert_eq!(
            validate_selection(Some("ABC"), &devices).unwrap_err().code,
            "device_unavailable"
        );
    }
    #[test]
    fn missing_metadata_does_not_invent_usb() {
        let devices = list("ABC device\nDEF device model:Pixel transport_id:invalid");
        assert!(
            devices[0].model.is_none()
                && devices[0].product.is_none()
                && devices[0].device.is_none()
                && devices[0].transport_id.is_none()
        );
        assert_eq!(devices[0].connection, AdbConnection::Unknown);
        assert_eq!(devices[1].model.as_deref(), Some("Pixel"));
        assert!(devices[1].transport_id.is_none());
    }
    #[test]
    fn empty_list_and_daemon_messages() {
        assert!(list("").is_empty());
        assert!(parse_devices("* daemon not running; starting now at tcp:5037\n* daemon started successfully\nList of devices attached\n\n").unwrap().is_empty());
    }
    #[test]
    fn other_states_are_preserved_and_unlaunchable() {
        for state in [
            "recovery",
            "sideload",
            "bootloader",
            "authorizing",
            "connecting",
            "future_state",
        ] {
            let devices = list(&format!("ABC {state}"));
            assert_eq!(devices[0].state, state);
            assert!(validate_selection(Some("ABC"), &devices).is_err());
        }
        let devices =
            list("ABC no permissions (user is not in plugdev group) usb:1-2 transport_id:4");
        assert_eq!(devices[0].state, "no permissions");
        assert_eq!(devices[0].usb.as_deref(), Some("1-2"));
    }
    #[test]
    fn malformed_output_is_not_an_empty_success() {
        assert!(parse_devices("unexpected output").is_err());
        assert!(parse_devices("List of devices attached\nABC").is_err());
    }
    #[test]
    fn runtime_selection_validation() {
        assert_eq!(
            validate_selection(None, &list("")).unwrap_err().code,
            "no_usable_devices"
        );
        assert!(validate_selection(None, &list("ABC device")).is_ok());
        let devices = list("ABC device\nDEF device");
        assert_eq!(
            validate_selection(None, &devices).unwrap_err().code,
            "device_selection_required"
        );
        assert!(validate_selection(Some("DEF"), &devices).is_ok());
        assert_eq!(
            validate_selection(Some("gone"), &devices).unwrap_err().code,
            "device_not_found"
        );
        assert_eq!(
            validate_selection(None, &list("ABC device\nDEF offline"))
                .unwrap_err()
                .code,
            "device_selection_required"
        );
        assert_eq!(
            validate_selection(Some("ABC"), &list("ABC device\nABC device"))
                .unwrap_err()
                .code,
            "ambiguous_device"
        );
    }
    #[test]
    fn discovery_prefers_path_then_scrcpy_sibling() {
        let root = env::temp_dir().join(format!(
            "scrcpy-launcher-adb-discovery-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(root.join("path")).unwrap();
        std::fs::create_dir_all(root.join("portable")).unwrap();
        let path_adb = root.join("path").join(executable_name("adb"));
        let sibling_adb = root.join("portable").join(executable_name("adb"));
        for path in [&path_adb, &sibling_adb] {
            std::fs::write(path, b"native executable discovery fixture").unwrap();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
            }
        }
        let scrcpy = root.join("portable").join(executable_name("scrcpy"));
        let path = env::join_paths([root.join("path")]).unwrap();
        let found = discover_in(Some(&path), Some(&scrcpy)).unwrap();
        assert_eq!(found.path, path_adb.canonicalize().unwrap());
        assert_eq!(found.source, AdbSource::Path);
        let found = discover_in(None, Some(&scrcpy)).unwrap();
        assert_eq!(found.path, sibling_adb.canonicalize().unwrap());
        assert_eq!(found.source, AdbSource::ScrcpyDirectory);
        std::fs::remove_file(&sibling_adb).unwrap();
        std::fs::write(root.join("portable").join("adb.cmd"), b"must not run").unwrap();
        assert!(discover_in(None, Some(&scrcpy)).is_none());
        assert!(discover_in(None, None).is_none());
        std::fs::remove_dir_all(root).unwrap();
    }
}
