pub mod arguments;
pub mod availability;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct ScrcpyConfig {
    pub device: DeviceConfig,
    pub video: VideoConfig,
    pub audio: AudioConfig,
    pub display: DisplayConfig,
    pub input: InputConfig,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct DeviceConfig {
    pub serial: Option<String>,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct VideoConfig {
    pub enabled: Option<bool>,
    pub codec: Option<VideoCodec>,
    pub bitrate_mbps: Option<u32>,
    pub max_fps: Option<u16>,
    pub max_size: Option<u16>,
    pub display_id: Option<u32>,
    pub encoder: Option<String>,
    pub buffer_ms: Option<u32>,
    pub capture_orientation: CaptureOrientation,
    pub downsize_on_error: Option<bool>,
    pub hardware_decoding: Option<HardwareDecoding>,
    pub codec_options: Vec<CodecOption>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum VideoCodec {
    H264,
    H265,
    Av1,
    Vp8,
    Vp9,
}

impl VideoCodec {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::H264 => "h264",
            Self::H265 => "h265",
            Self::Av1 => "av1",
            Self::Vp8 => "vp8",
            Self::Vp9 => "vp9",
        }
    }
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct AudioConfig {
    pub enabled: Option<bool>,
    pub source: Option<AudioSource>,
    pub codec: Option<AudioCodec>,
    pub bitrate_kbps: Option<u32>,
    pub encoder: Option<String>,
    pub buffer_ms: Option<u32>,
    pub output_buffer_ms: Option<u16>,
    pub duplication: Option<bool>,
    pub require_audio: Option<bool>,
    pub codec_options: Vec<CodecOption>,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct InputConfig {
    pub enabled: Option<bool>,
    pub keyboard: Option<KeyboardMode>,
    pub mouse: Option<MouseMode>,
    pub text_injection: Option<TextInjection>,
    pub key_repeat: Option<bool>,
    pub mouse_hover: Option<bool>,
    pub clipboard_autosync: Option<bool>,
    pub show_touches: Option<bool>,
    pub shortcut_modifiers: Vec<ShortcutModifier>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum KeyboardMode {
    Sdk,
    Uhid,
    Aoa,
    Disabled,
}

impl KeyboardMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sdk => "sdk",
            Self::Uhid => "uhid",
            Self::Aoa => "aoa",
            Self::Disabled => "disabled",
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MouseMode {
    Sdk,
    Uhid,
    Aoa,
    Disabled,
}
impl MouseMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sdk => "sdk",
            Self::Uhid => "uhid",
            Self::Aoa => "aoa",
            Self::Disabled => "disabled",
        }
    }
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct DisplayConfig {
    pub fullscreen: Option<bool>,
    pub always_on_top: Option<bool>,
    pub window_title: Option<String>,
    pub window_x: Option<i32>,
    pub window_y: Option<i32>,
    pub window_width: Option<u16>,
    pub window_height: Option<u16>,
    pub borderless: Option<bool>,
    pub orientation: Option<Orientation>,
    pub render_fit: Option<RenderFit>,
    pub aspect_ratio_lock: Option<bool>,
    pub disable_screensaver: Option<bool>,
    pub turn_screen_off: Option<bool>,
    pub stay_awake: Option<bool>,
    pub power_off_on_close: Option<bool>,
    pub power_on: Option<bool>,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct CaptureOrientation {
    pub orientation: Option<Orientation>,
    pub locked: Option<bool>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum Orientation {
    #[serde(rename = "0")]
    Deg0,
    #[serde(rename = "90")]
    Deg90,
    #[serde(rename = "180")]
    Deg180,
    #[serde(rename = "270")]
    Deg270,
    #[serde(rename = "flip0")]
    Flip0,
    #[serde(rename = "flip90")]
    Flip90,
    #[serde(rename = "flip180")]
    Flip180,
    #[serde(rename = "flip270")]
    Flip270,
}
impl Orientation {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Deg0 => "0",
            Self::Deg90 => "90",
            Self::Deg180 => "180",
            Self::Deg270 => "270",
            Self::Flip0 => "flip0",
            Self::Flip90 => "flip90",
            Self::Flip180 => "flip180",
            Self::Flip270 => "flip270",
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum HardwareDecoding {
    Auto,
    Disabled,
    Vaapi,
    D3d11va,
    Videotoolbox,
}
impl HardwareDecoding {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Disabled => "disabled",
            Self::Vaapi => "vaapi",
            Self::D3d11va => "d3d11va",
            Self::Videotoolbox => "videotoolbox",
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AudioSource {
    Output,
    Playback,
    Mic,
    MicUnprocessed,
    MicCamcorder,
    MicVoiceRecognition,
    MicVoiceCommunication,
    VoiceCall,
    VoiceCallUplink,
    VoiceCallDownlink,
    VoicePerformance,
}
impl AudioSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Output => "output",
            Self::Playback => "playback",
            Self::Mic => "mic",
            Self::MicUnprocessed => "mic-unprocessed",
            Self::MicCamcorder => "mic-camcorder",
            Self::MicVoiceRecognition => "mic-voice-recognition",
            Self::MicVoiceCommunication => "mic-voice-communication",
            Self::VoiceCall => "voice-call",
            Self::VoiceCallUplink => "voice-call-uplink",
            Self::VoiceCallDownlink => "voice-call-downlink",
            Self::VoicePerformance => "voice-performance",
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AudioCodec {
    Opus,
    Aac,
    Flac,
    Raw,
}
impl AudioCodec {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Opus => "opus",
            Self::Aac => "aac",
            Self::Flac => "flac",
            Self::Raw => "raw",
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RenderFit {
    Letterbox,
    Stretched,
    Unscaled,
}
impl RenderFit {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Letterbox => "letterbox",
            Self::Stretched => "stretched",
            Self::Unscaled => "unscaled",
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TextInjection {
    Mixed,
    Text,
    Raw,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ShortcutModifier {
    Lctrl,
    Rctrl,
    Lalt,
    Ralt,
    Lsuper,
    Rsuper,
}
impl ShortcutModifier {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Lctrl => "lctrl",
            Self::Rctrl => "rctrl",
            Self::Lalt => "lalt",
            Self::Ralt => "ralt",
            Self::Lsuper => "lsuper",
            Self::Rsuper => "rsuper",
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CodecOption {
    pub key: String,
    #[serde(rename = "type")]
    pub value_type: CodecOptionType,
    pub value: String,
}
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CodecOptionType {
    Int,
    Long,
    Float,
    String,
}
impl CodecOptionType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Int => "int",
            Self::Long => "long",
            Self::Float => "float",
            Self::String => "string",
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LauncherError {
    pub code: &'static str,
    pub message: String,
    pub details: Option<String>,
}

impl LauncherError {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: None,
        }
    }
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
}
