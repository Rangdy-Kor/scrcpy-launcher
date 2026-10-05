import { useEffect, useRef, useState } from "react";
import { getAdbDevices, getScrcpyStatus, launchScrcpy, previewScrcpy, toLauncherError } from "./lib/tauri";
import { DevicePanel } from "./components/DevicePanel";
import { ErrorMessage } from "./components/ErrorMessage";
import type { AdbStatus } from "./types/device";
import { useScrcpyConfig } from "./store/config";
import type { CommandPreview, InputMode, LauncherError, ScrcpyConfig, ScrcpyStatus, VideoCodec } from "./types/scrcpy";
import "./App.css";

function App() {
  const [config, setConfig] = useScrcpyConfig();
  const [status, setStatus] = useState<ScrcpyStatus | null>(null);
  const [checking, setChecking] = useState(false);
  const [statusError, setStatusError] = useState<LauncherError | null>(null);
  const [preview, setPreview] = useState<{ config: ScrcpyConfig; value: CommandPreview } | null>(null);
  const [previewError, setPreviewError] = useState<LauncherError | null>(null);
  const [launchError, setLaunchError] = useState<LauncherError | null>(null);
  const [launchMessage, setLaunchMessage] = useState("");
  const [launching, setLaunching] = useState(false);
  const [adbStatus, setAdbStatus] = useState<AdbStatus | null>(null);
  const [adbError, setAdbError] = useState<LauncherError | null>(null);
  const [refreshingDevices, setRefreshingDevices] = useState(true);
  const deviceRequestId = useRef(0);

  async function refreshDevices() {
    const requestId = ++deviceRequestId.current;
    setRefreshingDevices(true);
    setAdbError(null);
    try {
      const status = await getAdbDevices();
      if (requestId !== deviceRequestId.current) return;
      setAdbStatus(status);
      if (!status.error) {
        const usable = status.devices.filter((device) => device.state === "device");
        setConfig((previous) => {
          const selected = previous.device.serial;
          if (selected && usable.some((device) => device.serial === selected)) return previous;
          // Preserve ready selections. A vanished/blocked selection is cleared,
          // never silently switched to a different device by a refresh.
          const serial = selected ? null : usable.length === 1 ? usable[0].serial : null;
          return serial === selected ? previous : { ...previous, device: { serial } };
        });
      }
    } catch (error) {
      if (requestId !== deviceRequestId.current) return;
      setAdbStatus(null);
      setAdbError(toLauncherError(error));
    } finally {
      if (requestId === deviceRequestId.current) setRefreshingDevices(false);
    }
  }

  async function refreshStatus() {
    setChecking(true);
    setStatusError(null);
    try { setStatus(await getScrcpyStatus()); }
    catch (error) { setStatus(null); setStatusError(toLauncherError(error)); }
    finally { setChecking(false); }
  }

  useEffect(() => { void refreshStatus(); }, []);
  useEffect(() => { void refreshDevices(); }, []);
  useEffect(() => {
    let cancelled = false;
    setPreviewError(null);
    // Responses for older configs cannot replace the current preview.
    void previewScrcpy(config).then((value) => {
      if (!cancelled) setPreview({ config, value });
    }).catch((error: unknown) => {
      if (!cancelled) setPreviewError(toLauncherError(error));
    });
    return () => { cancelled = true; };
  }, [config]);

  const currentPreview = preview?.config === config ? preview.value : null;
  const selectedDeviceReady = !!config.device.serial &&
    adbStatus?.devices.some((device) => device.serial === config.device.serial && device.state === "device");
  const canLaunch = !launching && !checking && !refreshingDevices && !!status?.installed &&
    !!adbStatus?.installed && !adbStatus.error && !adbError && selectedDeviceReady && !!currentPreview && !previewError;

  async function launch() {
    if (!canLaunch) return;
    setLaunching(true);
    setLaunchError(null);
    setLaunchMessage("Device 상태 확인 및 scrcpy 실행 중입니다. 종료하면 결과가 표시됩니다.");
    try {
      const result = await launchScrcpy(config);
      setLaunchMessage(`scrcpy가 정상 종료되었습니다 (exit ${result.exitCode ?? "unknown"}).`);
    } catch (error) {
      setLaunchMessage("");
      setLaunchError(toLauncherError(error));
    } finally { setLaunching(false); }
  }

  function updateVideo(patch: Partial<ScrcpyConfig["video"]>) {
    setConfig((previous) => ({ ...previous, video: { ...previous.video, ...patch } }));
  }
  function updateInput(key: "keyboard" | "mouse", value: string) {
    setConfig((previous) => ({ ...previous, input: { ...previous.input, [key]: (value || null) as InputMode | null } }));
  }

  return <main className="container">
    <header><h1>Scrcpy Launcher</h1><p>GUI 설정으로 공식 scrcpy를 실행합니다.</p></header>
    <section aria-labelledby="status-title" className="panel">
      <div className="row"><h2 id="status-title">scrcpy 상태</h2>
        <button disabled={checking || launching} onClick={() => void refreshStatus()}>{checking ? "확인 중…" : "다시 확인"}</button></div>
      {status && <><p>{status.installed ? status.version ? `scrcpy ${status.version.raw}` : "설치 감지됨 · 버전 확인 불가" : "PATH에서 scrcpy를 찾을 수 없습니다."}</p>
        {status.executable && <code>{status.executable}</code>}
        {!status.installed && <p>공식 scrcpy를 설치하고 PATH에 추가한 뒤 앱을 다시 시작하세요.</p>}
        {status.error && <ErrorMessage error={status.error} />}</>}
      {status?.installed && !status.version && status.versionOutput && <pre>{status.versionOutput}</pre>}
      {statusError && <ErrorMessage error={statusError} />}
    </section>
    <DevicePanel status={adbStatus} error={adbError} refreshing={refreshingDevices} launching={launching}
      selectedSerial={config.device.serial} onRefresh={() => void refreshDevices()}
      onSelect={(serial) => setConfig((previous) => ({ ...previous, device: { serial } }))} />
    <p className="hint">기본값은 공식 scrcpy에 맡깁니다. 빈 숫자 입력은 옵션을 생략합니다.</p>
    <div className="settings">
      <fieldset disabled={launching}><legend>Video</legend>
        <label>Video codec<select value={config.video.codec ?? ""} onChange={(event) => updateVideo({ codec: (event.target.value || null) as VideoCodec | null })}>
          <option value="">기본값</option><option value="h264">H.264</option><option value="h265">H.265</option><option value="av1">AV1</option></select></label>
        <label>Video bitrate (Mbps)<input type="number" min="1" max="2147" step="1" placeholder="기본값" value={config.video.bitrateMbps ?? ""}
          onChange={(event) => updateVideo({ bitrateMbps: event.target.value === "" ? null : Number(event.target.value) })} /></label>
        <label>Max FPS<input type="number" min="1" max="65535" step="1" placeholder="기본값" value={config.video.maxFps ?? ""}
          onChange={(event) => updateVideo({ maxFps: event.target.value === "" ? null : Number(event.target.value) })} /></label>
      </fieldset>
      <fieldset disabled={launching}><legend>Audio</legend><label>Audio forwarding
        <select value={config.audio.enabled === null ? "" : String(config.audio.enabled)} onChange={(event) => setConfig((previous) => ({ ...previous, audio: { enabled: event.target.value === "" ? null : event.target.value === "true" } }))}>
          <option value="">기본값 (enabled)</option><option value="true">Enabled</option><option value="false">Disabled</option></select></label>
      </fieldset>
      <fieldset disabled={launching}><legend>Input</legend>
        {(["keyboard", "mouse"] as const).map((key) => <label key={key}>{key === "keyboard" ? "Keyboard" : "Mouse"}
          <select value={config.input[key] ?? ""} onChange={(event) => updateInput(key, event.target.value)}>
            <option value="">기본값</option><option value="sdk">SDK</option><option value="uhid">UHID</option><option value="aoa">AOA</option><option value="disabled">Disabled</option></select></label>)}
      </fieldset>
    </div>
    <section className="panel" aria-labelledby="preview-title"><h2 id="preview-title">Command preview</h2>
      <pre aria-live="polite">{currentPreview?.display ?? (previewError ? "설정을 확인하세요." : "생성 중…")}</pre>
      <p className="hint">Rust가 생성한 표시용 preview입니다. 실행은 executable과 arguments 배열을 직접 사용합니다.</p>
      {previewError && <ErrorMessage error={previewError} />}
    </section>
    <button className="launch" disabled={!canLaunch} onClick={() => void launch()}>{launching ? "실행 중…" : "Launch"}</button>
    <p role="status">{launchMessage}</p>
    {launchError && <ErrorMessage error={launchError} />}
  </main>;
}

export default App;
