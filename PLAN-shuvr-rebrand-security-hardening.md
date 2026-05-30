# PLAN-shuvr-rebrand-security-hardening

## Context

This repository is being forked and rebranded from **Herdr** to **Shuvr**. The fork is a separate product namespace, not a compatibility layer over existing Herdr installs.

Decisions already made:

| Decision | Choice |
| --- | --- |
| Runtime/config compatibility | Separate namespace: no automatic Herdr migration, no Herdr aliases |
| First distribution target | GitHub Releases only |
| Security scope | Strict fixes for all high and medium audit findings in the same slice |
| Website scope | Root `website/**` is out of scope for the remaining goal; non-website docs and staged docs remain in scope |
| Normal docs guardrail | Root README/changelog edits are allowed because this is an explicit fork rebrand |

Relevant audit findings to fix:

- High: update/install/remote bootstrap downloads install release binaries without artifact integrity verification.
- Medium: persisted session snapshots/history can contain secrets and must be written with explicit owner-only file permissions.
- Medium: stale socket cleanup must not remove a non-socket path.
- Medium: config/state fallback paths must avoid predictable shared `/tmp` locations when HOME/XDG vars are unavailable.

## Goals

- Rebrand runtime, CLI, package, release, non-website docs, scripts, and tooling surfaces from Herdr to Shuvr.
- Keep Shuvr isolated from upstream Herdr runtime paths, sockets, env vars, and release manifests.
- Add cryptographic checksum verification to every release-binary download path that remains in scope.
- Harden local filesystem behavior for sockets, config/state directories, and persisted session data.
- Leave a repository state that can be validated and released through GitHub Releases.

## Non-Goals

- Do not preserve `herdr` CLI aliases.
- Do not read or migrate `HERDR_*` env vars, `~/.config/herdr`, `~/.local/state/herdr`, or Herdr sockets.
- Do not continue work under root `website/**` in this goal.
- Do not publish Homebrew, Nix registry, package manager, or public-domain infrastructure in this slice.
- Do not change the client/server wire protocol unless implementation reveals a real protocol compatibility change.
- Do not edit vendored `vendor/libghostty-vt/**` except if strictly required for validation tooling, which is not expected.

## Milestone 0: Validation Tooling

- [x] Install or locate Zig 0.15.2 outside constrained `/tmp`.
- [x] Confirm `ZIG=/home/shuv/.local/opt/zig-x86_64-linux-0.15.2/zig cargo test --locked --all-targets --no-run` reaches project compilation and succeeds.
- [x] Install or confirm `just`.
- [x] Install or confirm `cargo-nextest`.
- [x] Install or confirm `cargo-audit`.
- [x] Record exact tool paths and versions in `implementation-notes.html`.

## Milestone 1: Namespace and Runtime Rebrand

- [x] Rename the Cargo package and lockfile package from `herdr` to `shuvr`.
- [x] Update package repository/homepage metadata to `shuv1337/shuvr`.
- [x] Review package description and product copy for accurate Shuvr wording.
- [x] Replace runtime config/state directory names with `shuvr` and `shuvr-dev`.
- [x] Replace socket filenames with `shuvr.sock` and `shuvr-client.sock`.
- [x] Replace public env vars from `HERDR_*` to `SHUVR_*`.
- [x] Confirm no compatibility fallback reads old Herdr env vars or paths.
- [x] Rename integration agent-state assets to `shuvr-agent-state.*`.
- [x] Verify every integration asset reference, installed path, and embedded hook name matches the renamed files.
- [x] Add or update tests for config dir, state dir, socket path, env override behavior, and integration env propagation.

Validation:

- [x] `rg -n "HERDR_|herdr.sock|herdr-client.sock|herdr-dev|\\.config/herdr|\\.local/state/herdr" src scripts docs .github README.md CHANGELOG.md` returns only intentional historical references.
- [x] Targeted config/session/socket/integration tests pass.

## Milestone 2: Release Assets, Workflows, and Tooling

- [x] Update `scripts/changelog.py` default release repo and live manifest URL to `shuv1337/shuvr`.
- [x] Update expected release asset names to `shuvr-linux-x86_64`, `shuvr-linux-aarch64`, `shuvr-macos-x86_64`, and `shuvr-macos-aarch64`.
- [x] Update release workflows to copy from `target/<target>/release/shuvr`.
- [x] Update release workflows to upload Shuvr asset filenames.
- [x] Add release workflow checksum computation and pass real checksums into manifest generation.
- [x] Review bot token/user renames and document required `SHUVR_*` secrets if repository configuration is required.
- [x] Pin mutable GitHub Actions by commit SHA where feasible, prioritizing workflows with write permissions.
- [x] Update `justfile`, `flake.nix`, and `nix/package.nix` for Shuvr package/app/binary names.

Validation:

- [x] `python3 -m unittest scripts.test_changelog scripts.test_vendor_libghostty_vt` passes.
- [x] Workflow YAML parses.
- [x] `rg -n "herdr|Herdr|HERDR|KANGAL|kangal" .github justfile flake.nix nix` returns no unintended hits.
- [x] `nix flake check` passes if Nix is available, otherwise the skip reason is documented.

## Milestone 3: Release Binary Integrity Verification

Manifest shape includes required checksum fields:

```json
{
  "version": "0.x.y",
  "protocol": 11,
  "notes": "### ...",
  "assets": {
    "linux-x86_64": "https://github.com/shuv1337/shuvr/releases/download/v0.x.y/shuvr-linux-x86_64"
  },
  "checksums": {
    "linux-x86_64": "sha256:<64 hex chars>"
  }
}
```

- [x] Add checksum fields to generated manifests.
- [x] Require a checksum for every required asset target.
- [x] Normalize emitted checksums to canonical `sha256:<hex>`.
- [x] Remove any placeholder/dummy checksum path from release-facing code.
- [x] Parse real checksum data from GitHub release asset digest or workflow-generated checksum metadata.
- [x] Add Python tests for manifest checksums, missing checksum rejection, invalid checksum rejection, and real release payload checksum parsing.
- [x] Update `src/update.rs` to select checksum by platform and verify the downloaded file before chmod/install.
- [x] Update `src/remote.rs` to verify the downloaded remote bootstrap binary before upload.
- [x] Add Rust unit tests for checksum parsing, target selection, missing checksum rejection, and mismatch rejection in update/remote helpers.
- [x] Skip further root `website/**` installer work per approved goal; do not rely on root website changes for completion.

Validation:

- [x] `python3 -m unittest scripts.test_changelog scripts.test_vendor_libghostty_vt` passes.
- [x] Rust checksum tests pass.
- [x] `rg -n "sha256:6c696e|placeholder|dummy checksum|default_release_checksums" scripts src website` finds no release-facing placeholder checksum path.

## Milestone 4: Filesystem Hardening

Private session persistence:

- [x] Write `session.json` and `session-history.json` with owner-only permissions on Unix.
- [x] Preserve symlink behavior from `resolve_write_target`.
- [x] Set temp files used during atomic writes to owner-only permissions before rename.
- [x] Create parent directories with owner-only permissions where the platform supports it.
- [x] Add tests that assert saved files are not group/world-readable on Unix.

Safe socket cleanup:

- [x] If the path exists and is a socket, keep stale/live socket behavior.
- [x] If the path exists and is not a socket, return an error and do not remove it.
- [x] Include the path and file type in the non-socket error message where practical.
- [x] Add tests for live socket rejection and directory-not-removed behavior.

Safe config/state fallbacks:

- [x] Prefer XDG/HOME as today when available.
- [x] Avoid predictable `/tmp/shuvr` and `/tmp/shuvr-state` when neither HOME nor XDG is available.
- [x] Use a private unpredictable temp directory fallback.
- [x] Add tests for no-HOME/no-XDG private fallback behavior.

Validation:

- [x] Focused `ipc`, `persist::io`, and `config::io` tests pass.
- [x] `rg -n "/tmp/(herdr|shuvr)" src scripts tests docs` has no unsafe predictable fallback references.

## Milestone 5: Non-Website Docs and Historical Attribution

- [x] Update `README.md` and `docs/next/README.md` to Shuvr commands, paths, release URLs, and config docs.
- [x] Review `CHANGELOG.md` and `docs/next/CHANGELOG.md` so upstream Herdr history is not misleadingly presented as Shuvr-authored history.
- [x] Update staged docs under `docs/next/website/src/content/docs/**` for Shuvr install, quick start, configuration, CLI reference, socket API, session state, persistence/remote, integrations, and agent skill pages.
- [x] Keep root `website/**` unchanged in this goal unless the user explicitly reopens that scope.

Validation:

- [x] `just release-docs-check` passes, or the exact website-scope reason it cannot pass is documented.
- [x] Manual review of README, non-website install docs, socket API docs, and changelog baseline is recorded.

## Milestone 6: Final Validation and Release Readiness

- [x] Run `cargo fmt --check`.
- [x] Run `python3 -m unittest scripts.test_changelog scripts.test_vendor_libghostty_vt`.
- [x] Run `ZIG=/home/shuv/.local/opt/zig-x86_64-linux-0.15.2/zig cargo test --locked --all-targets`.
- [x] Run `ZIG=/home/shuv/.local/opt/zig-x86_64-linux-0.15.2/zig cargo clippy --all-targets --locked -- -D warnings`.
- [x] Run `just check`.
- [x] Run `cargo audit`.
- [x] Run `nix flake check` if Nix is available.
- [x] Build release binary with `ZIG=/home/shuv/.local/opt/zig-x86_64-linux-0.15.2/zig cargo build --release --locked`.
- [x] Run `./target/release/shuvr --version`.
- [x] Update `implementation-notes.html` with final decisions, deviations, validation outputs, and blockers if any.

## Commit Strategy

Propose exact conventional commit messages and wait for maintainer alignment before committing. After alignment, commit in the task worktree, fast-forward `/home/shuv/repos/shuvr`, push `origin/master`, then clean up the task worktree and branch according to `AGENTS.md`.

Potential commit grouping:

1. `chore: rebrand package namespace to shuvr`
2. `chore: rebrand cli docs and integration surfaces`
3. `fix: verify release binary checksums`
4. `fix: harden shuvr filesystem state handling`
5. `chore: rebrand release workflows and tooling`
6. `docs: update shuvr release documentation`

## Risks and Watchpoints

- Renaming env vars and integration hook variables is a breaking change; this is intentional for the separate namespace decision.
- Manifest checksum schema must be updated consistently in local update, remote bootstrap, release workflow, and tests.
- Root `website/**` was touched by an earlier mechanical pass and has been restored out of scope; do not continue website work without explicit approval.
- Root README/changelog are normally protected by repo conventions, but this explicit rebrand requires coordinated edits.
- Debug builds must preserve separate `shuvr-dev` paths.
- Do not accidentally edit vendored files under `vendor/libghostty-vt/**`; deeper AGENTS files apply there.
