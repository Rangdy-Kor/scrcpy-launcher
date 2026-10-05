# Scrcpy Launcher

Official scrcpy's GUI frontend, built with the existing Tauri 2 / React 19 /
TypeScript / Vite setup. No scrcpy implementation is bundled or modified.

## Run

Install official scrcpy separately and put its native executable on PATH
(`scrcpy.exe` on Windows). ADB must be available as required by that scrcpy
installation. Connect an Android device with USB debugging authorized. Restart
the launcher after changing PATH. Node, pnpm, Rust and Tauri's platform build
prerequisites must also be available in your terminal.

```sh
pnpm tauri dev
```

`pnpm dev` opens the frontend alone; it cannot invoke the Rust backend.
The UI detects installation/version, edits Video / Audio / Input settings,
shows a Rust-generated command preview, and launches scrcpy. The Launch request
stays pending until the scrcpy window closes; settings and Launch are disabled
during that request. Nonzero exits include captured stderr/stdout in the error
panel. Close scrcpy's own window to end the session. Closing the launcher is not
a process termination mechanism in this foundation.

## Configuration and execution

- `src/types/scrcpy.ts`: frontend wire types.
- `src/store/config.ts`: React configuration state and unset defaults.
- `src/lib/tauri.ts`: typed Tauri requests and error normalization.
- `src/App.tsx` / `src/App.css`: small native-control UI.
- `src-tauri/src/config/mod.rs`: serde models, supported enums and error payload.
- `src-tauri/src/config/arguments.rs`: pure validation and argument generation.
- `src-tauri/src/commands/scrcpy.rs`: async Tauri entry points.
- `src-tauri/src/process/scrcpy.rs`: PATH discovery, version query, preview and
  process execution on blocking workers.

React sends a structured `ScrcpyConfig`, never a command string. Both preview
and launch call the same `build_arguments()` function. Launch uses
`Command::new(executable).args(arguments)` directly; the displayed string is
never parsed or executed. Outdated preview responses are discarded and Launch
is only enabled once the current configuration's preview is ready.

Missing/null settings produce no arguments. Audio enabled also produces no
argument because forwarding is scrcpy's default; disabled adds `--no-audio`.
This stage exposes integer Mbps (1–2147) and integer FPS (1–65535), three video
codecs (`h264`, `h265`, `av1`) and input modes (`sdk`, `uhid`, `aoa`, `disabled`).
Zero/overflow values are rejected; arbitrary option strings are not supported.
The FPS range is the foundation model's chosen integer subset, not a claim that
official scrcpy only supports these values.

Example wire configuration:

```json
{
  "video": { "codec": "h265", "bitrateMbps": 20, "maxFps": 60 },
  "audio": { "enabled": true },
  "input": { "keyboard": "uhid", "mouse": "uhid" }
}
```

Produces:

```text
--video-codec=h265 --video-bit-rate=20M --max-fps=60 --keyboard=uhid --mouse=uhid
```

The target syntax was checked against official **scrcpy v3.3.4** documentation:
[video](https://github.com/Genymobile/scrcpy/blob/v3.3.4/doc/video.md),
[audio](https://github.com/Genymobile/scrcpy/blob/v3.3.4/doc/audio.md),
[keyboard](https://github.com/Genymobile/scrcpy/blob/v3.3.4/doc/keyboard.md),
[mouse](https://github.com/Genymobile/scrcpy/blob/v3.3.4/doc/mouse.md), and
[CLI validation](https://github.com/Genymobile/scrcpy/blob/v3.3.4/app/src/cli.c).
Installed versions are reported, but automatic version/capability negotiation
is not implemented. Older versions and device limitations can reject options;
their process errors are surfaced in the UI.

Errors have `{ code, message, details }`. Backend codes include
`invalid_config`, `not_installed`, `version_failed`, `spawn_failed`,
`wait_failed`, `process_failed` and `internal_error`. Tauri transport or typed
deserialization failures are normalized by the frontend as `request_failed`.

## Verify

```sh
pnpm build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo test --manifest-path src-tauri/Cargo.toml --locked
cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings
pnpm tauri build --debug --no-bundle
```

`pnpm build` includes TypeScript checking and Vite production build. There is no
configured frontend lint/format/test runner; no extra dependency was added.
Rust tests cover empty config, video settings, UHID, audio off/on, combined
settings, invalid values, supported enum values, preview/builder consistency,
executable display quoting and empty PATH discovery.

## Next extensions

Add typed settings to the TS/Rust models, their GUI controls, builder mapping
and unit tests. Keep all CLI generation in the Rust builder. The existing empty
ADB/device placeholder files remain available for a later device-discovery
stage; no separate ADB process is launched now (official scrcpy handles ADB).

Wireless pairing, downloads/updates, profiles, instance management, detailed
camera/virtual display/recording settings, the full CLI surface and final UX
are intentionally outside this stage. Live log streaming, a stop command and
launcher-owned process lifecycle can be added later. Currently output is
captured in memory until scrcpy exits.
