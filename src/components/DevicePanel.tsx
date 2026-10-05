import type { AdbConnection, AdbStatus } from "../types/device";
import type { LauncherError } from "../types/scrcpy";
import { ErrorMessage } from "./ErrorMessage";

const connectionLabels: Record<AdbConnection, string> = {
  usb: "USB",
  tcpIp: "TCP/IP",
  wirelessDebugging: "Wireless Debugging / mDNS",
  emulator: "Emulator",
  unknown: "연결 방식 미확인",
};

interface DevicePanelProps {
  status: AdbStatus | null;
  error: LauncherError | null;
  refreshing: boolean;
  launching: boolean;
  selectedSerial: string | null;
  onRefresh: () => void;
  onSelect: (serial: string | null) => void;
}

export function DevicePanel({ status, error, refreshing, launching, selectedSerial, onRefresh, onSelect }: DevicePanelProps) {
  const devices = status?.devices ?? [];
  const usableCount = devices.filter((device) => device.state === "device").length;
  return (
    <section aria-labelledby="device-title" className="panel">
      <div className="row">
        <h2 id="device-title">Device</h2>
        <button disabled={refreshing || launching} onClick={onRefresh}>
          {refreshing ? "조회 중…" : "목록 새로고침"}
        </button>
      </div>
      {status?.executable && <p className="hint">ADB ({status.source === "path" ? "PATH" : "scrcpy 설치 폴더"}): <code>{status.executable}</code></p>}
      {status?.error && <ErrorMessage error={status.error} />}
      {error && <ErrorMessage error={error} />}
      {status && !status.error && (
        <>
          <label>실행할 serial / transport
            <select disabled={launching || refreshing || usableCount === 0} value={selectedSerial ?? ""}
              onChange={(event) => onSelect(event.target.value || null)}>
              <option value="">{usableCount === 0 ? "실행 가능한 device 없음" : "Device를 선택하세요"}</option>
              {devices.map((device, index) => (
                <option key={`${device.serial}-${device.transportId}-${index}`} value={device.serial} disabled={device.state !== "device"}>
                  {device.model ? `${device.model} · ` : ""}{device.serial} · {connectionLabels[device.connection]} · {device.state}
                </option>
              ))}
            </select>
          </label>
          <p className="hint">{usableCount === 0 ? "device 상태가 ready인지 확인하세요. unauthorized는 기기에서 USB 디버깅을 허용해야 합니다." : "같은 기기의 여러 serial도 각각 별도의 실행 대상입니다."}</p>
          {devices.length > 0 && (
            <ul className="device-list">
              {devices.map((device, index) => (
                <li key={`${device.serial}-${device.transportId}-${index}`}>
                  <span className={device.state === "device" ? "device-ready" : "device-unavailable"}>{device.state}</span>
                  {device.model && <strong>{device.model}</strong>}
                  <code>{device.serial}</code>
                  <small>{connectionLabels[device.connection]}{device.transportId !== null ? ` · transport ${device.transportId}` : ""}</small>
                </li>
              ))}
            </ul>
          )}
        </>
      )}
    </section>
  );
}
