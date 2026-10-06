import { useEffect, useRef, useState } from "react";
import { getAdbDevices, getScrcpyStatus, launchScrcpy, previewScrcpy, toLauncherError } from "./lib/tauri";
import { DeviceSelector } from "./components/DeviceSelector";
import { ErrorMessage } from "./components/ErrorMessage";
import type { AdbStatus } from "./types/device";
import { createDefaultConfig, useScrcpyConfig } from "./store/config";
import type { ConfigPreview, LauncherError, ScrcpyConfig, ScrcpyStatus } from "./types/scrcpy";
import "./App.css";

import { CommandPreview } from "./components/CommandPreview";
import { GeneralSettings } from "./components/settings/GeneralSettings";
import { VideoSettings } from "./components/settings/VideoSettings";
import { AudioSettings } from "./components/settings/AudioSettings";
import { DisplaySettings } from "./components/settings/DisplaySettings";
import { InputSettings } from "./components/settings/InputSettings";
type Category = "general" | "video" | "audio" | "display" | "input";
const categories: Category[] = ["general", "video", "audio", "display", "input"];
const descriptions: Record<Category, string> = {
  general: "설치된 runtime과 Launcher의 기본 정보를 확인합니다.",
  video: "기기 영상의 품질, 크기와 캡처 동작을 설정합니다.",
  audio: "오디오 캡처, 인코딩과 출력 지연을 설정합니다.",
  display: "Host window와 Android 화면의 동작을 설정합니다.",
  input: "키보드, 마우스와 기기 제어 방식을 설정합니다.",
};
function App() {
  const [category, setCategory] = useState<Category>("general");
  const [config, setConfig] = useScrcpyConfig();
  const [status, setStatus] = useState<ScrcpyStatus | null>(null);
  const [checking, setChecking] = useState(false);
  const [statusError, setStatusError] = useState<LauncherError | null>(null);
  const [preview, setPreview] = useState<{ config: ScrcpyConfig; status: ScrcpyStatus | null; value: ConfigPreview } | null>(null);
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
    const timer = window.setTimeout(() => { void previewScrcpy(config).then((value) => {
      if (!cancelled) setPreview({ config, status, value });
    }).catch((error: unknown) => {
      if (!cancelled) setPreviewError(toLauncherError(error));
    }); }, 120);
    return () => { cancelled = true; window.clearTimeout(timer); };
  }, [config, status]);

  const currentPreview = preview?.config === config && preview.status === status ? preview.value : null;
  const selectedDeviceReady = !!config.device.serial &&
    adbStatus?.devices.some((device) => device.serial === config.device.serial && device.state === "device");
  const canLaunch = !launching && !checking && !refreshingDevices && !!status?.installed &&
    !!adbStatus?.installed && !adbStatus.error && !adbError && selectedDeviceReady && !!currentPreview?.command && !currentPreview.error && !previewError;

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

  function update<K extends Exclude<Category, "general">>(key: K, patch: Partial<ScrcpyConfig[K]>) {
    setConfig(previous => ({ ...previous, [key]: { ...previous[key], ...patch } }));
  }
  const availability = currentPreview?.availability ?? preview?.value.availability ?? null;
  const configError = previewError ?? currentPreview?.error ?? null;
  const deviceError = adbError ?? adbStatus?.error ?? null;
  const launchHint = launching ? launchMessage : refreshingDevices ? "Device 상태 확인 중…" :
    deviceError ? "ADB 상태를 확인하세요." : !selectedDeviceReady ? "상단에서 ready device를 선택하세요." :
    configError ? "설정을 수정한 뒤 실행하세요." : !currentPreview ? "설정 확인 중…" :
    !status?.installed ? "General에서 scrcpy 설치 상태를 확인하세요." : "실행 준비됨";
  return <main className="app-shell">
    <header className="app-bar">
      <div className="brand"><h1>Scrcpy Launcher</h1><span>Product UI v1</span></div>
      <div className="app-actions">
        <DeviceSelector status={adbStatus} selectedSerial={config.device.serial} refreshing={refreshingDevices} disabled={launching}
          onRefresh={() => void refreshDevices()} onSelect={serial => setConfig(previous => ({ ...previous, device: { serial } }))} />
        <button className="launch" disabled={!canLaunch} onClick={() => void launch()}>{launching ? "실행 중…" : "Launch"}</button>
      </div>
    </header>
    <div className="workspace">
      <nav className="sidebar" aria-label="Settings categories"><p className="nav-label">SETTINGS</p>
        {categories.map(key => <button key={key} aria-current={category === key ? "page" : undefined}
          onClick={() => setCategory(key)}>{key[0].toUpperCase() + key.slice(1)}</button>)}
        <div className="sidebar-runtime"><span className={status?.installed ? "ready" : ""}>{status?.version ? "scrcpy " + status.version.raw : "scrcpy 미확인"}</span>
          <small>Target · 5.0</small></div>
      </nav>
      <section className="settings-content" aria-labelledby="category-title" tabIndex={0}>
        <div className="content-heading"><div><h2 id="category-title">{category[0].toUpperCase() + category.slice(1)}</h2><p className="hint">{descriptions[category]}</p></div>
          {category !== "general" && <button disabled={launching} onClick={() => setConfig(previous => ({ ...previous, [category]: createDefaultConfig()[category] }))}>Reset category</button>}
        </div>
        {category !== "general" && <p className="default-note">Default = argument 생략 · 비활성화된 설정 값은 보존됩니다.</p>}
        {category !== "general" && availability && !availability.expandedOptions &&
          <p className="availability-note">확장 설정은 확인된 scrcpy 4.1 이상에서 사용할 수 있습니다. General에서 설치 버전을 확인하세요.</p>}
        <fieldset className="category-fields" disabled={launching}>
          {category === "general" && <GeneralSettings status={status} adb={adbStatus} error={statusError} checking={checking} disabled={launching} onRefresh={() => void refreshStatus()} />}
          {category === "video" && <VideoSettings value={config.video} availability={availability} onChange={patch => update("video", patch)} />}
          {category === "audio" && <AudioSettings value={config.audio} availability={availability} onChange={patch => update("audio", patch)} />}
          {category === "display" && <DisplaySettings value={config.display} availability={availability} onChange={patch => update("display", patch)} />}
          {category === "input" && <InputSettings value={config.input} availability={availability} onChange={patch => update("input", patch)} />}
        </fieldset>
      </section>
    </div>
    <div className="feedback"><p role="status">{launchHint}</p>
      {!launching && launchMessage && <p role="status">{launchMessage}</p>}
      {deviceError && <ErrorMessage error={deviceError} />}
      {configError && <ErrorMessage error={configError} />}
      {launchError && <ErrorMessage error={launchError} />}
    </div>
    <CommandPreview value={currentPreview?.command ?? null} pending={!currentPreview && !previewError} invalid={!!configError} />
  </main>;
}
export default App;
