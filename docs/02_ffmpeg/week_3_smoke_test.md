## Week 3 How To Test (Option A)

### Prerequisites

- Week 1 signaling flow works end-to-end (browser offer/answer/ICE path).
- Week 2 FFmpeg capture baseline is validated on this machine.
- FFmpeg is available in PATH.
- `h264_qsv` and `ddagrab` are available in current FFmpeg build.
- Rust backend includes Week 3 FFmpeg spawn + ingest + WebRTC track wiring.
- If testing from another device: firewall allows inbound TCP `3000`.

### Test A: Start backend services

1. Start signaling/backend service in terminal 1.
2. Confirm service is listening on expected host/port.
3. For remote device test, confirm bind is `0.0.0.0:3000` (or reachable LAN IP).

Pass criteria:
- Service starts without panic/error.
- Startup logs include signaling ready state.

### Test B: Start FFmpeg pipeline through Rust

1. Open browser client page.
2. Click `Connect`.
3. Click `Call stream bot` (target peer: `ffmpeg-bot`).
4. Watch backend logs.

Pass criteria:
- Log confirms `ffmpeg_spawned`.
- No immediate child-process exit.
- `first_frame_ingested` appears.
- `track_active samples_sent=...` keeps increasing.

### Test C: Browser playback

1. Open current Week 3 browser page.
2. Join session and complete signaling negotiation.
3. Wait for remote track and video render.
4. Optional: click `Fullscreen` for large viewing area.

Pass criteria:
- Browser receives remote video track.
- Desktop stream appears within 10 seconds after connection.
- Logs include `peer_connected` and `track_active`.

### Test D: Stability run (10-15 minutes)

1. Keep stream running for at least 10 minutes (15 preferred).
2. Move mouse/windows periodically to force motion changes.
3. Observe browser playback and backend logs.

Pass criteria:
- No backend crash/panic.
- No permanent black screen.
- Playback remains mostly smooth (PoC quality accepted).

### Test E: Restart and reconnect behavior

1. Stop server process (or trigger FFmpeg failure path).
2. Confirm backend detects exit and reports error.
3. Re-trigger start/rejoin path.

Pass criteria:
- Failure is logged clearly (not silent).
- Stream can recover by restart or reconnect sequence.
- Browser displays video again after recovery.

## Expected Log Milestones

Target events (exact wording can vary):

- `ffmpeg_spawned`
- `first_frame_ingested`
- `track_active`
- `peer_connected`
- error log on stream failure (only in failure/restart tests)

## Troubleshooting

- Browser connects but no video:
  - Confirm `first_frame_ingested` appears before/near `track_active`.
  - Confirm codec/profile assumptions match browser-compatible H264 path.
- Browser shows small video area:
  - Use `Fullscreen` button on the page.
  - Confirm latest `public/index.html` is loaded (hard refresh).
- FFmpeg exits immediately:
  - Recheck capture flags and device availability from Week 2.
  - Verify no conflicting desktop capture process is running.
- High latency/drift:
  - Recheck bitrate/GOP/preset parameters and queue depth.
  - Ensure buffer policy does not allow unbounded growth.
- Works once but fails after reconnect:
  - Ensure sender/task cleanup occurs on peer disconnect.
  - Ensure fresh track/session state is created on next join.

## Week 3 Test Completion Criteria

- Tests A-E pass on one reproducible local setup.
- Run steps are clear enough to repeat without guesswork.
- Any known limitations are documented for Week 4 follow-up.
