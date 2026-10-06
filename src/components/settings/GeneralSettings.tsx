import type { AdbStatus } from "../../types/device";
import type { LauncherError, ScrcpyStatus } from "../../types/scrcpy";
import { ErrorMessage } from "../ErrorMessage";
export function GeneralSettings({ status, adb, error, checking, disabled, onRefresh }: {
  status: ScrcpyStatus | null; adb: AdbStatus | null; error: LauncherError | null; checking: boolean; disabled: boolean; onRefresh: () => void;
}) {
  return <div className="general-settings">
    <div className="section-heading"><h3>scrcpy runtime</h3><button disabled={disabled || checking} onClick={onRefresh}>{checking ? "확인 중…" : "다시 확인"}</button></div>
    <p className="runtime-version">{!status ? error ? "Runtime 조회 실패" : "설치 확인 중…" : !status.installed ? "scrcpy를 찾을 수 없습니다" : status.version ? `scrcpy ${status.version.raw}` : "설치 감지됨 · 버전 확인 불가"}</p>
    <dl><dt>Executable</dt><dd><code>{status?.displayExecutable ?? "PATH에서 공식 scrcpy를 탐지합니다."}</code></dd>
      <dt>Target specification</dt><dd>scrcpy 5.0 · 설치된 버전에 따라 기능이 활성화됩니다.</dd>
      <dt>Hardware decoding</dt><dd>{status?.capabilities.hardwareDecoding === "supported" ? "지원 · Video → Advanced" : "scrcpy 5.0 이상 필요"}</dd></dl>
    {!status?.installed && <p className="hint">공식 scrcpy를 설치하고 PATH에 추가한 뒤 앱을 다시 시작하세요.</p>}
    {status?.error && <ErrorMessage error={status.error} />}{error && <ErrorMessage error={error} />}
    {status?.installed && !status.version && status.versionOutput && <pre>{status.versionOutput}</pre>}
    <h3>ADB</h3><dl><dt>Executable</dt><dd><code>{adb?.displayExecutable ?? "탐지된 ADB 없음"}</code></dd>
      <dt>Discovery</dt><dd>PATH 우선, 이후 scrcpy 인접 폴더 · {adb?.source === "path" ? "PATH" : adb?.source === "scrcpyDirectory" ? "scrcpy 폴더" : "미확인"}</dd></dl>
    <h3>설정 안내</h3><p className="hint">Default 또는 빈 입력은 설치된 scrcpy의 기본 동작에 맡깁니다. 비활성화된 하위 설정은 보존되고 실행 arguments에서는 제외됩니다.</p>
    <p className="hint">설정은 이번 실행 세션에만 적용됩니다. scrcpy 창을 닫으면 Launcher에서 종료 결과를 확인할 수 있습니다.</p>
  </div>;
}
