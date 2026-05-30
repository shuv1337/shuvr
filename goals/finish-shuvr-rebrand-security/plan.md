# Finish Shuvr Rebrand and Security Hardening Plan

## Solution Approach

Continue from the existing task worktree at `/home/shuv/repos/herdr-worktrees/shuvr-rebrand-security` on branch `shuvr/rebrand-security`. Treat the current implementation as a partial first pass: keep the useful broad namespace/security changes, then finish the unchecked slices, clean up mechanically-renamed text where it became nonsensical, restore historical attribution where needed, and resolve Zig 0.15.2 availability before continuing, then run full validation as changes are completed.

The source facts are in `goals/finish-shuvr-rebrand-security/facts.md`; the live implementation checklist is `PLAN-shuvr-rebrand-security-hardening.md`.

## Ordered Steps

### 1. Resolve Zig 0.15.2 and Validation Tooling Up Front

Files/systems:
- Local toolchain
- `implementation-notes.html`

Tasks:
- Make Zig 0.15.2 available before further implementation. Do not use constrained `/tmp`; install/extract under a path with sufficient space such as `/home/shuv/.local/opt/zig-x86_64-linux-0.15.2` or another user-local cache.
- Export or pass `ZIG=/path/to/zig-0.15.2/zig` for Rust build/test commands.
- Install or confirm `just` and `cargo-nextest` availability.
- Run a minimal compile/test probe once Zig is available so later failures are real code failures, not environment setup failures.
- Record exact tool paths and versions in `implementation-notes.html`.

Verification:
```bash
/path/to/zig-0.15.2/zig version
ZIG=/path/to/zig-0.15.2/zig cargo test --locked --all-targets --no-run
```

### 2. Normalize the Plan and Current State

Files/systems:
- `PLAN-shuvr-rebrand-security-hardening.md`
- `implementation-notes.html`
- Git worktree state

Tasks:
- Fix mechanically-renamed checklist text such as `SHUVR_CONFIG_PATH -> SHUVR_CONFIG_PATH`, `shuvr -> shuvr`, and `Shuvr -> Shuvr` so the remaining checklist is meaningful.
- Re-check `git status --short` and separate changes into these groups: core runtime, security hardening, release workflows, non-website docs, tests/tooling, goal artifacts.
- Keep `goals/finish-shuvr-rebrand-security/**` as the goal package; do not accidentally treat it as implementation code.
- Treat `website/**` as out of scope for remaining work except reverting or avoiding already-created website changes if needed for consistency.

Verification:
- `rg -n "SHUVR_.*-> SHUVR_|shuvr.*-> shuvr|Shuvr.*to Shuvr" PLAN-shuvr-rebrand-security-hardening.md` returns no nonsensical checklist entries.
- `git status --short` is understood and no unexpected vendored files are modified.

### 3. Finish Namespace and Runtime Rebrand

Files/systems:
- `Cargo.toml`, `Cargo.lock`
- `src/config.rs`, `src/config/io.rs`, `src/session.rs`
- `src/api/mod.rs`, `src/server/socket_paths.rs`, `src/client/mod.rs`
- `src/cli/**`, `src/integration/**`, `tests/**`

Tasks:
- Verify the package name, binary name, config/state directories, socket names, CLI help, runtime env vars, integration hook names, and tests consistently use `shuvr` / `Shuvr` / `SHUVR_*`.
- Confirm no compatibility fallback reads old `HERDR_*` variables or old Herdr paths.
- Add or repair unit tests for config dir, state dir, socket path, env override behavior, and integration env propagation.
- Ensure renamed integration assets are referenced correctly by `include_str!` and install paths.

Verification:
- `rg -n "herdr|Herdr|HERDR|ogulcancelik/herdr|herdr\\.dev" -g '!vendor/**' -g '!target/**' -g '!website/bun.lock' .` returns only intentional historical/goal-package references, if any.
- Targeted Rust tests for config/session/socket/integration behavior pass once Zig 0.15.2 is available.
- `cargo fmt --check` passes.

### 4. Complete Release Manifest and Checksum Hardening

Files/systems:
- `scripts/changelog.py`
- `scripts/test_changelog.py`
- `src/update.rs`
- `src/remote.rs`
- `website/install.sh` and `website/latest.json` only if retaining already-made changes is necessary; otherwise skip website files per gate feedback

Tasks:
- Replace placeholder checksum behavior with release-ready checksum handling:
  - Manifest generator must require real `sha256:<64 hex>` checksums for all four assets.
  - GitHub release parsing must use a reliable digest source; if GitHub API asset `digest` is not available, add workflow-generated checksum input/artifact support rather than silently inventing checksums.
  - Do not spend further effort on `website/latest.json`; website work is out of scope. If current website manifest edits remain in the worktree, either revert them or document why keeping them is necessary.
- Add Rust unit tests for checksum parsing, checksum selection by target, missing checksum rejection, and mismatch rejection in update/remote download helpers.
- Add install script smoke tests or a small shell fixture test for success, missing checksum, and mismatch behavior.
- Keep all download paths fail-closed before chmod/install/remote upload/replacement.

Verification:
- `python3 -m unittest scripts.test_changelog scripts.test_vendor_libghostty_vt` passes.
- New Rust checksum tests pass with Zig 0.15.2 available.
- Install script smoke tests pass.
- `rg -n "sha256:6c696e|placeholder|dummy checksum|default_release_checksums" scripts src website` finds no release-facing placeholder checksum path.

### 5. Complete Filesystem Hardening

Files/systems:
- `src/ipc.rs`
- `src/persist/io.rs`
- `src/config/io.rs`
- Related tests

Tasks:
- Finish socket cleanup tests: live socket rejected, stale socket removed, regular file not removed, directory not removed.
- Include path and file type in the non-socket socket-path error message where practical.
- Add no-HOME/no-XDG tests for private fallback behavior or, if switching to fallible config/state dirs, update callers and tests accordingly.
- Verify session/history files remain `0600`, symlink behavior still works, and parent dirs are private.

Verification:
- Focused Rust tests for `ipc`, `persist::io`, and `config::io` pass with Zig 0.15.2 available.
- `rg -n "/tmp/(herdr|shuvr)" src scripts tests docs` has no unsafe predictable fallback references.

### 6. Finish Release Workflows and Tooling

Files/systems:
- `.github/workflows/release.yml`
- `.github/workflows/build-artifacts-manual.yml`
- `.github/workflows/approve-*.yml`, `.github/workflows/label-next-release-issues.yml`
- `justfile`
- `flake.nix`, `nix/package.nix`

Tasks:
- Verify all release artifact names are exactly:
  - `shuvr-linux-x86_64`
  - `shuvr-linux-aarch64`
  - `shuvr-macos-x86_64`
  - `shuvr-macos-aarch64`
- Ensure workflows copy from `target/<target>/release/shuvr` and upload Shuvr artifact paths.
- Add release workflow checksum computation and wire those checksums into `scripts/changelog.py sync-latest-json`.
- Review bot token/user renames. If `SHUVR_GITHUB_TOKEN` and `shuvr-bot` are not actually configured, either document required secrets or use `github.token` where sufficient.
- Update `justfile`, Nix package/app names, and docs examples so `nix run` and release recipes target `shuvr`.
- Pin mutable GitHub Actions by commit SHA where feasible, prioritizing workflows with write permissions.

Verification:
- YAML parses.
- `rg -n "herdr|Herdr|HERDR|KANGAL|kangal" .github justfile flake.nix nix` returns no unintended hits.
- `nix flake check` passes if Nix is available; otherwise note skipped reason.

### 7. Review Non-Website Docs and Historical Attribution

Files/systems:
- `README.md`, `CHANGELOG.md`
- `docs/next/README.md`, `docs/next/CHANGELOG.md`
- `docs/next/website/src/content/docs/**` only if treated as staged docs outside the website skip
- Do not continue work under `website/**`

Tasks:
- Keep product-facing docs fully Shuvr.
- Review changelog/release-note history so upstream Herdr history is not misleadingly rewritten as if old releases were originally Shuvr-authored. Use an explicit fork-baseline note or preserve upstream attribution where clearer.
- Replace non-website install/update examples with GitHub Releases-first Shuvr commands.
- Skip `website/**` work. If release-doc mirror checks require website files, document that this goal intentionally excludes those checks or ask before reopening website scope.

Verification:
- `just release-docs-check` passes once `just` is available.
- Manual review of README, non-website install docs, socket API docs, and changelog baseline.

### 8. Run Full Validation

Files/systems:
- Local toolchain established in Step 1
- `implementation-notes.html`

Tasks:
- Rerun the full validation sequence after implementation work is complete.
- If a validation command cannot run, record the exact blocker and evidence in `implementation-notes.html`.

Verification commands:
```bash
cargo fmt --check
python3 -m unittest scripts.test_changelog scripts.test_vendor_libghostty_vt
ZIG=/path/to/zig-0.15.2/zig cargo test --locked --all-targets
ZIG=/path/to/zig-0.15.2/zig cargo clippy --all-targets --locked -- -D warnings
just check
cargo audit
nix flake check
```

### 9. Finalize Plan Progress and Commit Flow

Files/systems:
- `PLAN-shuvr-rebrand-security-hardening.md`
- `implementation-notes.html`
- Git branch/worktrees

Tasks:
- Mark completed plan checkboxes only after corresponding validation passes or a documented blocker is accepted.
- Update `implementation-notes.html` with final decisions, deviations, validation outputs, and remaining blockers if any.
- Propose commit messages and wait for alignment.
- After alignment, commit in the task worktree, fast-forward the shared checkout at `/home/shuv/repos/shuvr`, push `origin/master`, then clean up the task worktree and branch according to `AGENTS.md`.

Verification:
- `git diff --check` passes.
- `git status --short` contains only intended changes before commit.
- Shared checkout fast-forwards cleanly after commit.
- Remote push succeeds.

## Risks and Open Questions

- The current partial implementation includes a broad mechanical rebrand across docs, tests, and historical changelog text; website files should not receive further work unless explicitly reopened. The next implementer must review this carefully rather than assuming every replacement is semantically correct.
- Zig 0.15.2 setup is the first implementation step; do not defer it to the end.
- Workflow bot identity and `SHUVR_GITHUB_TOKEN` may need repository secret setup; if unavailable, document exact required GitHub configuration.
