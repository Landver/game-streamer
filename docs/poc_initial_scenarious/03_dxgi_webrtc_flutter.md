# PoC Roadmap: DXGI + MediaFoundation + WebRTC + Flutter Client

Goal:
Native Android app using Flutter instead of browser.
Server same as Scenario 1.

Stack:
Server:
- Rust
- DXGI
- MediaFoundation
- webrtc-rs

Client:
- Flutter
- flutter_webrtc plugin

---

# Week 1 — Server Signaling + WebRTC

Same as Scenario 1 Week 1.

---

# Week 2 — Server Capture + Encode

Same as Scenario 1 Week 2.

---

# Week 3 — Flutter WebRTC Client

## Goal:
Receive video stream in Flutter app.

Tasks:
- Create Flutter project
- Add flutter_webrtc plugin
- Implement signaling client
- Handle:
  - SDP exchange
  - ICE candidates
- Display RTCVideoRenderer

Deliverable:
Flutter app shows live Windows desktop.

---

# Week 4 — Controller + Input

## Goal:
Full input pipeline.

Tasks:
- Capture:
  - Touch events
  - Physical controller input
- Serialize input
- Send via DataChannel
- Inject via SendInput on Windows

Deliverable:
Android app controls Windows machine.

---

# Week 5 — UX Improvements

- Add bitrate selector
- Add resolution selector
- Add connection status UI
- Add reconnect logic

---

# Pros

- Clean mobile UX
- App-store ready
- Future iOS support
- Better controller support than browser

# Cons

- More client work
- WebRTC mobile debugging pain
- Slightly slower initial PoC vs browser

Best if:
You want real product direction immediately.