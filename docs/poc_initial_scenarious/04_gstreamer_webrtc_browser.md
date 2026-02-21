# PoC Roadmap: GStreamer + WebRTC + Browser Client

Goal:
Use GStreamer to handle:
- Screen capture
- Hardware encoding
- WebRTC transport

Browser receives stream.
Rust used only for signaling.

Stack:
- GStreamer (Windows build)
- webrtcbin plugin
- Rust WebSocket signaling server
- Browser WebRTC client

---

# Architecture Overview

GStreamer Pipeline:

dxgiscreencapsrc
    ↓
videoconvert
    ↓
hardware encoder (h264 via MediaFoundation / QuickSync)
    ↓
webrtcbin
    ↓
Browser

Signaling:
Rust WebSocket server exchanges SDP + ICE.

---

# Week 1 — Learn GStreamer Basics

Goal:
Understand pipelines and elements.

Tasks:
- Install GStreamer (Windows MSVC build)
- Run test pipelines:
    - videotestsrc → autovideosink
    - dxgiscreencapsrc → autovideosink
- Enable hardware encoding:
    - Use mfh264enc (MediaFoundation encoder)
- Validate real-time encoding

Deliverable:
You can capture and encode desktop using pipeline.

---

# Week 2 — Add WebRTC

Goal:
Stream to browser via webrtcbin.

Tasks:
- Build pipeline with:
    dxgiscreencapsrc
    → videoconvert
    → mfh264enc
    → rtph264pay
    → webrtcbin
- Implement signaling:
    - Browser generates offer
    - Send to Rust server
    - Forward to GStreamer app
    - Exchange ICE candidates

Deliverable:
Browser displays Windows desktop stream.

---

# Week 3 — Input Channel

Goal:
Control PC from browser.

Tasks:
- Browser:
    - Capture keyboard/mouse/gamepad
    - Send via WebRTC DataChannel
- GStreamer:
    - Receive datachannel messages
- Forward input to small Rust service
- Inject via SendInput

Deliverable:
Full remote control works.

---

# Week 4 — Latency Tuning

Tasks:
- Enable low-latency encoder settings
- Tune:
    - GOP size
    - Bitrate
    - Keyframe interval
- Measure:
    - End-to-end latency
    - Frame drops
    - Jitter

Deliverable:
Sub-60ms LAN latency target.

---

# Pros

- Very production-grade architecture
- Cross-platform future
- Built-in WebRTC support
- Cleaner media pipeline separation

# Cons

- Steeper learning curve
- Harder debugging
- Rust integration less natural
- Documentation fragmented

Best if:
You want scalable, long-term streaming engine.