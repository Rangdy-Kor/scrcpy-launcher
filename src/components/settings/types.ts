import type { ConfigAvailability, ScrcpyConfig } from "../../types/scrcpy";
export interface SettingsProps<K extends "video" | "audio" | "display" | "input"> {
  value: ScrcpyConfig[K]; onChange: (patch: Partial<ScrcpyConfig[K]>) => void; availability: ConfigAvailability | null;
}
