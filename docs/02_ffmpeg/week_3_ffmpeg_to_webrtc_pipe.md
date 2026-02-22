# Week 3: FFmpeg -> Rust Pipe -> WebRTC (Option A)

## Context

Week 1 complete: signaling + basic WebRTC.
Week 2 complete: FFmpeg desktop capture + hardware H264 encode baseline.

Week 3 goal is to connect those parts so browser playback is powered by FFmpeg media through Rust WebRTC.

## Week 3 Goal

Spawn FFmpeg from Rust, ingest encoded video from process output, and publish it to a WebRTC video track that the browser can render in real time.

## Current Implementation Snapshot

- Server-side synthetic peer ID: `ffmpeg-bot`.
- Browser uses receive-only WebRTC (`recvonly`) and calls `ffmpeg-bot`.
- Rust backend handles `offer` and `ice_candidate` messages targeting `ffmpeg-bot`.
- Rust spawns FFmpeg and reads H264 Annex-B from stdout.
- H264 is grouped into access-unit samples and pushed to `TrackLocalStaticSample`.
- Default bind is `0.0.0.0:3000` (override with `HOST` / `PORT` env vars).

## Scope

- Keep signaling flow from Week 1 (`offer`, `answer`, `ice_candidate`, `join`, `leave`).
- Reuse the validated Week 2 FFmpeg capture settings as source.
- Focus on single video track path first (no audio required for Week 3).
- Prioritize stability and reproducibility over perfect quality tuning.

## Architecture (Option A)

1. Browser joins room through signaling server.
2. Rust creates peer connection + outgoing video track.
3. Rust starts FFmpeg as child process with low-latency desktop capture + H264 encode flags.
4. Rust reads FFmpeg output stream and forwards frame payloads to the WebRTC sender path.
5. Browser receives remote track and renders video.

## Milestones

### Milestone 1: FFmpeg process control in Rust (done)

- Add FFmpeg child process launcher in backend.
- Keep command args in one place (easy tuning).
- Capture stderr logs for diagnostics.
- Detect startup failure and process exit.

Exit criteria:
- FFmpeg starts from Rust and runs continuously.
- Rust logs show start, running, and exit events clearly.

### Milestone 2: Ingest loop + frame boundaries (done)

- Implement non-blocking ingest loop from FFmpeg output.
- Add frame boundary handling for chosen output format.
- Guard against unbounded memory growth (bounded buffers/backpressure).
- Expose counters (bytes read, frames sent, dropped frames).

Exit criteria:
- Rust continuously ingests media bytes without crashing or deadlocking.
- Frame counters are visible in logs.

### Milestone 3: Feed WebRTC track (done)

- Map ingested media into WebRTC sender input format.
- Attach sender to negotiated peer connection.
- Start send loop after peer/session is ready.
- Handle disconnect/reconnect by restarting sender path safely.

Exit criteria:
- Browser receives remote track and displays live desktop.
- Session survives at least one reconnect sequence.

### Milestone 4: Stability + latency tuning (in progress)

- Run 10-15 minute stability test.
- Tune FFmpeg bitrate/GOP/preset if stutter appears.
- Add practical guardrails (restart policy, clear error messages).
- Freeze known-good run parameters in docs.

Exit criteria:
- Stream remains stable for target duration.
- Reproducible local run steps documented.

## Recommended Implementation Order

1. Add FFmpeg process launcher and logging.
2. Verify ingest loop with metrics only (before wiring to browser).
3. Wire ingest output into WebRTC track path.
4. Validate browser playback.
5. Tune and harden.

## Risks and Mitigations

- Frame parsing mismatch:
  - Start with one known output format and document exact assumptions.
- Backpressure when browser/network slows down:
  - Use bounded queue and drop policy for oldest frames when needed.
- FFmpeg child process unexpectedly exits:
  - Add restart policy with capped retries and explicit logs.
- High latency over time:
  - Keep small GOP, disable deep look-ahead, and track queue depth.

## Definition of Done

- Browser displays remote desktop video sourced from FFmpeg (`ffmpeg-bot` path).
- End-to-end path is reproducible on Windows using documented commands.
- 10-15 minute run completes without crash or unrecoverable stall.
- Logs clearly show lifecycle: ffmpeg_spawned -> first_frame_ingested -> track_active -> peer_connected.

## Known Limitations

- Input control (mouse/keyboard injection) is not part of Week 3 scope.
- Week 3 provides video transport only; interactive control is Week 4.
