export type VideoCodec = "h264" | "h265" | "av1" | "vp8" | "vp9";
export type KeyboardMode = "sdk" | "uhid" | "aoa" | "disabled";
export type MouseMode = "sdk" | "uhid" | "aoa" | "disabled";
export type AudioCodec = "opus" | "aac" | "flac" | "raw";
export type AudioSource = "output" | "playback" | "mic" | "mic-unprocessed" | "mic-camcorder" | "mic-voice-recognition" | "mic-voice-communication" | "voice-call" | "voice-call-uplink" | "voice-call-downlink" | "voice-performance";
export type Orientation = "0" | "90" | "180" | "270" | "flip0" | "flip90" | "flip180" | "flip270";
export type HardwareDecoding = "auto" | "disabled" | "vaapi" | "d3d11va" | "videotoolbox";
export type ShortcutModifier = "lctrl" | "rctrl" | "lalt" | "ralt" | "lsuper" | "rsuper";
export interface CodecOption { key: string; type: "int" | "long" | "float" | "string"; value: string }
// null omits arguments and defers to installed scrcpy. Inactive values are retained.
export interface ScrcpyConfig {
  device: { serial: string | null };
  video: {
    enabled: boolean | null; codec: VideoCodec | null; bitrateMbps: number | null; maxFps: number | null;
    maxSize: number | null; displayId: number | null; encoder: string | null; bufferMs: number | null;
    captureOrientation: { orientation: Orientation | null; locked: boolean | null };
    downsizeOnError: boolean | null; hardwareDecoding: HardwareDecoding | null; codecOptions: CodecOption[];
  };
  audio: {
    enabled: boolean | null; source: AudioSource | null; codec: AudioCodec | null; bitrateKbps: number | null;
    encoder: string | null; bufferMs: number | null; outputBufferMs: number | null;
    duplication: boolean | null; requireAudio: boolean | null; codecOptions: CodecOption[];
  };
  display: {
    fullscreen: boolean | null; alwaysOnTop: boolean | null; windowTitle: string | null;
    windowX: number | null; windowY: number | null; windowWidth: number | null; windowHeight: number | null;
    borderless: boolean | null; orientation: Orientation | null; renderFit: "letterbox" | "stretched" | "unscaled" | null;
    aspectRatioLock: boolean | null; disableScreensaver: boolean | null; turnScreenOff: boolean | null;
    stayAwake: boolean | null; powerOffOnClose: boolean | null; powerOn: boolean | null;
  };
  input: {
    enabled: boolean | null; keyboard: KeyboardMode | null; mouse: MouseMode | null;
    textInjection: "mixed" | "text" | "raw" | null; keyRepeat: boolean | null; mouseHover: boolean | null;
    clipboardAutosync: boolean | null; showTouches: boolean | null; shortcutModifiers: ShortcutModifier[];
  };
}
export interface LauncherError { code: string; message: string; details: string | null }
export interface ScrcpyVersion { major: number; minor: number; patch: number; raw: string }
export type CapabilitySupport = "supported" | "unsupported" | "unknown";
export interface ScrcpyCapabilities { expandedOptions: CapabilitySupport; hardwareDecoding: CapabilitySupport }
export interface ScrcpyStatus {
  installed: boolean; executable: string | null; displayExecutable: string | null;
  version: ScrcpyVersion | null; versionOutput: string | null; capabilities: ScrcpyCapabilities; error: LauncherError | null;
}
export interface ConfigAvailability {
  video: boolean; audio: boolean; control: boolean; window: boolean; audioEncoding: boolean;
  audioBitrate: boolean; audioDuplication: boolean; sdkKeyboard: boolean; sdkMouse: boolean; devicePower: boolean;
  expandedOptions: boolean; hardwareDecoding: boolean; hardwareDecoders: HardwareDecoding[]; aoaInput: boolean;
}
export interface CommandPreview { executable: string; displayExecutable: string; arguments: string[]; display: string }
export interface ConfigPreview { command: CommandPreview | null; availability: ConfigAvailability; error: LauncherError | null }
export interface LaunchResult { command: CommandPreview; exitCode: number | null }
