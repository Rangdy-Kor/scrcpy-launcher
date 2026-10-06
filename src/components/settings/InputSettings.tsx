import { Advanced, Choice, Toggle } from "./Controls";
import type { ShortcutModifier } from "../../types/scrcpy";
import type { SettingsProps } from "./types";
const modifiers: ShortcutModifier[] = ["lctrl", "rctrl", "lalt", "ralt", "lsuper", "rsuper"];
export function InputSettings({ value: v, onChange: set, availability: a }: SettingsProps<"input">) {
  const expanded = !a?.expandedOptions; const off = !a?.control;
  return <>
    <Toggle label="Device control" hint="끄면 keyboard, mouse 및 기기 제어 하위 설정을 생략합니다." value={v.enabled} disabled={expanded} onChange={enabled => set({ enabled })} />
    <Choice label="Keyboard mode" hint="Windows mirroring에서는 AOA를 사용할 수 없습니다. UHID 또는 SDK를 사용하세요." value={v.keyboard} options={a?.aoaInput ? ["sdk", "uhid", "aoa", "disabled"] : ["sdk", "uhid", "disabled"]} disabled={off} onChange={keyboard => set({ keyboard })} />
    <Choice label="Mouse mode" hint="UHID는 pointer를 기기로 캡처합니다. Shortcut modifier로 해제합니다. AOA는 USB 전용입니다." value={v.mouse} options={a?.aoaInput ? ["sdk", "uhid", "aoa", "disabled"] : ["sdk", "uhid", "disabled"]} disabled={off} onChange={mouse => set({ mouse })} />
    <Toggle label="Clipboard auto-sync" value={v.clipboardAutosync} disabled={expanded || off} onChange={clipboardAutosync => set({ clipboardAutosync })} />
    <Advanced>
      <Choice label="Text injection" hint="SDK keyboard 전용입니다. Text와 Raw는 함께 지정할 수 없습니다." value={v.textInjection}
        options={[["mixed", "Mixed"], ["text", "Prefer text"], ["raw", "Raw key events"]]} disabled={expanded || !a?.sdkKeyboard} onChange={textInjection => set({ textInjection })} />
      <Toggle label="Key repeat" hint="SDK keyboard 전용입니다." value={v.keyRepeat} disabled={expanded || !a?.sdkKeyboard} onChange={keyRepeat => set({ keyRepeat })} />
      <Toggle label="Mouse hover" hint="SDK mouse 전용입니다." value={v.mouseHover} disabled={expanded || !a?.sdkMouse} onChange={mouseHover => set({ mouseHover })} />
      <Toggle label="Show touches on device" hint="기기에서 손가락으로 터치한 위치를 표시합니다." positiveOnly value={v.showTouches} disabled={expanded || off} onChange={showTouches => set({ showTouches })} />
      <fieldset className="shortcut-options" disabled={expanded}><legend>Shortcut modifiers</legend>
        <p className="hint">선택하지 않으면 Default입니다. 각 선택은 독립적인 modifier이며 조합 키가 아닙니다.</p>
        {modifiers.map(modifier => <label key={modifier}><input type="checkbox" checked={v.shortcutModifiers.includes(modifier)}
          onChange={e => set({ shortcutModifiers: e.target.checked ? [...v.shortcutModifiers, modifier] : v.shortcutModifiers.filter(m => m !== modifier) })} />{modifier}</label>)}
      </fieldset>
    </Advanced>
  </>;
}
