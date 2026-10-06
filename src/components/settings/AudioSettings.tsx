import { Advanced, Choice, CodecOptions, Numeric, TextSetting, Toggle } from "./Controls";
import type { AudioSource } from "../../types/scrcpy";
import type { SettingsProps } from "./types";
const sources: AudioSource[] = ["output", "playback", "mic", "mic-unprocessed", "mic-camcorder", "mic-voice-recognition", "mic-voice-communication", "voice-call", "voice-call-uplink", "voice-call-downlink", "voice-performance"];
export function AudioSettings({ value: v, onChange: set, availability: a }: SettingsProps<"audio">) {
  const more = !a?.expandedOptions || !a?.audio;
  return <>
    <Toggle label="Audio forwarding" hint="끄면 하위 설정을 보존하고 arguments에서 제외합니다." value={v.enabled} onChange={enabled => set({ enabled })} />
    <Choice label="Source" hint="Android 버전과 앱의 캡처 정책에 따라 사용 가능한 source가 다릅니다." value={v.source} options={sources} disabled={more} onChange={source => set({ source })} />
    <Choice label="Codec" value={v.codec} options={["opus", "aac", "flac", "raw"]} disabled={more} onChange={codec => set({ codec })} />
    <Numeric label="Bitrate · Kbps" hint="Raw와 FLAC에서는 생략합니다." min={1} max={2147483} value={v.bitrateKbps} disabled={more || !a?.audioBitrate} onChange={bitrateKbps => set({ bitrateKbps })} />
    <Toggle label="Duplicate audio" hint="기기에서도 소리를 재생합니다. Default source는 playback으로 전환되며 다른 명시적 source에서는 생략합니다." value={v.duplication} positiveOnly disabled={more || !a?.audioDuplication} onChange={duplication => set({ duplication })} />
    <Advanced>
      <Numeric label="Audio buffer · ms" max={3600000} value={v.bufferMs} disabled={more} onChange={bufferMs => set({ bufferMs })} />
      <Numeric label="Output buffer · ms" hint="Host 오디오 출력 buffer입니다. 너무 작으면 끊길 수 있습니다." max={1000} value={v.outputBufferMs} disabled={more} onChange={outputBufferMs => set({ outputBufferMs })} />
      <TextSetting label="Encoder" hint="Raw에서는 encoder와 codec options를 생략합니다." value={v.encoder} disabled={more || !a?.audioEncoding} onChange={encoder => set({ encoder })} />
      <Toggle label="Require audio" hint="오디오 캡처가 실패하면 scrcpy 실행을 종료합니다." value={v.requireAudio} positiveOnly disabled={more} onChange={requireAudio => set({ requireAudio })} />
      <CodecOptions value={v.codecOptions} disabled={more || !a?.audioEncoding} onChange={codecOptions => set({ codecOptions })} />
    </Advanced>
  </>;
}
