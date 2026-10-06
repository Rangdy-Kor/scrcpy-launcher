import type { LauncherError } from "./scrcpy";

export type AdbConnection = "usb" | "tcpIp" | "wirelessDebugging" | "emulator" | "unknown";

export interface AdbDevice {
  serial: string;
  state: string;
  model: string | null;
  product: string | null;
  device: string | null;
  transportId: number | null;
  usb: string | null;
  connection: AdbConnection;
}

export interface AdbStatus {
  installed: boolean;
  executable: string | null;
  displayExecutable: string | null;
  source: "path" | "scrcpyDirectory" | null;
  devices: AdbDevice[];
  error: LauncherError | null;
}
