# Week 4: Input Channel (Keyboard + Mouse over WebRTC DataChannel)

## Context

Week 3 is complete: browser receives desktop video from `ffmpeg-bot` through Rust WebRTC.

Week 4 goal is to add interactivity:
- Browser captures keyboard + mouse input.
- Input is sent over WebRTC DataChannel.
- Rust receives events and injects them into Windows using `SendInput`.

This week intentionally excludes:
- Gamepad support
- Fallback transport (no WebSocket fallback)
- Fine-grained security controls (PoC trust model)

## Week 4 Goal

From a desktop browser on the same LAN, the user can:
- move mouse on streamed video
- click (left/right)
- scroll
- type with keyboard

and see those actions reflected on the host Windows desktop in real time.

## Scope and Decisions (Locked)

- Input devices: keyboard + mouse only
- Client: desktop browser
- Network: same LAN
- Transport: WebRTC DataChannel only
- Control policy: direct full input (no restrictions)
- Security: basic room/session behavior only (PoC-level)
- Coordinate model: pointer on remote video maps to equivalent host screen location

## Architecture

1. Browser joins signaling session and calls `ffmpeg-bot` (already in Week 3).
2. During peer setup, Rust (`ffmpeg-bot`) creates a DataChannel for control (for example `input`).
3. Browser waits for that DataChannel (`pc.ondatachannel`) and binds event handlers.
4. Browser serializes keyboard/mouse events to compact JSON messages.
5. Rust parses messages and injects events via Windows `SendInput`.

## Suggested Input Message Protocol (v1)

Use one JSON envelope to keep parsing simple:

```json
{ "v": 1, "kind": "mouse_move_abs", "x_norm": 0.53, "y_norm": 0.27 }
```

Recommended event kinds:

- Mouse:
  - `mouse_move_abs` with normalized `x_norm`, `y_norm` in range `[0.0, 1.0]`
  - `mouse_down` with `button` (`left`, `right`, `middle`)
  - `mouse_up` with `button`
  - `mouse_wheel` with `delta_y` (positive/negative wheel ticks)
- Keyboard:
  - `key_down` with `code` (from browser KeyboardEvent `code`)
  - `key_up` with `code`

Notes:
- Prefer `code` over `key` for keyboard consistency across layouts.
- Ignore repeated `key_down` if unwanted (`event.repeat`), or pass repeat explicitly.

## Coordinate Mapping (What You Asked)

Your expected behavior is correct:
- If you place cursor at a point inside the streamed video in browser,
- host cursor should move to the equivalent point on host screen.

Implementation approach:
- In browser, compute pointer position relative to displayed `<video>` rectangle.
- Normalize to `[0..1]` (`x_norm`, `y_norm`).
- In Rust, convert normalized coordinates to absolute `SendInput` range (`0..65535`) with virtual desktop metrics.

This keeps mapping stable even when video element is scaled in UI.

## Implementation Plan

### 1) Add DataChannel on Rust side (`ffmpeg-bot`)

File focus:
- `src/media_bridge.rs`

Tasks:
- During `handle_offer`, create DataChannel on bot peer connection:
  - label: `input`
  - optional: ordered + reliable for first pass
- Register `on_open` and `on_message` handlers with clear logs:
  - `input_channel_open`
  - `input_event_received`

### 2) Parse and dispatch input events in Rust

File focus:
- `src/models.rs` (new input DTOs)
- `src/media_bridge.rs` or a new module like `src/input_injector.rs`

Tasks:
- Add serde structs/enums for input messages.
- Implement message decode path:
  - UTF-8 text -> JSON parse -> event enum -> dispatcher
- Add robust validation:
  - clamp normalized coordinates to `[0.0, 1.0]`
  - ignore unknown event kinds without panic

### 3) Inject input on Windows

File focus:
- new `src/input_injector.rs` (recommended)
- `Cargo.toml` (add Windows API dependency)

Tasks:
- Add Windows bindings crate (`windows`) and required Win32 features.
- Implement helper functions:
  - `send_mouse_move_abs(x_norm, y_norm)`
  - `send_mouse_button_down(button)`
  - `send_mouse_button_up(button)`
  - `send_mouse_wheel(delta_y)`
  - `send_key_down(code)`
  - `send_key_up(code)`
- Map browser keyboard `code` values to Windows virtual keys/scancodes.

Keep mapping minimal for Week 4 MVP:
- letters, digits, arrows, Enter, Backspace, Space, Shift, Ctrl, Alt, Escape, Tab

### 4) Capture and send events in browser

File focus:
- `public/index.html`

Tasks:
- Add DataChannel handling:
  - receive channel in `pc.ondatachannel`
  - keep `inputDc` reference and ready-state checks
- Bind mouse events on `#remoteVideo`:
  - `mousemove` -> send normalized abs move (throttle to avoid flooding)
  - `mousedown` / `mouseup`
  - `wheel` (prevent default scrolling while focused)
- Bind keyboard events:
  - make video container focusable (`tabindex`)
  - send `keydown` / `keyup` when focused

Practical UX for PoC:
- Click on video to "arm" input capture.
- Show small log line: `input active`.

### 5) Logging and failure handling

File focus:
- `src/media_bridge.rs`
- `public/index.html`

Tasks:
- Log channel lifecycle and first successful event.
- On parse/injection failure, log warning and continue.
- Do not crash stream path on bad input packet.

## Recommended Milestones

### Milestone 1: DataChannel wiring

Exit criteria:
- Channel opens between browser and `ffmpeg-bot`.
- Test message from browser reaches Rust logs.

### Milestone 2: Mouse control

Exit criteria:
- Move, click, scroll are injected correctly on host.
- Cursor mapping matches pointer position in remote video.

### Milestone 3: Keyboard control

Exit criteria:
- Key down/up works for common keys.
- Can type in host Notepad from browser.

### Milestone 4: Stability pass

Exit criteria:
- 5-10 minute interactive run without crashes.
- Input remains responsive while video continues streaming.

## Risks and Mitigations

- Event flood from `mousemove`:
  - throttle send rate (for example 60 Hz max).
- Keyboard mapping mismatches:
  - start with common key subset and log unmapped codes.
- Browser focus loss:
  - explicit click-to-focus behavior + visible log status.
- Accidental host control during testing:
  - use a dedicated test window/session on host machine.

## Definition of Done

- Desktop browser (same LAN) controls host through stream view.
- Mouse move/click/scroll works.
- Keyboard typing works.
- Input path uses WebRTC DataChannel only.
- No gamepad code in Week 4 scope.
- Run steps are documented and reproducible.
