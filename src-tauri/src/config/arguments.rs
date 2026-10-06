use super::{
    availability::dependencies, CodecOption, CodecOptionType, LauncherError, ScrcpyConfig,
    TextInjection,
};

/// Pure builder shared by preview and launch. No OS access or shell syntax.
pub fn build_arguments(config: &ScrcpyConfig) -> Result<Vec<String>, LauncherError> {
    let mut args = Vec::new();
    let active = dependencies(config);
    if !active.video && !active.audio && !active.control {
        return Err(LauncherError::new(
            "option_conflict",
            "Video, audio and control cannot all be disabled: scrcpy would have nothing to do.",
        ));
    }
    if let Some(serial) = &config.device.serial {
        if serial.trim().is_empty() || serial.chars().any(char::is_control) {
            return Err(LauncherError::new(
                "invalid_config",
                "Device serial must be nonempty and contain no control characters.",
            ));
        }
        args.push(format!("--serial={serial}"));
    }
    if config.video.enabled == Some(false) {
        args.push("--no-video".into());
    }
    if active.video {
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
        number(
            &mut args,
            "max-size",
            config.video.max_size.map(u32::from),
            0,
            65535,
        )?;
        number(
            &mut args,
            "display-id",
            config.video.display_id,
            0,
            i32::MAX as u32,
        )?;
        text(&mut args, "video-encoder", config.video.encoder.as_deref())?;
        number(
            &mut args,
            "video-buffer",
            config.video.buffer_ms,
            0,
            3_600_000,
        )?;
        let capture = &config.video.capture_orientation;
        if capture.orientation.is_some() || capture.locked == Some(true) {
            args.push(format!(
                "--capture-orientation={}{}",
                if capture.locked == Some(true) {
                    "@"
                } else {
                    ""
                },
                capture
                    .orientation
                    .map(|value| value.as_str())
                    .unwrap_or("")
            ));
        }
        flag(
            &mut args,
            "no-downsize-on-error",
            config.video.downsize_on_error == Some(false),
        );
        if let Some(mode) = config.video.hardware_decoding {
            args.push(format!("--hwdec={}", mode.as_str()));
        }
        codec_options(
            &mut args,
            "video-codec-options",
            &config.video.codec_options,
        )?;
    }
    if config.audio.enabled == Some(false) {
        args.push("--no-audio".into());
    }
    if active.audio {
        if let Some(source) = config.audio.source {
            args.push(format!("--audio-source={}", source.as_str()));
        }
        if let Some(codec) = config.audio.codec {
            args.push(format!("--audio-codec={}", codec.as_str()));
        }
        if active.audio_bitrate {
            if let Some(bitrate) = config.audio.bitrate_kbps {
                if !(1..=i32::MAX as u32 / 1000).contains(&bitrate) {
                    return Err(LauncherError::new(
                        "invalid_config",
                        "Audio bitrate must be 1–2147483 Kbps, or Default.",
                    ));
                }
                args.push(format!("--audio-bit-rate={bitrate}K"));
            }
        }
        if active.audio_encoding {
            text(&mut args, "audio-encoder", config.audio.encoder.as_deref())?;
            codec_options(
                &mut args,
                "audio-codec-options",
                &config.audio.codec_options,
            )?;
        }
        number(
            &mut args,
            "audio-buffer",
            config.audio.buffer_ms,
            0,
            3_600_000,
        )?;
        number(
            &mut args,
            "audio-output-buffer",
            config.audio.output_buffer_ms.map(u32::from),
            0,
            1000,
        )?;
        flag(
            &mut args,
            "audio-dup",
            active.audio_duplication && config.audio.duplication == Some(true),
        );
        flag(
            &mut args,
            "require-audio",
            config.audio.require_audio == Some(true),
        );
    }
    let display = &config.display;
    if active.window {
        flag(&mut args, "fullscreen", display.fullscreen == Some(true));
        flag(
            &mut args,
            "always-on-top",
            display.always_on_top == Some(true),
        );
        flag(
            &mut args,
            "window-borderless",
            display.borderless == Some(true),
        );
        text(&mut args, "window-title", display.window_title.as_deref())?;
        for (name, value) in [
            ("window-x", display.window_x),
            ("window-y", display.window_y),
        ] {
            if let Some(value) = value {
                if !(-32767..=32767).contains(&value) {
                    return Err(LauncherError::new(
                        "invalid_config",
                        "Window position must be -32767 to 32767, or Default.",
                    ));
                }
                args.push(format!("--{name}={value}"));
            }
        }
        number(
            &mut args,
            "window-width",
            display.window_width.map(u32::from),
            0,
            65535,
        )?;
        number(
            &mut args,
            "window-height",
            display.window_height.map(u32::from),
            0,
            65535,
        )?;
        if let Some(orientation) = display.orientation {
            args.push(format!("--display-orientation={}", orientation.as_str()));
        }
        if let Some(fit) = display.render_fit {
            args.push(format!("--render-fit={}", fit.as_str()));
        }
        flag(
            &mut args,
            "no-window-aspect-ratio-lock",
            display.aspect_ratio_lock == Some(false),
        );
        flag(
            &mut args,
            "disable-screensaver",
            display.disable_screensaver == Some(true),
        );
    }
    if active.device_power {
        flag(
            &mut args,
            "turn-screen-off",
            display.turn_screen_off == Some(true),
        );
        flag(&mut args, "stay-awake", display.stay_awake == Some(true));
        flag(
            &mut args,
            "power-off-on-close",
            display.power_off_on_close == Some(true),
        );
        // Official no-video already suppresses power-on; omit this inactive setting.
        flag(
            &mut args,
            "no-power-on",
            active.video && display.power_on == Some(false),
        );
    }
    if config.input.enabled == Some(false) {
        args.push("--no-control".into());
    }
    if active.control {
        if let Some(mode) = config.input.keyboard {
            args.push(format!("--keyboard={}", mode.as_str()));
        }
        if let Some(mode) = config.input.mouse {
            args.push(format!("--mouse={}", mode.as_str()));
        }
        if active.sdk_keyboard {
            match config.input.text_injection {
                Some(TextInjection::Text) => args.push("--prefer-text".into()),
                Some(TextInjection::Raw) => args.push("--raw-key-events".into()),
                _ => {}
            }
            flag(
                &mut args,
                "no-key-repeat",
                config.input.key_repeat == Some(false),
            );
        }
        flag(
            &mut args,
            "no-mouse-hover",
            active.sdk_mouse && config.input.mouse_hover == Some(false),
        );
        flag(
            &mut args,
            "no-clipboard-autosync",
            config.input.clipboard_autosync == Some(false),
        );
        flag(
            &mut args,
            "show-touches",
            config.input.show_touches == Some(true),
        );
    }
    // Host shortcuts (including quit/fullscreen) are useful even without control.
    if !config.input.shortcut_modifiers.is_empty() {
        let modifiers: Vec<_> = config
            .input
            .shortcut_modifiers
            .iter()
            .map(|modifier| modifier.as_str())
            .collect();
        let unique: std::collections::HashSet<_> = modifiers.iter().collect();
        if unique.len() != modifiers.len() {
            return Err(LauncherError::new(
                "invalid_config",
                "Shortcut modifiers must not repeat.",
            ));
        }
        args.push(format!("--shortcut-mod={}", modifiers.join(",")));
    }
    Ok(args)
}

fn flag(args: &mut Vec<String>, name: &str, enabled: bool) {
    if enabled {
        args.push(format!("--{name}"));
    }
}
fn number(
    args: &mut Vec<String>,
    name: &str,
    value: Option<u32>,
    min: u32,
    max: u32,
) -> Result<(), LauncherError> {
    if let Some(value) = value {
        if !(min..=max).contains(&value) {
            return Err(LauncherError::new(
                "invalid_config",
                format!("{name} must be {min}–{max}, or Default."),
            ));
        }
        args.push(format!("--{name}={value}"));
    }
    Ok(())
}
fn text(args: &mut Vec<String>, name: &str, value: Option<&str>) -> Result<(), LauncherError> {
    if let Some(value) = value {
        if value.trim().is_empty() || value.chars().any(char::is_control) {
            return Err(LauncherError::new("invalid_config", format!("{name} must be nonempty and contain no control characters. Clear it for Default.")));
        }
        args.push(format!("--{name}={value}"));
    }
    Ok(())
}
fn codec_options(
    args: &mut Vec<String>,
    name: &str,
    options: &[CodecOption],
) -> Result<(), LauncherError> {
    let mut keys = std::collections::HashSet::new();
    let mut values = Vec::new();
    for option in options {
        let key_valid = !option.key.is_empty()
            && option
                .key
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_' | b'.'))
            && keys.insert(&option.key);
        let value_valid = !option.value.is_empty()
            && !option.value.contains(',')
            && !option.value.chars().any(char::is_control)
            && match option.value_type {
                CodecOptionType::Int => option.value.parse::<i32>().is_ok(),
                CodecOptionType::Long => option.value.parse::<i64>().is_ok(),
                CodecOptionType::Float => option
                    .value
                    .parse::<f32>()
                    .is_ok_and(|value| value.is_finite()),
                CodecOptionType::String => true,
            };
        if !key_valid || !value_valid {
            return Err(LauncherError::new("invalid_config", format!("{name}: enter unique MediaFormat keys and valid typed values (no commas/control characters).")));
        }
        values.push(format!(
            "{}:{}={}",
            option.key,
            option.value_type.as_str(),
            option.value
        ));
    }
    if !values.is_empty() {
        args.push(format!("--{name}={}", values.join(",")));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn args(value: serde_json::Value) -> Vec<String> {
        build_arguments(&serde_json::from_value(value).unwrap()).unwrap()
    }

    #[test]
    fn unselected_device_adds_no_argument() {
        assert!(args(json!({"device":{"serial":null}})).is_empty());
    }
    #[test]
    fn serial_selection() {
        assert_eq!(
            args(json!({"device":{"serial":"192.168.1.188:42999"}})),
            ["--serial=192.168.1.188:42999"]
        );
    }
    #[test]
    fn serial_combined_with_existing_options() {
        assert_eq!(
            args(
                json!({"device":{"serial":"adb-example._adb-tls-connect._tcp"},
            "video":{"codec":"h265","bitrateMbps":20,"maxFps":60},"audio":{"enabled":false},"input":{"keyboard":"uhid","mouse":"uhid"}})
            ),
            [
                "--serial=adb-example._adb-tls-connect._tcp",
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
    fn serial_is_one_literal_argument() {
        let serial = r#"serial with spaces; & $(command) "quote" --no-video"#;
        assert_eq!(
            args(json!({"device":{"serial":serial}})),
            [format!("--serial={serial}")]
        );
    }
    #[test]
    fn invalid_serial_format() {
        for serial in ["", "  ", "abc\nxyz", "abc\0xyz"] {
            assert_eq!(
                build_arguments(
                    &serde_json::from_value(json!({"device":{"serial":serial}})).unwrap()
                )
                .unwrap_err()
                .code,
                "invalid_config"
            );
        }
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

    #[test]
    fn video_expansion_and_advanced_arguments() {
        assert_eq!(
            args(json!({"video":{"codec":"vp9","maxSize":1920,"displayId":2,
            "encoder":"c2.qti.vp9.encoder","bufferMs":50,"captureOrientation":{"orientation":"flip90","locked":true},
            "downsizeOnError":false,"hardwareDecoding":"d3d11va","codecOptions":[{"key":"quality","type":"int","value":"80"}]}})),
            [
                "--video-codec=vp9",
                "--max-size=1920",
                "--display-id=2",
                "--video-encoder=c2.qti.vp9.encoder",
                "--video-buffer=50",
                "--capture-orientation=@flip90",
                "--no-downsize-on-error",
                "--hwdec=d3d11va",
                "--video-codec-options=quality:int=80"
            ]
        );
        assert_eq!(
            args(json!({"video":{"captureOrientation":{"locked":true}}})),
            ["--capture-orientation=@"]
        );
        assert_eq!(
            args(
                json!({"video":{"codec":"vp8","captureOrientation":{"orientation":"270","locked":false}}})
            ),
            ["--video-codec=vp8", "--capture-orientation=270"]
        );
    }

    #[test]
    fn audio_expansion_and_advanced_arguments() {
        assert_eq!(
            args(
                json!({"audio":{"source":"playback","codec":"aac","bitrateKbps":192,
            "encoder":"c2.android.aac.encoder","bufferMs":80,"outputBufferMs":20,"duplication":true,"requireAudio":true,
            "codecOptions":[{"key":"aac-profile","type":"int","value":"2"}]}})
            ),
            [
                "--audio-source=playback",
                "--audio-codec=aac",
                "--audio-bit-rate=192K",
                "--audio-encoder=c2.android.aac.encoder",
                "--audio-codec-options=aac-profile:int=2",
                "--audio-buffer=80",
                "--audio-output-buffer=20",
                "--audio-dup",
                "--require-audio"
            ]
        );
    }

    #[test]
    fn display_window_and_device_power_arguments() {
        assert_eq!(
            args(
                json!({"display":{"fullscreen":true,"alwaysOnTop":true,"borderless":true,
            "windowTitle":"Galaxy & desktop","windowX":-100,"windowY":200,"windowWidth":800,"windowHeight":600,
            "orientation":"90","renderFit":"stretched","aspectRatioLock":false,"disableScreensaver":true,
            "turnScreenOff":true,"stayAwake":true,"powerOffOnClose":true,"powerOn":false}})
            ),
            [
                "--fullscreen",
                "--always-on-top",
                "--window-borderless",
                "--window-title=Galaxy & desktop",
                "--window-x=-100",
                "--window-y=200",
                "--window-width=800",
                "--window-height=600",
                "--display-orientation=90",
                "--render-fit=stretched",
                "--no-window-aspect-ratio-lock",
                "--disable-screensaver",
                "--turn-screen-off",
                "--stay-awake",
                "--power-off-on-close",
                "--no-power-on"
            ]
        );
    }

    #[test]
    fn input_sdk_options_and_shortcut_modifiers() {
        assert_eq!(
            args(
                json!({"input":{"textInjection":"text","keyRepeat":false,"mouseHover":false,
            "clipboardAutosync":false,"showTouches":true,"shortcutModifiers":["lalt","rsuper"]}})
            ),
            [
                "--prefer-text",
                "--no-key-repeat",
                "--no-mouse-hover",
                "--no-clipboard-autosync",
                "--show-touches",
                "--shortcut-mod=lalt,rsuper"
            ]
        );
        assert_eq!(
            args(json!({"input":{"textInjection":"raw"}})),
            ["--raw-key-events"]
        );
        assert!(args(json!({"input":{"textInjection":"mixed"}})).is_empty());
        assert!(serde_json::from_value::<ScrcpyConfig>(
            json!({"input":{"textInjection":["text","raw"]}})
        )
        .is_err());
    }

    #[test]
    fn inactive_video_and_window_values_are_preserved_but_not_emitted() {
        let config: ScrcpyConfig = serde_json::from_value(json!({"video":{"enabled":false,"bitrateMbps":0,
            "maxSize":1920,"hardwareDecoding":"auto","codecOptions":[{"key":"bad,key","type":"int","value":"bad"}]},
            "display":{"fullscreen":true,"windowTitle":"window","powerOn":false}})).unwrap();
        assert_eq!(build_arguments(&config).unwrap(), ["--no-video"]);
        assert_eq!(config.video.bitrate_mbps, Some(0));
        assert_eq!(config.video.max_size, Some(1920));
    }

    #[test]
    fn inactive_audio_values_are_restored_when_reenabled() {
        let mut config: ScrcpyConfig =
            serde_json::from_value(json!({"audio":{"enabled":false,"codec":"aac",
            "bitrateKbps":128,"duplication":true,"source":"playback"}}))
            .unwrap();
        assert_eq!(build_arguments(&config).unwrap(), ["--no-audio"]);
        config.audio.enabled = None;
        assert_eq!(
            build_arguments(&config).unwrap(),
            [
                "--audio-source=playback",
                "--audio-codec=aac",
                "--audio-bit-rate=128K",
                "--audio-dup"
            ]
        );
    }

    #[test]
    fn lossless_audio_encoding_dependencies() {
        assert_eq!(
            args(
                json!({"audio":{"codec":"raw","bitrateKbps":0,"encoder":"invalid\nname",
            "codecOptions":[{"key":"invalid","type":"int","value":"bad"}]}})
            ),
            ["--audio-codec=raw"]
        );
        assert_eq!(
            args(json!({"audio":{"codec":"flac","bitrateKbps":0,"encoder":"flac.encoder"}})),
            ["--audio-codec=flac", "--audio-encoder=flac.encoder"]
        );
        assert_eq!(
            args(json!({"audio":{"source":"mic","duplication":true}})),
            ["--audio-source=mic"]
        );
        assert_eq!(args(json!({"audio":{"duplication":true}})), ["--audio-dup"]);
    }

    #[test]
    fn disabled_control_omits_device_power_and_input_children() {
        assert_eq!(
            args(
                json!({"input":{"enabled":false,"keyboard":"uhid","mouse":"uhid","textInjection":"raw",
            "keyRepeat":false,"mouseHover":false,"showTouches":true,"clipboardAutosync":false,"shortcutModifiers":["lctrl"]},
            "display":{"turnScreenOff":true,"stayAwake":true,"powerOffOnClose":true,"powerOn":false}})
            ),
            ["--no-control", "--shortcut-mod=lctrl"]
        );
        assert_eq!(
            args(
                json!({"input":{"keyboard":"uhid","mouse":"disabled","textInjection":"text","keyRepeat":false,"mouseHover":false}})
            ),
            ["--keyboard=uhid", "--mouse=disabled"]
        );
    }

    #[test]
    fn nothing_to_do_is_a_domain_conflict() {
        let config = serde_json::from_value(
            json!({"video":{"enabled":false},"audio":{"enabled":false},"input":{"enabled":false}}),
        )
        .unwrap();
        assert_eq!(
            build_arguments(&config).unwrap_err().code,
            "option_conflict"
        );
    }

    #[test]
    fn validates_advanced_values_and_typed_codec_options() {
        for value in [
            json!({"video":{"displayId":2147483648_u64}}),
            json!({"video":{"bufferMs":3600001}}),
            json!({"audio":{"bitrateKbps":0}}),
            json!({"audio":{"outputBufferMs":1001}}),
            json!({"display":{"windowX":-32768}}),
            json!({"display":{"windowY":32768}}),
            json!({"display":{"windowTitle":"bad\nname"}}),
            json!({"video":{"encoder":" "}}),
            json!({"input":{"shortcutModifiers":["lalt","lalt"]}}),
        ] {
            assert_eq!(
                build_arguments(&serde_json::from_value(value).unwrap())
                    .unwrap_err()
                    .code,
                "invalid_config"
            );
        }
        for (kind, value) in [
            ("int", "2147483648"),
            ("long", "9223372036854775808"),
            ("float", "NaN"),
            ("float", "inf"),
            ("string", "a,b"),
        ] {
            assert!(build_arguments(
                &serde_json::from_value(
                    json!({"video":{"codecOptions":[{"key":"key","type":kind,"value":value}]}})
                )
                .unwrap()
            )
            .is_err());
        }
        for options in [
            json!([{"key":"a,b","type":"int","value":"1"}]),
            json!([{"key":"a","type":"int","value":"1"},{"key":"a","type":"long","value":"2"}]),
        ] {
            assert!(build_arguments(
                &serde_json::from_value(json!({"audio":{"codecOptions":options}})).unwrap()
            )
            .is_err());
        }
        assert_eq!(args(json!({"video":{"codecOptions":[{"key":"count","type":"long","value":"9223372036854775807"},
            {"key":"rate","type":"float","value":"1.5"},{"key":"label","type":"string","value":"hello & world"}]}})),
            ["--video-codec-options=count:long=9223372036854775807,rate:float=1.5,label:string=hello & world"]);
    }

    #[test]
    fn explicit_default_equivalents_do_not_force_unnecessary_flags() {
        assert!(args(json!({"video":{"enabled":true,"downsizeOnError":true,"captureOrientation":{"locked":false}},
            "audio":{"enabled":true,"duplication":false,"requireAudio":false},"input":{"enabled":true,"textInjection":"mixed","keyRepeat":true,"mouseHover":true,"clipboardAutosync":true,"showTouches":false},
            "display":{"fullscreen":false,"alwaysOnTop":false,"borderless":false,"aspectRatioLock":true,"powerOn":true}})).is_empty());
    }

    #[test]
    fn device_and_all_categories_remain_literal_arguments() {
        assert_eq!(
            args(
                json!({"device":{"serial":"adb-example._adb-tls-connect._tcp"},"video":{"maxSize":1600},
            "audio":{"codec":"opus","bitrateKbps":96},"display":{"windowTitle":"Galaxy; & --no-control"},
            "input":{"keyboard":"uhid","mouse":"uhid"}})
            ),
            [
                "--serial=adb-example._adb-tls-connect._tcp",
                "--max-size=1600",
                "--audio-codec=opus",
                "--audio-bit-rate=96K",
                "--window-title=Galaxy; & --no-control",
                "--keyboard=uhid",
                "--mouse=uhid"
            ]
        );
    }
}
