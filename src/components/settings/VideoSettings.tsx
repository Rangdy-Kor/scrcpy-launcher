import { Advanced, Choice, CodecOptions, Numeric, orientations, TextSetting, Toggle } from "./Controls";
import type { SettingsProps } from "./types";
export function VideoSettings({ value: v, onChange: set, availability: a }: SettingsProps<"video">) {
  const expanded = !a?.expandedOptions; const off = !a?.video; const more = expanded || off;
  return <>
    <Toggle label="Video forwarding" hint="끄면 Video와 host window 하위 설정을 생략합니다." value={v.enabled} disabled={expanded} onChange={enabled => set({ enabled })} />
    <Choice label="Codec" value={v.codec} disabled={off} options={a?.expandedOptions ? ["h264", "h265", "av1", "vp8", "vp9"] : ["h264", "h265", "av1"]} onChange={codec => set({ codec })} />
    <Numeric label="Bitrate · Mbps" min={1} max={2147} value={v.bitrateMbps} disabled={off} onChange={bitrateMbps => set({ bitrateMbps })} />
    <Numeric label="Max FPS" min={1} max={65535} value={v.maxFps} disabled={off} onChange={maxFps => set({ maxFps })} />
    <Numeric label="Max size · px" hint="긴 쪽의 최대 해상도. 0은 제한 없음입니다." max={65535} value={v.maxSize} disabled={more} onChange={maxSize => set({ maxSize })} />
    <Advanced>
      <Numeric label="Android display ID" max={2147483647} value={v.displayId} disabled={more} onChange={displayId => set({ displayId })} />
      <Choice label="Capture orientation" hint="기기에서 캡처하는 방향입니다. host 표시 방향은 Display에서 설정합니다." value={v.captureOrientation.orientation} options={orientations} disabled={more}
        onChange={orientation => set({ captureOrientation: { ...v.captureOrientation, orientation } })} />
      <Toggle label="Lock capture orientation" hint="방향을 지정하지 않고 잠그면 실행 시점의 방향을 유지합니다." value={v.captureOrientation.locked} positiveOnly disabled={more}
        onChange={locked => set({ captureOrientation: { ...v.captureOrientation, locked } })} />
      <Numeric label="Video buffer · ms" hint="지연을 추가하여 프레임 간격을 안정화합니다." max={3600000} value={v.bufferMs} disabled={more} onChange={bufferMs => set({ bufferMs })} />
      <TextSetting label="Encoder" hint="기기에 설치된 Android encoder 이름을 정확히 입력하세요." value={v.encoder} disabled={more} onChange={encoder => set({ encoder })} />
      <Toggle label="Downsize on encoder error" value={v.downsizeOnError} disabled={more} onChange={downsizeOnError => set({ downsizeOnError })} />
      <Choice label="Hardware decoding" hint="scrcpy 5.0 이상이 필요합니다. 실제 지원은 scrcpy build와 GPU에 따라 달라집니다." value={v.hardwareDecoding}
        disabled={!a?.hardwareDecoding} options={a?.hardwareDecoders ?? []} onChange={hardwareDecoding => set({ hardwareDecoding })} />
      <CodecOptions value={v.codecOptions} disabled={more} onChange={codecOptions => set({ codecOptions })} />
    </Advanced>
  </>;
}
