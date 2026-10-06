pub mod adb;
pub(crate) mod paths;
pub mod scrcpy;

use std::{
    env,
    ffi::OsStr,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

pub(crate) fn executable_name(name: &str) -> String {
    if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_owned()
    }
}

pub(crate) fn native_executable(path: &Path) -> Option<PathBuf> {
    if !path.is_file() {
        return None;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if path.metadata().ok()?.permissions().mode() & 0o111 == 0 {
            return None;
        }
    }
    path.canonicalize().ok()
}

/// Search only the named native binary; never shell scripts/.cmd/.bat.
pub(crate) fn find_in_path(name: &str, path: &OsStr) -> Option<PathBuf> {
    env::split_paths(path)
        .filter(|dir| !dir.as_os_str().is_empty())
        .find_map(|dir| native_executable(&dir.join(executable_name(name))))
}

pub(crate) fn find_executable(name: &str) -> Option<PathBuf> {
    env::var_os("PATH").and_then(|path| find_in_path(name, &path))
}

pub(crate) fn native_command(executable: &Path) -> Command {
    let mut command = Command::new(executable);
    command.stdin(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000); // CREATE_NO_WINDOW; SDL window still works.
    }
    command
}
