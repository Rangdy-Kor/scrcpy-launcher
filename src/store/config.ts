import { useState } from "react";
import type { ScrcpyConfig } from "../types/scrcpy";

export function createDefaultConfig(): ScrcpyConfig {
  return {
    device: { serial: null },
    video: { enabled: null, codec: null, bitrateMbps: null, maxFps: null, maxSize: null, displayId: null,
      encoder: null, bufferMs: null, captureOrientation: { orientation: null, locked: null },
      downsizeOnError: null, hardwareDecoding: null, codecOptions: [] },
    audio: { enabled: null, source: null, codec: null, bitrateKbps: null, encoder: null,
      bufferMs: null, outputBufferMs: null, duplication: null, requireAudio: null, codecOptions: [] },
    display: { fullscreen: null, alwaysOnTop: null, windowTitle: null, windowX: null, windowY: null,
      windowWidth: null, windowHeight: null, borderless: null, orientation: null, renderFit: null,
      aspectRatioLock: null, disableScreensaver: null, turnScreenOff: null, stayAwake: null, powerOffOnClose: null, powerOn: null },
    input: { enabled: null, keyboard: null, mouse: null, textInjection: null, keyRepeat: null,
      mouseHover: null, clipboardAutosync: null, showTouches: null, shortcutModifiers: [] },
  };
}

export function useScrcpyConfig() {
  return useState<ScrcpyConfig>(createDefaultConfig);
}
