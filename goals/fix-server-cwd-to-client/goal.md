# Goal: Fix Server/Client CWD Fallback

Make server/client mode create new workspaces, tabs, and pane splits from the active foreground client's launch directory when `new_cwd = "follow"` has no pane CWD to follow. The target implementation uses a post-handshake `SetCwd` client message, per-client server CWD state, foreground CWD sync into `AppState`, and updated CWD resolution precedence.

The shared understanding is captured in `facts.md`. The approved execution and verification plan is captured in `plan.md`.

Done condition: every accepted fact in `facts.md` is satisfied, `just check` passes, manual fresh-client/server and multi-client CWD validation is completed or explicitly documented as partial, and no commit is made until the proposed conventional commit message is explicitly approved.
