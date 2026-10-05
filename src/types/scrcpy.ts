export type VideoCodec = "h264" | "h265" | "av1";
export type InputMode = "sdk" | "uhid" | "aoa" | "disabled";

// null means "use official scrcpy default". Wire keys match Rust serde camelCase.
export interface ScrcpyConfig {
  device: { serial: string | null };
  video: { codec: VideoCodec | null; bitrateMbps: number | null; maxFps: number | null };
  audio: { enabled: boolean | null };
  input: { keyboard: InputMode | null; mouse: InputMode | null };
}

export interface LauncherError { code: string; message: string; details: string | null }
export interface ScrcpyVersion {
  major: number;
  minor: number;
  patch: number;
  raw: string;
}
export type CapabilitySupport = "supported" | "unsupported" | "unknown";
export interface ScrcpyCapabilities {
  hardwareDecoding: CapabilitySupport;
}
export interface ScrcpyStatus {
  installed: boolean;
  executable: string | null;
  version: ScrcpyVersion | null;
  versionOutput: string | null;
  capabilities: ScrcpyCapabilities;
  error: LauncherError | null;
}
export interface CommandPreview { executable: string; arguments: string[]; display: string }
export interface LaunchResult { command: CommandPreview; exitCode: number | null }
