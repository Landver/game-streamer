# Week 1 Plan: Signaling + Basic WebRTC

## Scope

- Build the minimum path for a working browser-to-Rust WebRTC video session.
- No FFmpeg integration in this week.
- Optimize for speed of delivery (PoC quality).

## Deliverables 

- Rust signaling server with `offer`, `answer`, `ice_candidate`, `join`, `leave`.
- Browser test page that connects to signaling and establishes `RTCPeerConnection`.
- At least one successful run on Windows with a visible remote video track.
- Logs for signaling lifecycle events.

## Day-by-day

- **Day 1:** Bootstrap signaling server endpoints and message model.
- **Day 2:** Build browser test client and local media capture.
- **Day 3:** Complete answer flow and ICE exchange.
- **Day 4:** Add STUN config and verify LAN/internet path.
- **Day 5:** Add observability and fix blockers.
- **Day 6:** Run smoke tests and document exact run steps.
- **Day 7:** Freeze handoff notes for Week 2 FFmpeg ingest.

## Definition of done

- Browser can establish session with Rust backend in under 10 seconds.
- Signaling logs show full path: offer -> answer -> ICE -> connected.
- A reproducible successful run is documented for Windows.
