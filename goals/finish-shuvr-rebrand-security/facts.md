# Facts

- The remaining work finishes all unchecked slices in PLAN-shuvr-rebrand-security-hardening.md from the shuvr/rebrand-security worktree.
- Shuvr remains a separate namespace with no herdr CLI aliases, no HERDR_* fallback env vars, and no automatic migration from Herdr config, state, sockets, or release manifests.
- Product-facing docs use Shuvr, while historical changelog and release-note content is reviewed so old upstream history is not misleadingly presented as originally Shuvr-authored.
- Every release binary download path verifies a required sha256 checksum before chmod, install, upload to a remote host, or replacement of an existing binary.
- Session persistence writes private owner-only files, stale socket cleanup refuses non-socket paths, and no-HOME/no-XDG fallback paths avoid predictable shared /tmp names.
- GitHub release workflows, release asset names, install/update scripts outside website, Nix/dev tooling, README, and staged docs consistently target shuv1337/shuvr and shuvr asset names; website files are out of scope for the remaining goal.
- The implementer must resolve Zig 0.15.2 availability up front, run the full repo validation, and document a blocker only if validation remains impossible after reasonable attempts.
- After validation, the implementer proposes commit messages, waits for alignment, then commits, fast-forwards the shared checkout, pushes origin/master, and cleans up the task worktree/branch according to AGENTS.md.
