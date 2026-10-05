pub mod arguments;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct ScrcpyConfig {
    pub device: DeviceConfig,
    pub video: VideoConfig,
    pub audio: AudioConfig,
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
    pub codec: Option<VideoCodec>,
    pub bitrate_mbps: Option<u32>,
    pub max_fps: Option<u16>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum VideoCodec {
    H264,
    H265,
    Av1,
}

impl VideoCodec {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::H264 => "h264",
            Self::H265 => "h265",
            Self::Av1 => "av1",
        }
    }
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct AudioConfig {
    pub enabled: Option<bool>,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct InputConfig {
    pub keyboard: Option<InputMode>,
    pub mouse: Option<InputMode>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum InputMode {
    Sdk,
    Uhid,
    Aoa,
    Disabled,
}

impl InputMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sdk => "sdk",
            Self::Uhid => "uhid",
            Self::Aoa => "aoa",
            Self::Disabled => "disabled",
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
