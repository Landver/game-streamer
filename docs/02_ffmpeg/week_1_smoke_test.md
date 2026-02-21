# Week 1 Smoke Test

## Run

1. Start server:
   - `cargo run`
2. Open:
   - `http://127.0.0.1:3000` in two browser tabs/windows.
3. In each tab:
   - Keep same `Session` value.
   - Use different `Peer` IDs.
   - Click `Connect`.
4. In one tab click:
   - `Call first remote peer`.

## Expected

- Remote video appears in the other tab.
- Server logs include signaling flow:
  - `offer`
  - `answer`
  - `ice_candidate`
- Reconnecting after tab refresh works.

## Notes for Windows validation

- Run the server on Windows and repeat the same test.
- Keep firewall open for local testing.
- For internet/NAT tests, TURN is not included in Week 1 scope.
