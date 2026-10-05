use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ScrcpyVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    /// Original version token, e.g. "4.1" (not dependency versions).
    pub raw: String,
}

impl ScrcpyVersion {
    pub fn at_least(&self, major: u32, minor: u32, patch: u32) -> bool {
        (self.major, self.minor, self.patch) >= (major, minor, patch)
    }
}

/// Official banner: `scrcpy VERSION <https://github.com/Genymobile/scrcpy>`.
/// Unknown/development suffixes remain unknown instead of assuming stable support.
pub fn parse_version(output: &str) -> Option<ScrcpyVersion> {
    for line in output.lines() {
        let mut tokens = line.split_whitespace();
        if tokens.next() != Some("scrcpy") {
            continue;
        }
        let raw = tokens.next()?;
        let parts: Vec<_> = raw.split('.').collect();
        if !(2..=3).contains(&parts.len())
            || parts
                .iter()
                .any(|part| part.is_empty() || !part.bytes().all(|c| c.is_ascii_digit()))
        {
            return None;
        }
        return Some(ScrcpyVersion {
            major: parts[0].parse().ok()?,
            minor: parts[1].parse().ok()?,
            patch: if parts.len() == 3 {
                parts[2].parse().ok()?
            } else {
                0
            },
            raw: raw.to_owned(),
        });
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_4_1_banner_with_dependency_versions() {
        let version = parse_version("scrcpy 4.1 <https://github.com/Genymobile/scrcpy>\r\n\r\nDependencies (compiled / linked):\r\n - SDL: 3.4.12 / 3.4.12\r\n").unwrap();
        assert_eq!((version.major, version.minor, version.patch), (4, 1, 0));
        assert_eq!(version.raw, "4.1");
    }
    #[test]
    fn parses_5_0_without_patch() {
        let version =
            parse_version("\n  scrcpy 5.0 <https://github.com/Genymobile/scrcpy>\n").unwrap();
        assert_eq!((version.major, version.minor, version.patch), (5, 0, 0));
    }
    #[test]
    fn parses_patch_version() {
        let version = parse_version("scrcpy 3.3.4 <https://github.com/Genymobile/scrcpy>").unwrap();
        assert_eq!((version.major, version.minor, version.patch), (3, 3, 4));
    }
    #[test]
    fn unexpected_output_is_unknown() {
        for output in [
            "",
            "SDL 5.0",
            "scrcpy",
            "scrcpy future",
            "scrcpy 5",
            "scrcpy 5.0.0.1",
            "scrcpy 5..0",
            "scrcpy 5.0-beta",
            "scrcpy 4294967296.0",
        ] {
            assert!(parse_version(output).is_none(), "{output}");
        }
    }
    #[test]
    fn numeric_version_comparison() {
        let version = parse_version("scrcpy 5.0").unwrap();
        assert!(version.at_least(5, 0, 0));
        assert!(version.at_least(4, 99, 99));
        assert!(!version.at_least(5, 0, 1));
        assert!(!version.at_least(5, 1, 0));
        assert!(parse_version("scrcpy 4.10").unwrap().at_least(4, 2, 0));
        assert!(parse_version("scrcpy 5.0.1").unwrap().at_least(5, 0, 1));
    }
}
