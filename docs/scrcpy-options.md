# Product UI v1 option inventory

Target checked on 2026-10-05: latest official stable **scrcpy 5.0**.
Sources: [release](https://github.com/Genymobile/scrcpy/releases/tag/v5.0),
[v5.0 CLI parser](https://github.com/Genymobile/scrcpy/blob/v5.0/app/src/cli.c),
[v4.1 CLI parser](https://github.com/Genymobile/scrcpy/blob/v4.1/app/src/cli.c).
The parser is authoritative where help wording is ambiguous (for example,
shortcut modifiers accept comma-separated alternatives, not `+` combinations).

All implemented controls have explicit TS/Rust types, a structured Tauri payload,
Rust validation/builder mapping, GUI availability and family-level unit tests.
No arbitrary CLI field is provided. `Default`/empty input means omission, not a
snapshot of the target version's defaults. Boolean settings with only a positive
CLI flag offer Default/Enabled; negative flags offer Default/Enabled/Disabled,
with the default-equivalent choice omitted. Existing Foundation integer FPS and
whole Mbps models remain intentional subsets of the official syntax.

## Implemented inventory

Except `--hwdec`, these additions were also checked in official v4.1. The backend
uses **4.1 as a conservative verified support baseline**, not as the historical
introduction version of every option. Older/unknown versions retain Foundation
controls but cannot activate the newly supported arguments. Full historical and
Android/build/encoder capability discovery is deferred.

### Video

Official [video documentation](https://github.com/Genymobile/scrcpy/blob/v5.0/doc/video.md).

| GUI setting | CLI argument | Model / availability |
| --- | --- | --- |
| Video forwarding | `--no-video` | Explicit disabled only; children and host window settings omitted |
| Codec | `--video-codec=` | h264, h265, av1, vp8, vp9; last two were already in v4.1 |
| Bitrate | `--video-bit-rate=…M` | Whole Mbps, 1–2147 |
| Max FPS | `--max-fps=` | Foundation integer subset 1–65535 |
| Max size | `--max-size=` | 0–65535 pixels; 0 removes size limit |
| Android display | `--display-id=` | 0–2147483647; device existence checked by scrcpy |
| Capture orientation / lock | `--capture-orientation=` | Eight quarter-turn/flip values; optional `@`; `@` alone locks initial orientation |
| Encoder | `--video-encoder=` | Nonempty native Android encoder name, one literal argument |
| Buffer | `--video-buffer=` | 0–3600000 ms |
| Downsize on failure | `--no-downsize-on-error` | Disabled only |
| Hardware decoding | `--hwdec=` | **>=5.0**; auto/disabled + native OS backend (d3d11va / vaapi / videotoolbox) |
| Codec options | `--video-codec-options=` | Structured key/type/value entries |

Hardware decoding denotes host decoding, not Android encoding. Version/platform
availability cannot guarantee a particular scrcpy build or GPU supports the
backend. Default leaves scrcpy's own auto-selection/fallback policy intact.

### Audio

Official [audio documentation](https://github.com/Genymobile/scrcpy/blob/v5.0/doc/audio.md).

| GUI setting | CLI argument | Model / availability |
| --- | --- | --- |
| Audio forwarding | `--no-audio` | Disabled only; preserves all inactive children |
| Source | `--audio-source=` | output, playback, mic, mic-unprocessed, mic-camcorder, mic-voice-recognition, mic-voice-communication, voice-call, voice-call-uplink, voice-call-downlink, voice-performance |
| Codec | `--audio-codec=` | opus, aac, flac, raw |
| Bitrate | `--audio-bit-rate=…K` | Whole Kbps 1–2147483; omitted for raw/flac |
| Duplicate audio | `--audio-dup` | Default source or playback only; Default source is switched to playback by official scrcpy |
| Encoder | `--audio-encoder=` | Omitted for raw |
| Buffer | `--audio-buffer=` | 0–3600000 ms |
| Output buffer | `--audio-output-buffer=` | 0–1000 ms |
| Require audio | `--require-audio` | Positive flag; scrcpy also implies this when no video is forwarded |
| Codec options | `--audio-codec-options=` | Structured entries; omitted for raw |

Audio source availability on Android and app capture policies remain scrcpy's
responsibility; the launcher does not infer them from ADB metadata.

### Display

Official [window](https://github.com/Genymobile/scrcpy/blob/v5.0/doc/window.md),
[device](https://github.com/Genymobile/scrcpy/blob/v5.0/doc/device.md), and video docs.

| Group | CLI arguments | Model / availability |
| --- | --- | --- |
| Host presentation | `--fullscreen`, `--always-on-top`, `--window-borderless` | Positive flags; require video |
| Title | `--window-title=` | Literal nonempty text; require video |
| Position | `--window-x=`, `--window-y=` | -32767–32767; blank leaves automatic positioning |
| Size | `--window-width=`, `--window-height=` | 0–65535; 0 uses automatic calculation |
| Host orientation | `--display-orientation=` | Eight quarter-turn/flip values; separate from capture orientation |
| Fit | `--render-fit=` | letterbox, stretched, unscaled |
| Aspect ratio lock | `--no-window-aspect-ratio-lock` | Disabled only |
| Host screensaver | `--disable-screensaver` | Positive flag |
| Android screen/power | `--turn-screen-off`, `--stay-awake`, `--power-off-on-close`, `--no-power-on` | Control required; no-power-on also omitted when video is off |

`stay-awake` applies while the Android device is plugged in, including while
wireless ADB is being used. No transport-based charging assumption is made.

### Input

Official [keyboard](https://github.com/Genymobile/scrcpy/blob/v5.0/doc/keyboard.md),
[mouse](https://github.com/Genymobile/scrcpy/blob/v5.0/doc/mouse.md),
[shortcuts](https://github.com/Genymobile/scrcpy/blob/v5.0/doc/shortcuts.md).

| GUI setting | CLI argument | Model / availability |
| --- | --- | --- |
| Control | `--no-control` | Disables input children and conflicting Android power actions |
| Keyboard | `--keyboard=` | Separate KeyboardMode: sdk, uhid, aoa, disabled |
| Mouse | `--mouse=` | Separate MouseMode: sdk, uhid, aoa, disabled |
| Text injection | `--prefer-text` / `--raw-key-events` | One typed choice: Default/mixed/text/raw; SDK only; mutually exclusive by construction |
| Key repeat | `--no-key-repeat` | SDK keyboard only |
| Hover | `--no-mouse-hover` | SDK mouse only |
| Clipboard auto-sync | `--no-clipboard-autosync` | Control required |
| Physical touch indicator | `--show-touches` | Control required; Android physical touches, not injected clicks |
| Shortcut modifiers | `--shortcut-mod=` | Unique subset of lctrl/rctrl/lalt/ralt/lsuper/rsuper; comma-separated alternatives; host shortcuts remain available without control |

AOA is USB-only and build-dependent. Official Windows CLI rejects AOA without
OTG; this launcher does not implement OTG, so Windows availability excludes AOA
and runtime validation rejects forged/stale AOA payloads. UHID works through ADB
including wireless transports, subject to Android support.

## Reviewed and deliberately deferred

| Category | Official options reviewed | Reason for deferral |
| --- | --- | --- |
| Video | `--video-source`, `--camera-*`, `--list-cameras`, `--list-camera-sizes` | display capture is current implicit source; camera needs its own coherent workflow |
| Video | `--crop`, `--angle`, `--min-size-alignment`, `--ignore-video-encoder-constraints` | specialized geometry/encoder tuning; add structured controls with focused validation later |
| Video | `--list-encoders`, `--list-displays` | encoder/display discovery is useful next; current names/IDs are typed and validated structurally but scrcpy validates device availability |
| Video/output | `--no-video-playback`, `--no-playback`, `--v4l2-sink`, `--v4l2-buffer`, `--record*` | output routing, Linux sinks and recording belong to future workflows |
| Audio | `--no-audio-playback`, `--no-playback` | capture-without-playback primarily needs recording/output routing, excluded this stage |
| Display | `--no-window`, `--no-terminal-title`, `--background-color`, `--render-driver`, `--no-mipmaps` | headless lifecycle/recording or less common host rendering tuning |
| Display | `--new-display`, `--flex-display`, `--no-vd-*`, `--display-ime-policy` | virtual/flex display workflow excluded; flex display introduced in v5.0 |
| Device power | `--keep-active`, `--screen-off-timeout` | additional device policy controls beyond the chosen common power actions |
| Input | `--mouse-bind`, `--legacy-paste`, `--gamepad`, `--otg`, `--push-target`, `--start-app`, `--list-apps` | specialized bindings, gamepads, USB-only control, file/app workflows need focused UI and validation |
| Connection/application | pairing/connect, `--tcpip`, tunnel/port, cleanup/ADB lifecycle, logging/time limit | dedicated connection/process lifecycle stage |

No empty Camera/Recording/Connection menus, custom arguments, generic DSL,
code generation framework, persistence or new dependency were introduced.

## Domain and process extension points

`config/availability.rs::dependencies()` defines active parent/source/mode
relationships once for both argument omission and GUI flags. `availability()`
adds centralized version/platform restrictions. `validate_runtime_arguments()`
checks the produced vector for preview and launch; it generates nothing and does
not query ADB. Config conflicts/ranges are validated in the pure builder. Stored
inactive children are omitted, not rejected or erased. Disabling video, audio
and control together returns `option_conflict`.

`preview_scrcpy` returns `{ command, availability, error }`, including availability
even on invalid config so users can correct it. Preview is debounced and stale
responses cannot authorize Launch. Launch rebuilds and rechecks capabilities and
ADB state immediately before `spawn_scrcpy()`. Both use `build_arguments()`;
there is no frontend argument generation. This lifecycle is intentionally still
wait-until-exit. A later process registry can take ownership at `spawn_scrcpy()`
and replace the `wait_with_output()` collector with bounded pipe readers,
instance state, Stop and an explicit exit policy.
