# PoC Roadmap: FFmpeg + WebRTC Gateway + Browser

Goal:
Use FFmpeg to handle capture + hardware encode.
Use WebRTC only for transport.
Client is browser.

This approach trades control for speed.

Stack:
- Rust signaling server
- FFmpeg CLI or embedded ffmpeg
- WebRTC (either:
    - webrtc-rs
    - or external WebRTC gateway)
- Browser client

---

# Why FFmpeg?

FFmpeg already knows how to:
- Capture desktop
- Use hardware encoders (QuickSync, NVENC)
- Encode H264
- Tune low-latency parameters

This avoids writing DXGI + MediaFoundation manually.

---

# Week 1 — Signaling + Basic WebRTC

Same as Scenario 1 Week 1.

Goal:
Browser <-> Rust WebRTC works.

---

# Week 2 — FFmpeg Desktop Capture

## Goal:
Produce low-latency H264 stream from FFmpeg.

Tasks:
- Use FFmpeg:
  - ddagrab (Windows screen capture)
  - h264_qsv (Intel hardware encoder)
- Tune for low latency:
  - zerolatency
  - small GOP
  - CBR mode
- Validate:
  - Stream playable in VLC

Deliverable:
You can generate hardware-encoded H264 in real time.

---

# Week 3 — Pipe FFmpeg Into WebRTC

Primary target (chosen):

Option A:
- Spawn FFmpeg process from Rust
- Pipe encoded output into Rust ingest loop
- Feed data into WebRTC video track for browser playback

Optional fallback (not in current Week 3 scope):

Option B:
- Use RTP output from FFmpeg
- Consume RTP packets before WebRTC sender path

Deliverable:
Browser displays stream powered by FFmpeg.

Week 3 docs:
- `docs/02_ffmpeg/week_3_ffmpeg_to_webrtc_pipe.md`
- `docs/02_ffmpeg/week_3_smoke_test.md`

---

# Week 4 — Input Channel

Same as Scenario 1 Week 4.

---

# Pros

- Fastest PoC
- No DXGI API coding
- Less Rust complexity
- Media handled by mature system

# Cons

- External dependency
- Less fine-grained control
- Harder long-term custom optimizations

Best if:
You want something working fast to validate idea.