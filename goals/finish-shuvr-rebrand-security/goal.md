# Finish Shuvr Rebrand and Security Hardening

Finish all remaining slices of the Shuvr fork/rebrand and security hardening work from the existing task worktree at `/home/shuv/repos/herdr-worktrees/shuvr-rebrand-security` on branch `shuvr/rebrand-security`. Continue from the current partial implementation, resolve Zig 0.15.2 availability up front, skip further website work unless explicitly reopened, and carry the change through validation and the AGENTS.md commit/integration flow.

Use `goals/finish-shuvr-rebrand-security/facts.md` as the shared factual contract. It defines the separate Shuvr namespace, checksum-required binary downloads, filesystem hardening, non-website release/doc surfaces, validation requirements, and final commit/push/cleanup expectations.

Use `goals/finish-shuvr-rebrand-security/plan.md` as the execution plan. Follow its ordered steps: toolchain setup first, plan/current-state cleanup, namespace/runtime rebrand completion, checksum hardening, filesystem hardening, release workflow/tooling updates, non-website docs review, full validation, and final plan/commit flow.

Done means every accepted fact is satisfied, all relevant plan checkboxes are completed with evidence, full validation has passed or any remaining blocker is explicitly documented and accepted, commit messages have been proposed and aligned, and the task branch has been committed, fast-forwarded into `/home/shuv/repos/shuvr`, pushed to `origin/master`, and cleaned up according to AGENTS.md.
