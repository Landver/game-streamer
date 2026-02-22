# Week 2 Plan: FFmpeg Desktop Capture (No Execution)

## Context

Week 1 is complete (`signaling + basic WebRTC`).  
Week 2 goal is to produce a low-latency H264 desktop stream from FFmpeg and validate playback.

## Current Machine Snapshot (latest checks)

- GPU: `Intel(R) Arc(TM) B580 Graphics`
- FFmpeg in PATH: installed (`8.0.1`)
- Intel QSV encoders available: `h264_qsv`, `hevc_qsv`, `av1_qsv`
- Desktop capture filter available: `ddagrab`
- VLC in default path: not verified in this doc
- `winget`: available

## Week 2 Goal

Generate real-time hardware-encoded H264 from desktop capture on Windows, tuned for low latency, and confirm it is playable locally.

## Install and Configure (commands to run manually)

### 1) Install FFmpeg

```powershell
winget install --id Gyan.FFmpeg -e
```

### 2) Install VLC

```powershell
winget install --id VideoLAN.VLC -e
```

### 3) Restart terminal

After install, open a new terminal so PATH changes are loaded.

### 4) Verify FFmpeg is available

```powershell
ffmpeg -version
```

### 5) Verify encoder/filter support

```powershell
ffmpeg -encoders | Select-String h264
ffmpeg -h filter=ddagrab
```

Expected:
- `h264_qsv` is listed
- `ddagrab` help is shown

### 6) Quick hardware-encode probe (1 frame)

This confirms the QSV device can be initialized and used for encoding:

```powershell
ffmpeg -hide_banner -init_hw_device qsv=hw `
  -f lavfi -i "color=black:s=128x72:d=0.2" `
  -frames:v 1 -c:v h264_qsv -f null -
```

## Candidate Streaming Command (Week 2 baseline)

Use FFmpeg to capture desktop and send MPEG-TS over UDP to localhost.
This syntax is validated for FFmpeg `8.0.1` on Windows with Intel Arc:

```powershell
ffmpeg -init_hw_device d3d11va=dx -init_hw_device qsv=qs@dx -filter_hw_device dx `
  -f lavfi -i "ddagrab=framerate=60:output_idx=0:draw_mouse=1" `
  -vf "hwmap=derive_device=qsv,format=qsv" `
  -c:v h264_qsv `
  -preset veryfast `
  -g 60 -keyint_min 60 `
  -b:v 5M -maxrate 5M -bufsize 5M `
  -bf 0 -look_ahead 0 -async_depth 1 `
  -f mpegts "udp://127.0.0.1:1234?pkt_size=1316"
```

Note: in PowerShell, each line-ending backtick (`` ` ``) must be the last character on the line.

## Week 2 Definition of Done

- FFmpeg captures desktop in real time.
- Hardware H264 encoding (`h264_qsv`) is active.
- Stream is receivable on localhost (`udp://127.0.0.1:1234`).
