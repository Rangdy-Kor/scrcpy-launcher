import { invoke } from "@tauri-apps/api/core";
import type { AdbStatus } from "../types/device";
import type { CommandPreview, LauncherError, LaunchResult, ScrcpyConfig, ScrcpyStatus } from "../types/scrcpy";

export const getScrcpyStatus = () => invoke<ScrcpyStatus>("get_scrcpy_status");
export const getAdbDevices = () => invoke<AdbStatus>("get_adb_devices");
export const previewScrcpy = (config: ScrcpyConfig) => invoke<CommandPreview>("preview_scrcpy", { config });
export const launchScrcpy = (config: ScrcpyConfig) => invoke<LaunchResult>("launch_scrcpy", { config });

export function toLauncherError(error: unknown): LauncherError {
  if (typeof error === "object" && error !== null && "code" in error && "message" in error
    && typeof error.code === "string" && typeof error.message === "string") {
    return { code: error.code, message: error.message,
      details: "details" in error && typeof error.details === "string" ? error.details : null };
  }
  // Tauri transport/deserialization failures are strings, not backend Results.
  return { code: "request_failed", message: typeof error === "string" ? error : "Could not contact the Rust backend. Open this app with pnpm tauri dev.", details: null };
}
