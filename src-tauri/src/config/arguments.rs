use super::{LauncherError, ScrcpyConfig};

/// Pure builder shared by preview and launch. No OS access or shell syntax.
pub fn build_arguments(config: &ScrcpyConfig) -> Result<Vec<String>, LauncherError> {
    let mut args = Vec::new();
    if let Some(codec) = config.video.codec {
        args.push(format!("--video-codec={}", codec.as_str()));
    }
    if let Some(bitrate) = config.video.bitrate_mbps {
        // Official CLI limits bit rate to signed 31 bits, including on Windows.
        // M is a decimal multiplier; this foundation exposes whole Mbps.
        if bitrate == 0 || bitrate > i32::MAX as u32 / 1_000_000 {
            return Err(LauncherError::new(
                "invalid_config",
                "Video bitrate must be an integer from 1 to 2147 Mbps.",
            ));
        }
        args.push(format!("--video-bit-rate={bitrate}M"));
    }
    if let Some(fps) = config.video.max_fps {
        if fps == 0 {
            return Err(LauncherError::new(
                "invalid_config",
                "Max FPS must be a positive integer (1–65535).",
            ));
        }
        args.push(format!("--max-fps={fps}"));
    }
    if config.audio.enabled == Some(false) {
        args.push("--no-audio".into());
    }
    if let Some(mode) = config.input.keyboard {
        args.push(format!("--keyboard={}", mode.as_str()));
    }
    if let Some(mode) = config.input.mouse {
        args.push(format!("--mouse={}", mode.as_str()));
    }
    Ok(args)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn args(value: serde_json::Value) -> Vec<String> {
        build_arguments(&serde_json::from_value(value).unwrap()).unwrap()
    }

    #[test]
    fn empty_configuration() {
        assert!(build_arguments(&ScrcpyConfig::default())
            .unwrap()
            .is_empty());
        assert!(args(json!({})).is_empty());
        assert!(args(json!({"video": {}, "audio": {}, "input": {}})).is_empty());
    }
    #[test]
    fn h265_bitrate_and_fps() {
        assert_eq!(
            args(json!({"video": {"codec":"h265", "bitrateMbps":20, "maxFps":60}})),
            ["--video-codec=h265", "--video-bit-rate=20M", "--max-fps=60"]
        );
    }
    #[test]
    fn uhid_keyboard_and_mouse() {
        assert_eq!(
            args(json!({"input": {"keyboard":"uhid", "mouse":"uhid"}})),
            ["--keyboard=uhid", "--mouse=uhid"]
        );
    }
    #[test]
    fn audio_disabled() {
        assert_eq!(args(json!({"audio": {"enabled":false}})), ["--no-audio"]);
    }
    #[test]
    fn audio_enabled_uses_scrcpy_default() {
        assert!(args(json!({"audio": {"enabled":true}})).is_empty());
    }
    #[test]
    fn combined_configuration() {
        assert_eq!(
            args(
                json!({"video":{"codec":"h265","bitrateMbps":20,"maxFps":60},
            "audio":{"enabled":false},"input":{"keyboard":"uhid","mouse":"uhid"}})
            ),
            [
                "--video-codec=h265",
                "--video-bit-rate=20M",
                "--max-fps=60",
                "--no-audio",
                "--keyboard=uhid",
                "--mouse=uhid"
            ]
        );
    }
    #[test]
    fn invalid_numeric_values() {
        for value in [
            json!({"video":{"bitrateMbps":0}}),
            json!({"video":{"bitrateMbps":2148}}),
            json!({"video":{"maxFps":0}}),
        ] {
            assert_eq!(
                build_arguments(&serde_json::from_value(value).unwrap())
                    .unwrap_err()
                    .code,
                "invalid_config"
            );
        }
    }
    #[test]
    fn rejects_unknown_options_and_modes() {
        for value in [
            json!({"extraArgs":"--anything"}),
            json!({"input":{"keyboard":"shell"}}),
            json!({"video":{"bitrateMbps":-1}}),
        ] {
            assert!(serde_json::from_value::<ScrcpyConfig>(value).is_err());
        }
    }
    #[test]
    fn supported_modes_and_numeric_boundaries() {
        assert_eq!(
            args(
                json!({"video":{"codec":"av1","bitrateMbps":2147,"maxFps":65535},"input":{"keyboard":"sdk","mouse":"disabled"}})
            ),
            [
                "--video-codec=av1",
                "--video-bit-rate=2147M",
                "--max-fps=65535",
                "--keyboard=sdk",
                "--mouse=disabled"
            ]
        );
        assert_eq!(
            args(json!({"video":{"codec":"h264"},"input":{"keyboard":"aoa","mouse":"aoa"}})),
            ["--video-codec=h264", "--keyboard=aoa", "--mouse=aoa"]
        );
    }
}
