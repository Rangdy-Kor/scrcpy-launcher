import type { CommandPreview as Preview } from "../types/scrcpy";
export function CommandPreview({ value, pending, invalid }: { value: Preview | null; pending: boolean; invalid: boolean }) {
  return <details className="command-preview"><summary><span>Command Preview</span><small>{pending ? "확인 중…" : invalid ? "설정 확인 필요" : "Rust argument builder"}</small></summary>
    <div><pre>{value?.display ?? (pending ? "생성 중…" : "현재 설정의 오류를 먼저 수정하세요.")}</pre>
      <p className="hint">실행할 executable과 argument 배열의 표시입니다. Default는 argument를 생성하지 않습니다.</p>
    </div></details>;
}
