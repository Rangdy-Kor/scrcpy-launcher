use super::version::ScrcpyVersion;
use serde::Serialize;

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CapabilitySupport {
    Supported,
    Unsupported,
    Unknown,
}

/// Central location for version requirements. Not a platform/device capability
/// engine: this only describes CLI availability in known stable versions.
pub fn requires_version(
    version: Option<&ScrcpyVersion>,
    minimum: (u32, u32, u32),
) -> CapabilitySupport {
    match version {
        Some(version) if version.at_least(minimum.0, minimum.1, minimum.2) => {
            CapabilitySupport::Supported
        }
        Some(_) => CapabilitySupport::Unsupported,
        None => CapabilitySupport::Unknown,
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScrcpyCapabilities {
    /// --hwdec was introduced in official v5.0.
    pub hardware_decoding: CapabilitySupport,
    /// Conservative verified baseline for Product UI additions (not their
    /// historical introduction version). Existing Foundation options stay usable.
    pub expanded_options: CapabilitySupport,
}

impl ScrcpyCapabilities {
    pub fn from_version(version: Option<&ScrcpyVersion>) -> Self {
        Self {
            hardware_decoding: requires_version(version, (5, 0, 0)),
            expanded_options: requires_version(version, (4, 1, 0)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scrcpy::version::parse_version;
    #[test]
    fn capability_threshold_and_unknown_version() {
        assert_eq!(
            ScrcpyCapabilities::from_version(parse_version("scrcpy 4.1").as_ref())
                .hardware_decoding,
            CapabilitySupport::Unsupported
        );
        assert_eq!(
            ScrcpyCapabilities::from_version(parse_version("scrcpy 5.0").as_ref())
                .hardware_decoding,
            CapabilitySupport::Supported
        );
        assert_eq!(
            ScrcpyCapabilities::from_version(parse_version("scrcpy 6.0").as_ref())
                .hardware_decoding,
            CapabilitySupport::Supported
        );
        assert_eq!(
            ScrcpyCapabilities::from_version(None).hardware_decoding,
            CapabilitySupport::Unknown
        );
    }
}
