export type VideoCodec = "h264" | "h265" | "av1";
export type InputMode = "sdk" | "uhid" | "aoa" | "disabled";

// null means "use official scrcpy default". Wire keys match Rust serde camelCase.
export interface ScrcpyConfig {
  video: { codec: VideoCodec | null; bitrateMbps: number | null; maxFps: number | null };
  audio: { enabled: boolean | null };
  input: { keyboard: InputMode | null; mouse: InputMode | null };
}

export interface LauncherError { code: string; message: string; details: string | null }
export interface ScrcpyStatus {
  installed: boolean;
  executable: string | null;
  version: string | null;
  error: LauncherError | null;
}
export interface CommandPreview { executable: string; arguments: string[]; display: string }
export interface LaunchResult { command: CommandPreview; exitCode: number | null }
