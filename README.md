# Scrcpy Launcher

Official scrcpy's GUI frontend, built with the existing Tauri 2 / React 19 /
TypeScript / Vite setup. No scrcpy implementation is bundled or modified.

## Run

Install official scrcpy separately and put its native executable on PATH
(`scrcpy.exe` on Windows). ADB is searched in PATH first, then beside the resolved
scrcpy executable (the official portable distribution may bundle it there).
No winget/package-manager installation path is hardcoded. Connect an Android
device with USB debugging authorized. Restart
the launcher after changing PATH. Node, pnpm, Rust and Tauri's platform build
prerequisites must also be available in your terminal.

```sh
pnpm tauri dev
```

`pnpm dev` opens the frontend alone; it cannot invoke the Rust backend.
The UI detects installation/version, lists ADB devices, selects a serial,
edits Video / Audio / Input settings,
shows a Rust-generated command preview, and launches scrcpy. The Launch request
stays pending until the scrcpy window closes; settings and Launch are disabled
during that request. Nonzero exits include captured stderr/stdout in the error
panel. Close scrcpy's own window to end the session. Closing the launcher is not
a process termination mechanism in this foundation.

## Configuration and execution

- `src/types/scrcpy.ts`: frontend wire types.
- `src/types/device.ts`: ADB status/device wire types.
- `src/store/config.ts`: React configuration state and unset defaults.
- `src/lib/tauri.ts`: typed Tauri requests and error normalization.
- `src/App.tsx` / `src/App.css`: small native-control UI.
- `src/components/DevicePanel.tsx`: device list, state and serial selection.
- `src/components/ErrorMessage.tsx`: shared structured-error display.
- `src-tauri/src/config/mod.rs`: serde models, supported enums and error payload.
- `src-tauri/src/config/arguments.rs`: pure validation and argument generation.
- `src-tauri/src/commands/scrcpy.rs`: async Tauri entry points.
- `src-tauri/src/commands/adb.rs`: async ADB discovery/list entry point.
- `src-tauri/src/process/mod.rs`: shared native executable discovery/command setup.
- `src-tauri/src/process/scrcpy.rs`: version query, preview, launch orchestration
  and process execution on blocking workers.
- `src-tauri/src/process/adb.rs`: ADB resolution, pure device parser, query and
  pure runtime-snapshot selection validation.
- `src-tauri/src/scrcpy/version.rs`: structured version parser/comparison.
- `src-tauri/src/scrcpy/capabilities.rs`: centralized version requirements.

React sends a structured `ScrcpyConfig`, never a command string. Both preview
and launch call the same `build_arguments()` function. Launch uses
`Command::new(executable).args(arguments)` directly; the displayed string is
never parsed or executed. Outdated preview responses are discarded and Launch
is only enabled once the current configuration's preview is ready.
Device selection is `{ "device": { "serial": "..." } }`; the builder produces
one literal `--serial=<serial>` argument. It validates the serial's format but
never queries ADB. Spaces/metacharacters cannot become shell syntax because
there is no shell; the display quotes such tokens without changing the vector.

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
  "device": { "serial": "192.168.1.188:42999" },
  "video": { "codec": "h265", "bitrateMbps": 20, "maxFps": 60 },
  "audio": { "enabled": true },
  "input": { "keyboard": "uhid", "mouse": "uhid" }
}
```

Produces:

```text
--serial=192.168.1.188:42999 --video-codec=h265 --video-bit-rate=20M --max-fps=60 --keyboard=uhid --mouse=uhid
```

The latest official stable was verified on **2026-10-05** as **scrcpy v5.0** via
the [official latest release](https://github.com/Genymobile/scrcpy/releases/latest).
The target syntax was rechecked against tag-pinned official documentation:
[video](https://github.com/Genymobile/scrcpy/blob/v5.0/doc/video.md),
[audio](https://github.com/Genymobile/scrcpy/blob/v5.0/doc/audio.md),
[keyboard](https://github.com/Genymobile/scrcpy/blob/v5.0/doc/keyboard.md),
[mouse](https://github.com/Genymobile/scrcpy/blob/v5.0/doc/mouse.md),
[device selection](https://github.com/Genymobile/scrcpy/blob/v5.0/doc/connection.md),
and [CLI validation](https://github.com/Genymobile/scrcpy/blob/v5.0/app/src/cli.c).
Existing options/values remain valid; v5.0 additionally supports VP8/VP9, but
these/new v5.0 options are intentionally not added to the GUI in Foundation 2.
`--serial`, `--select-usb` and `--select-tcpip` remain official selection syntax;
this GUI uses explicit serial selection.

## Version and capability foundation

`ScrcpyStatus.version` is now `{ major, minor, patch, raw }` or null. The parser
reads only the official `scrcpy VERSION <...>` banner, not dependency versions.
`4.1` and `5.0` have patch 0; patch versions such as `3.3.4` are preserved.
`raw` retains the original version token. Numeric tuple comparisons handle
`4.10 > 4.2` correctly. `versionOutput` also retains the command output.
Unknown/malformed/development-suffixed output reports `version_failed` and
unknown capabilities without preventing other app functions or Launch.

Rust `scrcpy/capabilities.rs` owns minimum-version requirements through
`requires_version()`. Status returns a typed capability snapshot with
`supported`, `unsupported` or `unknown`. The first example is the `--hwdec`
CLI introduced in [v5.0](https://github.com/Genymobile/scrcpy/releases/tag/v5.0):
`hardwareDecoding` requires `(5, 0, 0)`. No hardware-decoding control is exposed.
This describes version-based CLI availability, not platform/Android/encoder
support. Future GUI controls should consume this backend snapshot, rather than
scatter version comparisons through React. Existing controls are unchanged;
complete historical capability checks are not implemented.

## Device discovery and launch validation

Rust resolves only native `adb.exe` / `adb` binaries, using PATH before an
immediate sibling of the resolved scrcpy executable. It does not recurse through
system/package directories, execute `.cmd`/`.bat`, or download tools.
Missing ADB and failed queries return structured status/errors.

`adb devices -l` execution and parsing are separate. The parser requires the
device-list header and accepts CRLF, tabs/spaces and optional metadata. Rows
preserve exact serial/state, model/product/device, numeric transport ID and USB
metadata where present. Missing values remain null; unknown states are retained.
Only state `device` is launchable. `unauthorized`, `offline`, `no permissions`
and recovery/other states stay visible but cannot be selected for mirroring.

Connection labels use explicit `usb:` metadata, IP:port serials, the wireless
debugging `_adb-tls-connect._tcp` suffix, or the emulator serial prefix. Other
serials have connection `unknown`; absence of an IP does not prove USB. Models
are labels only. Two serials for the same physical/model device are never merged.

The UI auto-selects when exactly one ready device exists. Multiple ready devices
require an explicit selection. Refresh preserves a still-ready selection; a
missing/blocked selection is cleared without switching to another device during
that refresh. Query errors and refresh-in-progress block Launch. The backend
queries devices again immediately before spawning scrcpy, then rejects:

- no ready device;
- multiple transports with no selected serial (also one ready plus one offline);
- a selected serial no longer present;
- a selected device whose state is not `device`;
- duplicate rows sharing the selected serial, which `--serial` cannot distinguish.

For backward compatibility, an unset serial is valid in the pure builder and
backend Launch may proceed with exactly one ready row. It does not silently
insert a serial or alter the preview. The GUI auto-selection explicitly updates
the config, so GUI launches carry `--serial` even in the single-device case.
ADB changes after the validation snapshot remain possible; scrcpy's resulting
process errors are still reported.

The child receives the discovered ADB path through official scrcpy's `ADB`
environment variable, so query and mirroring use the same binary. The child's
inherited `ANDROID_SERIAL` is removed so config remains the selection authority.
Other ADB server environment settings are inherited by both processes.

Errors have `{ code, message, details }`. Backend codes include
`invalid_config`, `not_installed`, `version_failed`, `spawn_failed`,
`wait_failed`, `process_failed`, `adb_not_installed`, `adb_query_failed`,
`adb_parse_failed`, `no_usable_devices`, `device_selection_required`,
`device_not_found`, `device_unavailable`, `ambiguous_device` and `internal_error`.
Tauri transport or typed
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
All 12 Foundation 1 tests remain. Foundation 2 adds version parsing/comparison,
capability threshold/unknown handling, ADB parser states/transports/metadata,
discovery precedence/fallback, runtime selection validation, device arguments,
literal argument safety, preview consistency and Windows path JSON roundtrip.
The normal Rust suite has 37 passing tests plus one ignored live discovery test.
The live smoke test also passed against installed scrcpy 4.1 and two ready ADB
transports; it checks generated serial arguments and rejection of an unselected
multi-device Launch without opening a mirroring window. Run it deliberately:

```sh
cargo test --manifest-path src-tauri/Cargo.toml --locked live_discovery_smoke -- --ignored --nocapture
```

The smoke test prints local executable paths/device identifiers. Real selected
device mirroring and visual GUI behavior still need a desktop/device smoke test.

## Windows path display

Native Windows canonicalization may produce an extended-length `\\?\C:\...`
path. That prefix is valid and is preserved. Actual discovery JSON and a Unicode
path roundtrip test retain ordinary U+005C backslashes; React renders the strings
directly. No separator replacement or path rewriting was introduced.
`pre`/`code` now explicitly use Cascadia Mono / Consolas / system monospace fonts
and disable ligatures to avoid a locale/font fallback rendering backslash as a
different glyph. The old UI glyph itself was not reproduced in this session, so
the font explanation remains a rendering diagnosis rather than a measured
pixel-level confirmation.

## Next extensions

Add typed settings to the TS/Rust models, their GUI controls, builder mapping
and unit tests. Keep all CLI generation in the Rust builder and minimum-version
conditions in the backend capability module. ADB/device placeholder files are
now implemented; device parsing/validation is independent of the builder.

Process lifecycle review: `spawn_scrcpy()` returns a `Child` separately from the
current wait/result handling, so a later registry can own the Child/pipes without
changing config/version/device logic. Launch still waits until exit on a blocking
worker, captures output in memory, and does not own children across app shutdown.
Next prioritize a bounded live-log stream, registry/state, Stop and an explicit
launcher-exit policy together. Introducing those requires replacing
`wait_with_output()` with coordinated pipe readers and child ownership; merely
adding a Stop button to the current wait path would not be sufficient.

Wireless pairing, downloads/updates, profiles, instance management, detailed
camera/virtual display/recording settings, the full CLI surface and final UX
are intentionally outside this stage. Pair/connect workflows, arbitrary custom
arguments, full v5.0 option UI and a full capability engine are also excluded.
No new dependency or frontend lint framework was added.
