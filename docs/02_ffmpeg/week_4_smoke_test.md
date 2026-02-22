## Week 4 How To Test (Keyboard + Mouse Input Channel)

### Prerequisites

- Week 3 video path works (`ffmpeg-bot` stream visible in browser).
- Browser and host are on same LAN.
- Week 4 DataChannel input path is implemented.
- Windows host allows the app to inject input (running with proper permissions if needed).

### Test A: Baseline stream still works

1. Start backend with `cargo run`.
2. Open browser test page.
3. Connect and call `ffmpeg-bot`.
4. Confirm remote video renders.

Pass criteria:
- Video appears and remains stable for at least 1 minute.
- No new crashes/panics after adding input features.

### Test B: DataChannel opens

1. Keep the same session active.
2. Check browser log for input channel open event.
3. Check Rust logs for channel open event.

Pass criteria:
- Both browser and server show DataChannel ready/open.
- Sending one manual test packet is logged by server.

### Test C: Mouse move mapping

1. Open Notepad on host so cursor movement is easy to observe.
2. In browser, move cursor to top-left of remote video.
3. Move to center, then bottom-right.
4. Repeat while video element is resized/fullscreen.

Pass criteria:
- Host cursor moves to equivalent positions on host screen.
- Mapping remains correct after fullscreen toggle.

### Test D: Mouse click and drag

1. Single left click on host desktop icon via browser.
2. Double click to open item.
3. Right click to open context menu.
4. Click-and-drag a window by title bar.

Pass criteria:
- Left/right click actions trigger correctly.
- Drag behavior works without stuck button state.

### Test E: Mouse wheel

1. Open a scrollable page or document on host.
2. Scroll up/down from browser over remote video.

Pass criteria:
- Host scrolls in expected direction.
- No large jitter or repeated unwanted scroll bursts.

### Test F: Keyboard typing

1. Focus host Notepad.
2. From browser, type:
   - letters: `hello world`
   - digits: `123456`
   - symbols requiring shift: `!@#`
3. Test special keys:
   - Enter, Backspace, Tab, Escape
   - Arrow keys

Pass criteria:
- Characters appear correctly.
- Special keys behave as expected.
- No keys remain "stuck" after release.

### Test G: Combined interaction

1. While stream is active, alternate:
   - mouse movement/clicking
   - keyboard typing
2. Continue for 5-10 minutes.

Pass criteria:
- Input remains responsive.
- Video stream remains active.
- No memory/CPU runaway symptoms in logs.

## Expected Log Milestones

Target events (wording can vary):

- `input_channel_open`
- `input_event_received` (or periodic count)
- mouse injection success logs (or counters)
- keyboard injection success logs (or counters)
- warning logs for unknown/unmapped keys (non-fatal)

## Troubleshooting

- DataChannel never opens:
  - confirm channel is created by Rust peer and handled by browser `ondatachannel`.
  - verify offer/answer path still completes.
- Cursor mapping is offset/wrong:
  - verify browser sends coordinates relative to video element bounds, then normalized.
  - verify Rust maps normalized values to absolute `SendInput` range.
- Keyboard does nothing:
  - ensure browser element is focused before key events are sent.
  - verify key `code` to Windows mapping for the tested key.
- Scroll not working:
  - verify `wheel` event is sent and converted to Windows wheel units.
- Random stuck key/button:
  - add cleanup on page blur/unload to send key-up/button-up for pressed states.

## Week 4 Test Completion Criteria

- Tests A through G pass on one reproducible setup.
- User can type, click, move, and scroll from browser.
- Behavior matches PoC goal: direct host control via streamed view.
