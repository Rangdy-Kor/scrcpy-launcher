use std::path::Path;

/// Presentation only. The discovered PathBuf is never modified for execution.
pub(crate) fn display_path(path: &Path) -> String {
    #[cfg(windows)]
    {
        use std::ffi::OsString;
        use std::path::{Component, PathBuf, Prefix};
        let mut components = path.components();
        let base = match components.next() {
            Some(Component::Prefix(prefix)) => match prefix.kind() {
                Prefix::VerbatimDisk(drive) => Some(PathBuf::from(format!("{}:", drive as char))),
                Prefix::VerbatimUNC(server, share) => {
                    let mut base = OsString::from(r"\\");
                    base.push(server);
                    base.push(r"\");
                    base.push(share);
                    Some(PathBuf::from(base))
                }
                // Volume GUIDs and device namespaces cannot be rewritten safely.
                _ => None,
            },
            _ => None,
        };
        if let Some(mut display) = base {
            for component in components {
                display.push(component.as_os_str());
            }
            return display.to_string_lossy().into_owned();
        }
    }
    path.to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ordinary_path_unchanged() {
        assert_eq!(
            display_path(Path::new(r"C:\Users\사용자\scrcpy.exe")),
            r"C:\Users\사용자\scrcpy.exe"
        );
        assert_eq!(
            display_path(Path::new("/usr/local/bin/scrcpy")),
            "/usr/local/bin/scrcpy"
        );
    }
    #[cfg(windows)]
    #[test]
    fn verbatim_disk_display_preserves_execution_path() {
        let executable = Path::new(r"\\?\C:\Program Files\scrcpy\scrcpy.exe").to_path_buf();
        assert_eq!(
            display_path(&executable),
            r"C:\Program Files\scrcpy\scrcpy.exe"
        );
        assert_eq!(
            executable.as_os_str(),
            r"\\?\C:\Program Files\scrcpy\scrcpy.exe"
        );
    }
    #[cfg(windows)]
    #[test]
    fn verbatim_unc_is_displayed_as_unc() {
        assert_eq!(
            display_path(Path::new(r"\\?\UNC\server\share\folder\scrcpy.exe")),
            r"\\server\share\folder\scrcpy.exe"
        );
        assert_eq!(
            display_path(Path::new(r"\\server\share\folder\scrcpy.exe")),
            r"\\server\share\folder\scrcpy.exe"
        );
    }
    #[cfg(windows)]
    #[test]
    fn other_windows_namespaces_are_not_rewritten() {
        for path in [r"\\?\Volume{1234}\scrcpy.exe", r"\\.\device"] {
            assert_eq!(display_path(Path::new(path)), path);
        }
    }
    #[cfg(not(windows))]
    #[test]
    fn windows_looking_text_is_untouched_on_other_platforms() {
        assert_eq!(
            display_path(Path::new(r"\\?\C:\scrcpy.exe")),
            r"\\?\C:\scrcpy.exe"
        );
    }
}
