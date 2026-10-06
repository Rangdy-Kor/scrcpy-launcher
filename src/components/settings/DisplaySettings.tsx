import { Advanced, Choice, Numeric, orientations, TextSetting, Toggle } from "./Controls";
import type { SettingsProps } from "./types";
export function DisplaySettings({ value: v, onChange: set, availability: a }: SettingsProps<"display">) {
  const windowOff = !a?.expandedOptions || !a?.window; const powerOff = !a?.expandedOptions || !a?.devicePower;
  return <>
    <h3>Host window</h3>
    <Toggle label="Fullscreen" positiveOnly value={v.fullscreen} disabled={windowOff} onChange={fullscreen => set({ fullscreen })} />
    <Toggle label="Always on top" positiveOnly value={v.alwaysOnTop} disabled={windowOff} onChange={alwaysOnTop => set({ alwaysOnTop })} />
    <Choice label="Display orientation" value={v.orientation} options={orientations} disabled={windowOff} onChange={orientation => set({ orientation })} />
    <TextSetting label="Window title" value={v.windowTitle} disabled={windowOff} onChange={windowTitle => set({ windowTitle })} />
    <h3>Android screen & power</h3><p className="hint">Control이 꺼져 있으면 아래 기기 제어 설정은 생략합니다.</p>
    <Toggle label="Turn screen off on start" positiveOnly value={v.turnScreenOff} disabled={powerOff} onChange={turnScreenOff => set({ turnScreenOff })} />
    <Toggle label="Stay awake while plugged in" positiveOnly value={v.stayAwake} disabled={powerOff} onChange={stayAwake => set({ stayAwake })} />
    <Advanced>
      <h3>Window geometry & rendering</h3>
      <Numeric label="Window X" min={-32767} max={32767} value={v.windowX} disabled={windowOff} onChange={windowX => set({ windowX })} />
      <Numeric label="Window Y" min={-32767} max={32767} value={v.windowY} disabled={windowOff} onChange={windowY => set({ windowY })} />
      <Numeric label="Window width · px" max={65535} value={v.windowWidth} disabled={windowOff} onChange={windowWidth => set({ windowWidth })} />
      <Numeric label="Window height · px" hint="크기 0은 자동 계산입니다." max={65535} value={v.windowHeight} disabled={windowOff} onChange={windowHeight => set({ windowHeight })} />
      <Toggle label="Borderless" positiveOnly value={v.borderless} disabled={windowOff} onChange={borderless => set({ borderless })} />
      <Choice label="Render fit" value={v.renderFit} options={["letterbox", "stretched", "unscaled"]} disabled={windowOff} onChange={renderFit => set({ renderFit })} />
      <Toggle label="Lock window aspect ratio" value={v.aspectRatioLock} disabled={windowOff} onChange={aspectRatioLock => set({ aspectRatioLock })} />
      <Toggle label="Disable host screensaver" positiveOnly value={v.disableScreensaver} disabled={windowOff} onChange={disableScreensaver => set({ disableScreensaver })} />
      <h3>Device power</h3>
      <Toggle label="Power off screen on close" positiveOnly value={v.powerOffOnClose} disabled={powerOff} onChange={powerOffOnClose => set({ powerOffOnClose })} />
      <Toggle label="Power on at start" hint="Video가 꺼져 있으면 scrcpy 자체가 power-on을 생략합니다." value={v.powerOn} disabled={powerOff || !a?.video} onChange={powerOn => set({ powerOn })} />
    </Advanced>
  </>;
}
