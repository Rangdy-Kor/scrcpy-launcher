import { useState } from "react";
import type { ScrcpyConfig } from "../types/scrcpy";

export function createDefaultConfig(): ScrcpyConfig {
  return {
    video: { codec: null, bitrateMbps: null, maxFps: null },
    audio: { enabled: null },
    input: { keyboard: null, mouse: null },
  };
}

export function useScrcpyConfig() {
  return useState<ScrcpyConfig>(createDefaultConfig);
}
