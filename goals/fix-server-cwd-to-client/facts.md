# Facts

- A client sends its launch directory to the server after a successful protocol handshake when std::env::current_dir() succeeds.
- If the client cannot determine its launch directory, it does not send a fake fallback path such as /.
- The client/server protocol includes a CWD message and bumps PROTOCOL_VERSION so mismatched binaries fail explicitly during handshake.
- The server stores each app client's reported launch directory on that ClientConnection.
- When the foreground app client reports its CWD, AppState.foreground_client_cwd updates immediately without waiting for another foreground sync event.
- For new terminal creation with new_cwd = follow, CWD resolution uses pane follow CWD first, then foreground client CWD, then server current_dir(), then /.
- For new_cwd = current, CWD resolution continues to use the server process current_dir() and ignores foreground client CWD.
- Workspace creation, tab creation, and pane split paths use the updated default CWD resolution when no explicit CWD is provided.
- Local validation includes just check before any commit is made.
- Manual validation covers fresh matching client/server binaries launched from different directories and a multi-client foreground-CWD scenario.
- The implementation does not include public documentation, release manifest, README, or changelog edits unless validation reveals they are required.
- Before committing, the agent proposes a conventional commit message and waits for explicit alignment.
