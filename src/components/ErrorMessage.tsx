import type { LauncherError } from "../types/scrcpy";

export function ErrorMessage({ error }: { error: LauncherError }) {
  return (
    <div className="error" role="alert">
      <strong>{error.message}</strong>
      <small>{error.code}</small>
      {error.details && <pre>{error.details}</pre>}
    </div>
  );
}
