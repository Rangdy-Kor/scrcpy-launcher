use super::{
    AudioCodec, AudioSource, HardwareDecoding, KeyboardMode, LauncherError, MouseMode, ScrcpyConfig,
};
use crate::scrcpy::capabilities::{CapabilitySupport, ScrcpyCapabilities};
use serde::Serialize;

#[derive(Debug, Clone, Copy)]
pub enum HostPlatform {
    Windows,
    Linux,
    Macos,
    Other,
}
impl HostPlatform {
    pub fn current() -> Self {
        if cfg!(windows) {
            Self::Windows
        } else if cfg!(target_os = "linux") {
            Self::Linux
        } else if cfg!(target_os = "macos") {
            Self::Macos
        } else {
            Self::Other
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigAvailability {
    pub video: bool,
    pub audio: bool,
    pub control: bool,
    pub window: bool,
    pub audio_encoding: bool,
    pub audio_bitrate: bool,
    pub audio_duplication: bool,
    pub sdk_keyboard: bool,
    pub sdk_mouse: bool,
    pub device_power: bool,
    pub expanded_options: bool,
    pub hardware_decoding: bool,
    pub hardware_decoders: Vec<HardwareDecoding>,
    pub aoa_input: bool,
}

/// One domain definition used by argument emission and UI availability. Values
/// under an inactive parent are retained in config, but emit no arguments.
pub fn dependencies(config: &ScrcpyConfig) -> ConfigAvailability {
    let video = config.video.enabled != Some(false);
    let audio = config.audio.enabled != Some(false);
    let control = config.input.enabled != Some(false);
    let audio_encoding = audio && !matches!(config.audio.codec, Some(AudioCodec::Raw));
    ConfigAvailability {
        video,
        audio,
        control,
        window: video,
        audio_encoding,
        audio_bitrate: audio_encoding && !matches!(config.audio.codec, Some(AudioCodec::Flac)),
        audio_duplication: audio
            && matches!(config.audio.source, None | Some(AudioSource::Playback)),
        sdk_keyboard: control && matches!(config.input.keyboard, None | Some(KeyboardMode::Sdk)),
        sdk_mouse: control && matches!(config.input.mouse, None | Some(MouseMode::Sdk)),
        device_power: control,
        expanded_options: true,
        hardware_decoding: video,
        hardware_decoders: vec![],
        aoa_input: true,
    }
}

pub fn availability(
    config: &ScrcpyConfig,
    capabilities: &ScrcpyCapabilities,
    platform: HostPlatform,
) -> ConfigAvailability {
    let mut result = dependencies(config);
    result.expanded_options = capabilities.expanded_options == CapabilitySupport::Supported;
    // This launcher mirrors through ADB; Windows AOA requires the excluded OTG workflow.
    result.aoa_input = !matches!(platform, HostPlatform::Windows);
    result.hardware_decoding &= capabilities.hardware_decoding == CapabilitySupport::Supported;
    result.hardware_decoders = vec![HardwareDecoding::Auto, HardwareDecoding::Disabled];
    match platform {
        HostPlatform::Windows => result.hardware_decoders.push(HardwareDecoding::D3d11va),
        HostPlatform::Linux => result.hardware_decoders.push(HardwareDecoding::Vaapi),
        HostPlatform::Macos => result
            .hardware_decoders
            .push(HardwareDecoding::Videotoolbox),
        HostPlatform::Other => {}
    }
    result
}

/// Runtime capability checks over the vector produced by the pure builder.
/// Both preview and launch use this function; it never generates arguments.
pub fn validate_runtime_arguments(
    args: &[String],
    capabilities: &ScrcpyCapabilities,
    platform: HostPlatform,
) -> Result<(), LauncherError> {
    for argument in args {
        if matches!(platform, HostPlatform::Windows)
            && matches!(argument.as_str(), "--keyboard=aoa" | "--mouse=aoa")
        {
            return Err(LauncherError::new("unsupported_option", "AOA input on Windows requires OTG mode, which this mirroring launcher does not implement. Use UHID or SDK."));
        }
        let name = argument.split('=').next().unwrap_or(argument);
        if name == "--hwdec" {
            if capabilities.hardware_decoding != CapabilitySupport::Supported {
                return Err(LauncherError::new("unsupported_option", "Hardware decoding settings require a known scrcpy version >= 5.0. Reset Video or choose Default."));
            }
            let valid = match argument.as_str() {
                "--hwdec=auto" | "--hwdec=disabled" => true,
                "--hwdec=d3d11va" => matches!(platform, HostPlatform::Windows),
                "--hwdec=vaapi" => matches!(platform, HostPlatform::Linux),
                "--hwdec=videotoolbox" => matches!(platform, HostPlatform::Macos),
                _ => false,
            };
            if !valid {
                return Err(LauncherError::new(
                    "unsupported_option",
                    "This hardware decoder is not available on the launcher platform.",
                ));
            }
        } else {
            let legacy = matches!(
                name,
                "--serial"
                    | "--video-bit-rate"
                    | "--max-fps"
                    | "--no-audio"
                    | "--keyboard"
                    | "--mouse"
            ) || (name == "--video-codec"
                && !matches!(argument.as_str(), "--video-codec=vp8" | "--video-codec=vp9"));
            if !legacy && capabilities.expanded_options != CapabilitySupport::Supported {
                return Err(LauncherError::new("unsupported_option", "Product UI expanded settings require the verified scrcpy baseline >= 4.1. Reset this category or use Default.").with_details(argument));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::arguments::build_arguments, scrcpy::version::parse_version};
    use serde_json::json;
    fn capabilities(version: &str) -> ScrcpyCapabilities {
        ScrcpyCapabilities::from_version(parse_version(&format!("scrcpy {version}")).as_ref())
    }
    #[test]
    fn installed_4_1_disables_hardware_but_enables_verified_expansion() {
        let a = availability(
            &ScrcpyConfig::default(),
            &capabilities("4.1"),
            HostPlatform::Windows,
        );
        assert!(a.expanded_options);
        assert!(!a.hardware_decoding);
        assert!(!a.aoa_input);
        assert_eq!(
            a.hardware_decoders
                .iter()
                .map(|v| v.as_str())
                .collect::<Vec<_>>(),
            ["auto", "disabled", "d3d11va"]
        );
    }
    #[test]
    fn hardware_version_platform_and_unknown_are_checked_before_execution() {
        let args = build_arguments(
            &serde_json::from_value(json!({"video":{"hardwareDecoding":"d3d11va"}})).unwrap(),
        )
        .unwrap();
        assert!(
            validate_runtime_arguments(&args, &capabilities("5.0"), HostPlatform::Windows).is_ok()
        );
        for (caps, platform) in [
            (capabilities("4.1"), HostPlatform::Windows),
            (capabilities("5.0"), HostPlatform::Linux),
            (
                ScrcpyCapabilities::from_version(None),
                HostPlatform::Windows,
            ),
        ] {
            assert_eq!(
                validate_runtime_arguments(&args, &caps, platform)
                    .unwrap_err()
                    .code,
                "unsupported_option"
            );
        }
        for (platform, name) in [
            (HostPlatform::Linux, "vaapi"),
            (HostPlatform::Macos, "videotoolbox"),
        ] {
            let a = availability(&ScrcpyConfig::default(), &capabilities("5.0"), platform);
            assert!(a.hardware_decoding);
            assert!(a.hardware_decoders.iter().any(|v| v.as_str() == name));
        }
    }
    #[test]
    fn unknown_version_preserves_legacy_defaults_but_blocks_new_settings() {
        let caps = ScrcpyCapabilities::from_version(None);
        assert!(
            !availability(&ScrcpyConfig::default(), &caps, HostPlatform::Windows).expanded_options
        );
        assert!(validate_runtime_arguments(&[], &caps, HostPlatform::Windows).is_ok());
        assert!(validate_runtime_arguments(
            &["--video-codec=h265".into(), "--no-audio".into()],
            &caps,
            HostPlatform::Windows
        )
        .is_ok());
        assert!(validate_runtime_arguments(
            &["--max-size=1920".into()],
            &caps,
            HostPlatform::Windows
        )
        .is_err());
        assert!(validate_runtime_arguments(
            &["--max-size=1920".into()],
            &capabilities("4.0"),
            HostPlatform::Windows
        )
        .is_err());
    }
    #[test]
    fn dependencies_match_effective_builder_behavior() {
        let config = serde_json::from_value(
            json!({"audio":{"codec":"raw","source":"mic","duplication":true},
            "input":{"keyboard":"uhid","mouse":"disabled"}}),
        )
        .unwrap();
        let a = availability(&config, &capabilities("5.0"), HostPlatform::Windows);
        assert!(
            !a.audio_encoding
                && !a.audio_bitrate
                && !a.audio_duplication
                && !a.sdk_keyboard
                && !a.sdk_mouse
        );
        let disabled = serde_json::from_value(
            json!({"video":{"enabled":false},"audio":{"enabled":false},"input":{"enabled":false}}),
        )
        .unwrap();
        let a = availability(&disabled, &capabilities("5.0"), HostPlatform::Windows);
        assert!(
            !a.video
                && !a.audio
                && !a.control
                && !a.window
                && !a.device_power
                && !a.hardware_decoding
        );
    }
    #[test]
    fn windows_aoa_requires_excluded_otg_workflow() {
        for arg in ["--keyboard=aoa", "--mouse=aoa"] {
            assert!(validate_runtime_arguments(
                &[arg.into()],
                &capabilities("4.1"),
                HostPlatform::Windows
            )
            .is_err());
            assert!(validate_runtime_arguments(
                &[arg.into()],
                &capabilities("4.1"),
                HostPlatform::Linux
            )
            .is_ok());
        }
    }
}
