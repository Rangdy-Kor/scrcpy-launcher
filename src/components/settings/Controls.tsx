import { useId } from "react";
import type { ReactNode } from "react";
import type { CodecOption } from "../../types/scrcpy";
interface Base { label: string; hint?: string; disabled?: boolean }
export function Setting({ label, hint, children }: Base & { children: (id: string, descriptionId: string | undefined) => ReactNode }) {
  const id = useId();
  return <div className="setting-row"><div><label htmlFor={id}>{label}</label>{hint && <p id={`${id}-hint`} className="hint">{hint}</p>}</div>
    <div className="setting-control">{children(id, hint ? `${id}-hint` : undefined)}</div></div>;
}
export function Choice<T extends string>({ label, hint, disabled, value, options, onChange }: Base & {
  value: T | null; options: readonly (T | readonly [T, string])[]; onChange: (value: T | null) => void;
}) {
  return <Setting label={label} hint={hint}>{(id, desc) => <select id={id} aria-describedby={desc} disabled={disabled} value={value ?? ""}
    onChange={e => onChange((e.target.value || null) as T | null)}><option value="">Default</option>
    {options.map(option => { const [value, title] = typeof option === "string" ? [option, option] : option;
      return <option key={value} value={value}>{title}</option>; })}</select>}</Setting>;
}
export function Toggle({ label, hint, disabled, value, onChange, positiveOnly = false }: Base & {
  value: boolean | null; onChange: (value: boolean | null) => void; positiveOnly?: boolean;
}) {
  return <Choice label={label} hint={hint} disabled={disabled} value={value === null ? null : String(value)}
    options={positiveOnly ? [["true", "Enabled"]] : [["true", "Enabled"], ["false", "Disabled"]]}
    onChange={v => onChange(v === null ? null : v === "true")} />;
}
export function Numeric({ label, hint, disabled, value, min = 0, max, onChange }: Base & {
  value: number | null; min?: number; max: number; onChange: (value: number | null) => void;
}) {
  return <Setting label={label} hint={hint}>{(id, desc) => <input id={id} aria-describedby={desc} type="number" step="1" min={min} max={max}
    disabled={disabled} value={value ?? ""} placeholder="Default" onChange={e => onChange(e.target.value === "" ? null : Number(e.target.value))} />}</Setting>;
}
export function TextSetting({ label, hint, disabled, value, onChange }: Base & { value: string | null; onChange: (value: string | null) => void }) {
  return <Setting label={label} hint={hint}>{(id, desc) => <input id={id} aria-describedby={desc} disabled={disabled} value={value ?? ""}
    placeholder="Default" onChange={e => onChange(e.target.value || null)} />}</Setting>;
}
export function Advanced({ children }: { children: ReactNode }) {
  return <details className="advanced"><summary>Advanced</summary><div>{children}</div></details>;
}
export function CodecOptions({ value, disabled, onChange }: { value: CodecOption[]; disabled: boolean; onChange: (v: CodecOption[]) => void }) {
  function patch(index: number, update: Partial<CodecOption>) { onChange(value.map((entry, i) => i === index ? { ...entry, ...update } : entry)); }
  return <fieldset className="codec-options" disabled={disabled}><legend>Codec options</legend>
    <p className="hint">Android MediaFormat key와 타입별 값을 지정합니다. 지원되는 key는 기기의 encoder에 따라 다릅니다.</p>
    {value.map((entry, i) => <div className="codec-entry" key={i}>
      <input aria-label={`Codec option ${i + 1} key`} placeholder="Key" value={entry.key} onChange={e => patch(i, { key: e.target.value })} />
      <select aria-label={`Codec option ${i + 1} type`} value={entry.type} onChange={e => patch(i, { type: e.target.value as CodecOption["type"] })}>
        {["int", "long", "float", "string"].map(type => <option key={type}>{type}</option>)}</select>
      <input aria-label={`Codec option ${i + 1} value`} placeholder="Value" value={entry.value} onChange={e => patch(i, { value: e.target.value })} />
      <button aria-label={`Remove codec option ${i + 1}`} onClick={() => onChange(value.filter((_, index) => index !== i))}>Remove</button>
    </div>)}
    <button onClick={() => onChange([...value, { key: "", type: "int", value: "" }])}>Add option</button>
  </fieldset>;
}
export const orientations = ["0", "90", "180", "270", "flip0", "flip90", "flip180", "flip270"] as const;
