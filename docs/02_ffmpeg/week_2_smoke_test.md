## How To Test

### Prerequisites

- FFmpeg `8.0.1` installed and available in PATH.
- `ffmpeg -h filter=ddagrab` succeeds.
- `ffmpeg -encoders | Select-String h264_qsv` shows `h264_qsv`.

### Test A: Start sender (desktop capture + Intel QSV)

Run in terminal 1:

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

Pass criteria:
- No startup error from `ddagrab` or `h264_qsv`.
- FPS and bitrate keep updating continuously.

### Test B: Receiver smoke test with FFmpeg (recommended first)

Run in terminal 2 while terminal 1 is sending:

```powershell
ffmpeg -i "udp://127.0.0.1:1234?fifo_size=5000000&overrun_nonfatal=1" -t 5 -an -f null -
```

Pass criteria:
- Input is detected as MPEG-TS/H264.
- Command exits `0` after ~5 seconds.
- Decoded frame count increases (not stuck at 0).

### Test C: VLC playback smoke test

1. Keep sender running in terminal 1.
2. Open VLC.
3. `Media -> Open Network Stream`.
4. Enter:
   - `udp://@:1234`
5. Click Play.

Pass criteria:
- Video appears within a few seconds.
- Playback remains stable for at least 2 minutes.

### Test D: Latency check (manual)

1. Keep sender + VLC running.
2. Move mouse/windows quickly on desktop.
3. Observe delay in VLC.

Target:
- Visibly low delay (PoC acceptable range).
- No major stutter/freezes.

### Test E: Encoder confirmation (quick probe)

Run:

```powershell
ffmpeg -hide_banner -init_hw_device qsv=hw `
  -f lavfi -i "color=black:s=128x72:d=0.2" `
  -frames:v 1 -c:v h264_qsv -f null -
```

Pass criteria:
- Command succeeds and output line includes `h264_qsv`.

## Troubleshooting

- `ffmpeg not recognized`:
  - reopen terminal; if still failing, verify install path and PATH.
- `h264_qsv not found`:
  - update Intel graphics driver, then retest.
- `ddagrab not found`:
  - your FFmpeg build may lack this filter; install a build that includes it.
- `Unknown input format: 'ddagrab'`:
  - this means old syntax was used (`-f ddagrab -i desktop`). Use `-f lavfi -i "ddagrab=..."`.
- `non-existing PPS 0 referenced` on receiver startup:
  - expected when receiver joins mid-stream; wait for next keyframe (~1 second at GOP 60).
- VLC black screen:
  - confirm sender FPS is increasing and run Test B first; then retry `udp://@:1234`.
