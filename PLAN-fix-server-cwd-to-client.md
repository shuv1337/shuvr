# Plan: Fix Server CWD to Respect Client Launch Directory

## Problem

When `shuvr` runs in server/client mode, the long-lived server daemon keeps the CWD from the first launch. Later clients launched from other directories connect to that existing server, but new workspaces/tabs still resolve their default CWD from the server process instead of the active client's launch directory.

Expected behavior:

```bash
cd /some/project
shuvr
# Creating a workspace/tab should default to /some/project when no pane CWD is available.
```

## Root Cause

- `ClientMessage::Hello` does not carry the client's launch directory.
- `ClientConnection` does not track a per-client launch CWD.
- `HeadlessServer` tracks the foreground client for size/theme/keybindings, but not for CWD.
- `resolve_new_terminal_cwd()` falls back to `std::env::current_dir()`, which is the server daemon's CWD in server/client mode.

Relevant code:

- `src/protocol/wire.rs:56` defines `ClientMessage`.
- `src/client/mod.rs:416` sends the handshake.
- `src/server/client_transport.rs:353` maps client messages to `ServerEvent`.
- `src/server/headless.rs:523` syncs foreground client state into `AppState`.
- `src/app/creation.rs:11` resolves new terminal CWD.

## Design Decision

Do not add `cwd` to `ClientMessage::Hello`.

The protocol uses bincode over fixed enum/struct layouts. Adding a field to `Hello` would make old/new binaries fail to deserialize each other's handshake payloads. Instead, add a separate client message:

```rust
ClientMessage::SetCwd { cwd: PathBuf }
```

The client sends `SetCwd` immediately after a successful `Hello`/`Welcome` handshake and before entering the interactive read loop. The server records the CWD on the corresponding `ClientConnection` and syncs it into `AppState` when that client is foreground.

## Protocol Compatibility

Bump `PROTOCOL_VERSION` from `11` to `12`.

Reason: adding a new enum variant is a protocol change. With the current strict version check in `src/protocol/wire.rs:613`, bumping the version makes old-server/new-client and new-server/old-client mismatches fail during handshake with an explicit version error instead of allowing a new client to send an unknown `SetCwd` variant to an old server after handshake.

This is not wire-compatible across versions. That matches the repository's current policy: clients and servers with different `PROTOCOL_VERSION` values are rejected.

## Files to Modify

| File | Purpose |
|------|---------|
| `src/protocol/wire.rs` | Bump protocol version, add `ClientMessage::SetCwd`, add round-trip test |
| `src/client/mod.rs` | Capture client CWD and send `SetCwd` after successful handshake |
| `src/server/client_transport.rs` | Add `ServerEvent::ClientSetCwd` and emit it from `client_read_loop` |
| `src/server/clients.rs` | Add `cwd: Option<PathBuf>` to `ClientConnection` |
| `src/server/headless.rs` | Store CWD on clients and resync foreground CWD into `AppState` |
| `src/app/state.rs` | Add `foreground_client_cwd: Option<PathBuf>` to `AppState` |
| `src/app/creation.rs` | Use foreground client CWD before falling back to server `current_dir()` |
| `src/app/input/mod.rs` | Include foreground client CWD in direct `AppState::split_pane` resolution |
| `src/app/api/workspaces.rs` | Ensure API workspace creation default resolution uses updated app helper |
| `src/app/api/tabs.rs` | Ensure API tab creation default resolution uses updated app helper |
| `src/app/api/panes.rs` | Ensure API pane split default resolution uses updated app helper |

## Implementation Steps

### Phase 1: Protocol Extension

- [x] In `src/protocol/wire.rs`, import `std::path::PathBuf`.
- [x] Bump `PROTOCOL_VERSION` from `11` to `12`.
- [x] Add `ClientMessage::SetCwd { cwd: PathBuf }` after `Hello`.
- [x] Add a `client_set_cwd_roundtrip` bincode test.
- [x] Update any exhaustive `ClientMessage` matches to handle `SetCwd`.

### Phase 2: Client Send Path

- [x] In `src/client/mod.rs`, capture `std::env::current_dir().ok()` before or immediately after handshake.
- [x] After `do_handshake()` succeeds, send `ClientMessage::SetCwd { cwd }` only when `current_dir()` succeeded.
- [x] Do not substitute `/` for a failed client CWD. A failed lookup should mean "unknown client CWD" so the server can use its existing fallback chain.
- [x] Send `SetCwd` before `AttachTerminal` so all client modes have a known CWD when possible.
- [x] Use the existing `write_to_server()` helper at `src/client/mod.rs:962`.

### Phase 3: Server Transport Events

- [x] Add `ServerEvent::ClientSetCwd { client_id: u64, cwd: PathBuf }` in `src/server/client_transport.rs`.
- [x] Import `PathBuf` in `src/server/client_transport.rs`.
- [x] In `client_read_loop()`, map `ClientMessage::SetCwd { cwd }` to `ServerEvent::ClientSetCwd { client_id, cwd }`.
- [x] Do not modify `src/server/client_accept.rs` except for compile-required imports or signatures; it only accepts sockets and starts handshake threads.

### Phase 4: Client Connection State

- [x] Add `pub(crate) cwd: Option<PathBuf>` to `ClientConnection` in `src/server/clients.rs`.
- [x] Initialize `cwd` to `None` in `ClientConnection::new_with_mode()`.
- [x] Keep constructor signatures unchanged unless a test needs direct initialization. Prefer defaulting to `None` to minimize call-site churn.

### Phase 5: Foreground CWD State

- [x] Add `pub foreground_client_cwd: Option<PathBuf>` to `AppState` in `src/app/state.rs`.
- [x] Initialize it to `None` in `App::new()` at `src/app/mod.rs:394`.
- [x] In `HeadlessServer::sync_foreground_client_state()` at `src/server/headless.rs:523`, set `self.app.state.foreground_client_cwd` to the foreground client's `cwd.clone()`.
- [x] In all no-foreground branches of `sync_foreground_client_state()`, clear `foreground_client_cwd` to `None`.
- [x] In `HeadlessServer::handle_server_event()`, add a `ClientSetCwd` arm that updates `client.cwd = Some(cwd)`.
- [x] If `ClientSetCwd` is for the current `foreground_client_id`, immediately call `sync_foreground_client_state()`. This is required because `ClientConnected` currently foregrounds and syncs the client before the later `SetCwd` message can arrive.
- [x] Return `false` from the `ClientSetCwd` handler unless the immediate foreground sync needs a render for some future UI indicator. CWD alone should not require rerendering.
- [x] Existing removal/promotion paths should continue to call `sync_foreground_client_state()` through `promote_latest_remaining_client()` and `remove_client()`.

### Phase 6: CWD Resolution

- [x] Change the free function in `src/app/creation.rs` from:

```rust
resolve_new_terminal_cwd(policy, follow_cwd)
```

to:

```rust
resolve_new_terminal_cwd(policy, follow_cwd, foreground_client_cwd)
```

- [x] For `NewTerminalCwdConfig::Follow`, use this priority:

```text
follow_cwd -> foreground_client_cwd -> std::env::current_dir() -> /
```

- [x] Keep `NewTerminalCwdConfig::Current` as server `std::env::current_dir()`. The client CWD should only affect `Follow` fallback behavior, not the explicit `current` policy.
- [x] Update `App::resolve_new_terminal_cwd()` in `src/app/creation.rs:38` to pass `self.state.foreground_client_cwd.clone()`.
- [x] Update direct free-function callers, especially `src/app/input/mod.rs:410`, to pass `self.foreground_client_cwd.clone()`.
- [x] Confirm API default CWD flows still call `App::resolve_new_terminal_cwd()`:
  - `src/app/api/workspaces.rs:48`
  - `src/app/api/tabs.rs:66`
  - `src/app/api/panes.rs:23`

### Phase 7: Tests

- [x] Add protocol round-trip coverage for `ClientMessage::SetCwd` in `src/protocol/wire.rs`.
- [x] Add or update server transport coverage so a `SetCwd` frame becomes `ServerEvent::ClientSetCwd`.
- [x] Add a headless-server unit test for `ClientSetCwd` updating `ClientConnection.cwd`.
- [x] Add a headless-server or app-state test proving a foreground client's `SetCwd` immediately updates `app.state.foreground_client_cwd` after `ClientConnected`.
- [x] Update existing tests for `ClientMessage` exhaustive matches and protocol version expectations.
- [x] Update existing tests for `resolve_new_terminal_cwd()` to cover:
  - `follow_cwd` wins over client CWD.
  - client CWD wins over server `current_dir()` when `follow_cwd` is `None`.
  - server `current_dir()` fallback still works when client CWD is `None`.
  - `NewTerminalCwdConfig::Current` ignores client CWD.
- [x] If feasible, add a client-side test or small helper test ensuring failed `current_dir()` does not send `/` as a fake client CWD.

### Phase 8: Validation

- [x] Run `cargo test`.
- [x] Run `cargo clippy`.
- [x] If available, run `cargo nextest run`.
- [ ] Manual test with a fresh matching client/server binary:

```bash
cd ~/repos/shuvr
shuvr
# Create a workspace; expected cwd is ~/repos/shuvr when no pane cwd is available.

cd /tmp
shuvr
# Create a workspace; expected cwd is /tmp when no pane cwd is available.
```

- [ ] Manual multi-client test:

```bash
cd /tmp/project-a && shuvr
cd /tmp/project-b && shuvr
# Interact with project-a client, create workspace: expect /tmp/project-a.
# Interact with project-b client, create workspace: expect /tmp/project-b.
```

## Edge Cases

- Client launch directory disappears before `current_dir()` is captured: do not send `SetCwd`; server falls back normally.
- Non-UTF-8 paths: `PathBuf` supports platform paths, and bincode/serde should serialize `PathBuf` on Unix without requiring UTF-8.
- Multiple clients from different directories: foreground client's CWD is authoritative only when no pane/tab follow CWD is available.
- Foreground client disconnects: existing promotion/removal paths must clear or replace `foreground_client_cwd` via `sync_foreground_client_state()`.
- Direct terminal attach clients: they may send CWD, but `latest_app_client()` excludes terminal-attach clients from foreground promotion. This is acceptable unless future behavior needs attach clients to influence app workspace creation.

## Final Architecture

```text
Client launched from /tmp/project
  sends Hello { version: 12, cols, rows, ... }
  receives Welcome
  sends SetCwd { cwd: /tmp/project } when current_dir() succeeds
  enters normal input/read loop

Server
  receives Hello and creates ClientConnection { cwd: None }
  foregrounds the app client and syncs AppState { foreground_client_cwd: None }
  receives SetCwd and updates ClientConnection.cwd
  if that client is foreground, syncs AppState { foreground_client_cwd: Some(/tmp/project) }
  when creating a workspace/tab with new_cwd = follow:
    pane follow cwd -> foreground client cwd -> server current_dir -> /
```

## Commit Plan

- **TYPE:** `feat`
- **SUBJECT:** `server: use foreground client cwd for new terminals`
- **BODY:**
  - Add `SetCwd` to the client/server protocol and bump protocol version.
  - Track per-client launch CWD on `ClientConnection`.
  - Sync the foreground client's CWD into `AppState`.
  - Use foreground client CWD as the `new_cwd = "follow"` fallback for new workspaces, tabs, and pane splits.
  - Preserve server `current_dir()` fallback when no client CWD is known.

## Risks

- Protocol version bump requires matching client/server binaries. This is consistent with the current strict `PROTOCOL_VERSION` policy.
- `SetCwd` arrives after `ClientConnected`, so forgetting the immediate foreground resync would keep the bug for the initially foreground client until another sync event occurs.
- Over-applying client CWD could change explicit `new_cwd = "current"` semantics. Limit client CWD use to `Follow` fallback only.
- API-created workspaces/tabs without explicit `cwd` will inherit foreground client CWD when `new_cwd = "follow"`. This matches interactive behavior but should be noted in tests.
