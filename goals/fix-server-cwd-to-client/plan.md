# Plan: Fix Server/Client CWD Fallback

## Solution Approach

The target code is already present in the current `master` checkout at `e42372d feat: server: use foreground client cwd for new terminals`. Treat this goal as a finish-and-verify pass: audit the committed implementation against `facts.md`, run the repository validation gates, perform the manual server/client CWD checks, and only make targeted fixes if a fact is not satisfied.

The architecture is: the client reports its launch directory with `ClientMessage::SetCwd` after the handshake, the server stores that path on the corresponding `ClientConnection`, the foreground app client's CWD is synced into `AppState`, and new terminal CWD resolution uses that foreground client CWD only as the `new_cwd = "follow"` fallback.

## Ordered Steps

1. Confirm the protocol change.
   - Files: `src/protocol/wire.rs`, protocol fixtures/tests under `tests/`.
   - Check that `PROTOCOL_VERSION` is `12`, `ClientMessage::SetCwd { cwd: PathBuf }` exists, and protocol version mismatch tests remain explicit.
   - Verification: run the protocol unit tests or `just check`; inspect `client_set_cwd_roundtrip` and version mismatch coverage.

2. Confirm the client send path.
   - File: `src/client/mod.rs`.
   - Check that `std::env::current_dir()` is captured as optional state, successful values send `SetCwd`, failures send nothing, and no fake `/` CWD is substituted.
   - Check that `SetCwd` is sent after `Hello`/`Welcome` succeeds and before `AttachTerminal`.
   - Verification: run focused client tests if needed, then `just check`.

3. Confirm server transport and connection state.
   - Files: `src/server/client_transport.rs`, `src/server/clients.rs`.
   - Check that `ClientMessage::SetCwd` becomes `ServerEvent::ClientSetCwd { client_id, cwd }`, and `ClientConnection` stores `cwd: Option<PathBuf>` initialized to `None`.
   - Verification: run `client_read_loop_emits_client_set_cwd` or `just check`.

4. Confirm foreground sync behavior.
   - Files: `src/server/headless.rs`, `src/app/state.rs`, `src/app/mod.rs`.
   - Check that foreground client sync copies `ClientConnection.cwd` into `AppState.foreground_client_cwd`, clears it when there is no foreground app client, and immediately resyncs when the foreground client later sends `ClientSetCwd`.
   - Verification: run headless server tests covering `ClientSetCwd` and foreground CWD clearing, then `just check`.

5. Confirm CWD resolution behavior.
   - Files: `src/app/creation.rs`, `src/app/input/mod.rs`, `src/app/api/workspaces.rs`, `src/app/api/tabs.rs`, `src/app/api/panes.rs`.
   - Check that `new_cwd = "follow"` resolves in this order: pane follow CWD, foreground client CWD, server `current_dir()`, then `/`.
   - Check that `new_cwd = "current"` still ignores the client CWD and uses the server process `current_dir()`.
   - Check that workspace creation, tab creation, and pane split paths all use the shared app helper when no explicit CWD is provided.
   - Verification: run `resolve_new_terminal_cwd` tests and `just check`.

6. Run the full local validation gate.
   - Command: `just check`.
   - Expected result: formatting, clippy, nextest, and maintenance script tests pass.
   - Result: passed on 2026-06-01 with `ZIG=/home/shuv/.local/opt/zig-x86_64-linux-0.15.2/zig just check`. The unpinned system `zig` is 0.16.0 and fails the vendored libghostty-vt build, while CI pins Zig 0.15.2.
   - If this fails, fix the smallest failing area and rerun `just check`.

7. Perform manual server/client validation.
   - Build or use fresh matching binaries from the same checkout.
   - Single-client check:
     ```bash
     cd ~/repos/shuvr
     shuvr
     # Create a workspace; expect ~/repos/shuvr when no pane CWD is available.

     cd /tmp
     shuvr
     # Create a workspace; expect /tmp when no pane CWD is available.
     ```
   - Multi-client check:
     ```bash
     cd /tmp/project-a && shuvr
     cd /tmp/project-b && shuvr
     # Interact with project-a client, create workspace: expect /tmp/project-a.
     # Interact with project-b client, create workspace: expect /tmp/project-b.
     ```
   - Result: completed on 2026-06-01 against fresh matching `target/debug/shuvr` server/client binaries with isolated `XDG_CONFIG_HOME`, `XDG_RUNTIME_DIR`, and `SHUVR_SOCKET_PATH` under `/tmp/shuvr-manual*`.
     - Single-client no-pane-CWD case: client launched from `/tmp/shuvr-manual-1780301860633/project-a`; `shuvr workspace create --label from-a --focus` returned root pane `cwd` and `foreground_cwd` as `/tmp/shuvr-manual-1780301860633/project-a`.
     - Multi-client no-pane-CWD case: clients launched from `/tmp/shuvr-manual2-1780301932895/project-a` and `/tmp/shuvr-manual2-1780301932895/project-b`; with B as the latest foreground client and no pane CWD, `workspace create` returned root pane `cwd` and `foreground_cwd` as `project-b`. After closing that workspace and sending input to client A, the next `workspace create` returned root pane `cwd` as `project-a`.
     - Tab and pane split CWD fallback precedence is covered by `just check` unit/integration tests and source inspection; the manual run focused on the no-pane-CWD workspace fallback because tabs/splits normally have an existing pane CWD, which intentionally takes precedence over foreground client CWD.

8. Prepare the commit boundary if new fixes are required.
   - Before committing, propose a conventional commit message and wait for explicit alignment.
   - Do not edit root `README.md`, `CHANGELOG.md`, `website/latest.json`, or released website docs for this normal fix unless validation proves a required change.
   - If the implementation remains exactly as `e42372d`, no new implementation commit is needed for the code fix itself.

## Verification Matrix

| Fact | Verification |
| --- | --- |
| Client sends launch directory after handshake | Inspect `src/client/mod.rs`; run client tests and `just check` |
| Failed `current_dir()` sends no fake `/` | Inspect helper tests in `src/client/mod.rs`; run `just check` |
| Protocol includes CWD message and version bump | Inspect `src/protocol/wire.rs`; run protocol tests |
| Server stores per-client CWD | Inspect `src/server/clients.rs` and `src/server/headless.rs`; run headless tests |
| Foreground CWD updates immediately | Run `client_set_cwd_updates_connection_and_foreground_app_state` |
| Follow CWD precedence is correct | Run `src/app/creation.rs` unit tests |
| Current policy ignores client CWD | Run `current_policy_ignores_foreground_client_cwd` |
| Workspace/tab/pane defaults use helper | Inspect API paths and run `just check` |
| Local validation includes `just check` | Execute `just check` |
| Manual validation covers single and multi-client cases | Execute the manual scenarios above |
| Docs/release files stay untouched | Check `git status --short` and `git diff -- README.md CHANGELOG.md website/latest.json website/src/content/docs docs/next/website/src/content/docs` |
| Commit requires alignment | Stop after proposing the message; do not commit until confirmed |

## Risks And Open Questions

- Manual validation may require controlling or stopping an already-running daemon so both client and server binaries come from the same checkout.
- The protocol bump intentionally rejects mixed client/server versions; if a compatibility policy changes, revisit `PROTOCOL_VERSION` and handshake behavior explicitly.
- `SetCwd` arrives after `ClientConnected`; the immediate foreground resync is required for the initially foreground client.
- API-created workspaces, tabs, and pane splits inherit the foreground client CWD when `new_cwd = "follow"` and no explicit CWD is provided. This is intended but should remain covered by tests or inspection.
- If a future task changes terminal-attach client foreground semantics, revisit whether attach clients should influence `foreground_client_cwd`.
