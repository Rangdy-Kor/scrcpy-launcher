import { useRef } from "react";
import type { AdbConnection, AdbStatus } from "../types/device";
export const connectionLabels: Record<AdbConnection, string> = {
  usb: "USB", tcpIp: "TCP/IP", wirelessDebugging: "Wireless Debugging / mDNS", emulator: "Emulator", unknown: "Transport 미확인",
};
interface Props {
  status: AdbStatus | null; refreshing: boolean; disabled: boolean; selectedSerial: string | null;
  onRefresh: () => void; onSelect: (serial: string | null) => void;
}
export function DeviceSelector({ status, refreshing, disabled, selectedSerial, onRefresh, onSelect }: Props) {
  const popup = useRef<HTMLDetailsElement>(null);
  const selected = status?.devices.find(d => d.serial === selectedSerial);
  function select(serial: string | null) { onSelect(serial); if (popup.current) { popup.current.open = false; popup.current.querySelector("summary")?.focus(); } }
  return <div className="device-selector">
    <details ref={popup} onKeyDown={e => { if (e.key === "Escape" && popup.current) { popup.current.open = false; popup.current.querySelector("summary")?.focus(); } }}>
      <summary aria-label="실행할 device 선택"><span>{selected ? `${selected.model ?? "Android"} · ${connectionLabels[selected.connection]}` : "Device 선택"}</span></summary>
      <div className="device-popup">
        <div className="popup-heading"><strong>실행 대상</strong><span>{status?.devices.length ?? 0} transports</span></div>
        <p className="hint">같은 기기의 다른 serial도 별개의 실행 대상입니다.</p>
        <button className="device-option" disabled={disabled || refreshing} onClick={() => select(null)}>선택 해제</button>
        {status?.devices.map((d, i) => <button className={`device-option ${d.serial === selectedSerial ? "selected" : ""}`} key={`${d.serial}-${d.transportId}-${i}`}
          disabled={disabled || refreshing || d.state !== "device"} aria-pressed={d.serial === selectedSerial} onClick={() => select(d.serial)}>
          <span className="device-name"><strong>{d.model ?? "Android device"}</strong><span className={d.state === "device" ? "ready" : "blocked"}>{d.state}</span></span>
          <code>{d.serial}</code><small>{connectionLabels[d.connection]}{d.transportId !== null ? ` · transport ${d.transportId}` : ""}</small>
        </button>)}
        {!status?.devices.length && <p className="hint">{refreshing ? "Device 목록 조회 중…" : "Device가 없습니다. 연결과 디버깅 허용 상태를 확인하세요."}</p>}
        {status?.devices.some(d => d.state === "unauthorized") && <p className="hint">Unauthorized: Android 기기에서 디버깅 연결을 허용하세요.</p>}
      </div>
    </details>
    <button disabled={disabled || refreshing} onClick={onRefresh} aria-label="Device 목록 새로고침">{refreshing ? "조회 중…" : "Refresh"}</button>
  </div>;
}
