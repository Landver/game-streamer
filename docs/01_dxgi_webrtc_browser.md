# PoC Roadmap: DXGI + MediaFoundation + WebRTC + Browser Client

Goal:
Windows server captures screen via DXGI, encodes via MediaFoundation (H264 hardware),
streams via WebRTC.
Client is Android browser.

Stack:
- Rust server
- windows-rs (DXGI)
- MediaFoundation (hardware encode)
- webrtc-rs
- Tokio
- Simple WebSocket signaling server
- Browser (JS WebRTC + Gamepad API)

---

# Week 1 — Foundations + Signaling

## Goal:
Understand WebRTC signaling and build minimal working WebRTC connection (no video yet).

### Tasks:
- Learn basic Rust (async, ownership, Tokio)
- Build simple Rust WebSocket signaling server
- In browser:
  - Create RTCPeerConnection
  - Exchange SDP via WebSocket
  - Establish WebRTC connection
- Send test data over DataChannel (ping/pong)

Deliverable:
Browser connects to Rust server via WebRTC.
DataChannel works.

---

# Week 2 — Screen Capture + Encoding

## Goal:
Capture desktop and encode to H264 using hardware.

### Tasks:
- Implement DXGI Desktop Duplication capture
- Convert frame to NV12 or compatible format
- Use MediaFoundation to:
  - Initialize H264 encoder
  - Enable low-latency mode
  - Use hardware acceleration
- Verify encoding works:
  - Save output to .mp4 file

Deliverable:
You can capture screen and produce real-time H264 stream.

---

# Week 3 — Connect Encoding to WebRTC

## Goal:
Send encoded frames into WebRTC video track.

### Tasks:
- Create VideoTrack in webrtc-rs
- Feed H264 NAL units into track
- Configure browser to receive and display stream
- Tune:
  - Low-latency encoder settings
  - Bitrate
  - Frame pacing

Deliverable:
Android browser displays live Windows desktop stream.

---

# Week 4 — Input Handling

## Goal:
Send input from browser to Windows host.

### Tasks:
- In browser:
  - Capture keyboard
  - Capture mouse
  - Capture Gamepad via navigator.getGamepads()
- Send input via DataChannel
- On server:
  - Deserialize input
  - Inject using Windows SendInput API

Deliverable:
You can control Windows PC from Android browser.

---

# Pros

- Clean architecture
- Production-grade base
- Full control
- Hardware acceleration
- Future-proof

# Cons

- Rust learning curve
- MediaFoundation is complex
- More boilerplate

Best if:
You want long-term serious open-source project.