# shuvr


<p align="center">
  <img src="assets/logo.png" alt="shuvr" width="100" />
</p>

<p align="center">
  <a href="https://github.com/shuv1337/shuvr">github.com/shuv1337/shuvr</a> · <a href="#install">install</a> · <a href="#quick-start">quick start</a> · <a href="#supported-agents">supported agents</a> · <a href="https://github.com/shuv1337/shuvr/docs/integrations/">integrations</a> · <a href="https://github.com/shuv1337/shuvr/docs/configuration/">configuration</a> · <a href="https://github.com/shuv1337/shuvr/docs/socket-api/">socket api</a>
</p>

---

https://github.com/user-attachments/assets/043ec09f-4bdd-41d5-aee0-8fda6b83e267

**agent multiplexer that lives in your terminal.**

workspaces, tabs, panes. mouse-native: click, drag, split. every agent at a glance: blocked, working, done. detach and reattach, agents keep running. no gui app, no electron, no mac-only native wrapper. you see the agent's own terminal, not someone's interpretation of it.

---

## install

```bash
curl -fsSL https://github.com/shuv1337/shuvr/install.sh | sh
```

or install with homebrew:

```bash
brew install shuvr
```

or download the binary from [releases](https://github.com/shuv1337/shuvr/releases). requires linux or macos.

## quick start

Start Shuvr in the directory where the work lives:

```bash
shuvr
```

Shuvr starts or attaches to one background session server. Press `ctrl+b`, then `shift+n` to create a workspace. Run an agent in the root pane. Press `ctrl+b`, then `v` or `minus` to split panes, `ctrl+b`, then `c` to create a tab, and `ctrl+b`, then `w` to switch workspaces.

Press `ctrl+b q` to detach the client. The server and pane processes keep running. Open another terminal and run `shuvr` again to reattach.

## core concepts

**Server and client.** By default, `shuvr` attaches to a background server. Detaching closes only the client. `shuvr server stop` stops the default server and kills its panes. Named sessions are separate server namespaces: use `shuvr session attach work`, `shuvr session stop work`, and `shuvr session list` when you want fully separate runtime state.

**Workspaces, tabs, panes.** A workspace is the project-level container. Tabs group panes inside a workspace. Panes are real terminal processes, not rewritten agent views.

**Copy.** Shuvr copies pane text, not the sidebar. Drag-select inside a pane, double-click a word or token, or press `prefix+[` for keyboard copy mode. In copy mode, move with `h/j/k/l`, `w/b/e`, and `{`/`}`, start selection with `v` or Space, copy with `y` or Enter, and leave with `q` or Esc. In PuTTY and some SSH terminals, hold `Shift` while dragging to use the terminal's own selection, and `Shift` + right click to paste.

**Update and restore.** `shuvr update` installs a new binary, but a running server keeps using the old process until it is stopped or handed off. Run `shuvr server stop` for the default session, or `shuvr session stop <name>` for a named session, then start Shuvr again. `shuvr update --handoff` is experimental and tries to move live panes, including foreground processes such as dev servers, from the old server to the new one. If `[session] resume_agents_on_restore = true` is enabled and current official integrations are installed, supported agent panes can restart from their native agent sessions after a server restart or update.

**Keybindings.** Shuvr uses explicit keybinding strings. `prefix+n` means press the configured prefix, then `n`. `ctrl+alt+n`, `cmd+k`, `alt+1`, and function-key chords are direct terminal-mode shortcuts and do not need the prefix. Plain direct printable keys such as `n` steal normal typing, so use `prefix+n` unless you intentionally want a modifier-gated direct binding.

**Agent awareness.** The sidebar shows blocked, working, done, and idle states. Detection works with process names and terminal output by default. Official integrations make state reporting and native agent session restore more reliable, but Shuvr still works as a terminal multiplexer without them.

## update

Shuvr notifies you when a new version is available. Run manually:

```bash
shuvr update
```

`shuvr update` is for installs managed by Shuvr's own installer. Homebrew and Nix installs update through `brew upgrade shuvr` or your Nix workflow. See [install docs](https://github.com/shuv1337/shuvr/docs/install/) and [session state docs](https://github.com/shuv1337/shuvr/docs/session-state/) for the full update, restart, restore, and handoff matrix.

## how it compares

|                          | tmux | gui managers | shuvr |
|--------------------------|------|--------------|-------|
| persistent sessions       | ✓    | —            | ✓     |
| detach / reattach        | ✓    | —            | ✓     |
| panes, tabs, workspaces  | ✓    | ✓            | ✓     |
| agent awareness          | —    | ✓            | ✓     |
| lives in your terminal   | ✓    | —            | ✓     |
| real terminal views      | ✓    | —            | ✓     |
| mouse-native            | —    | ✓            | ✓     |
| lightweight binary       | ✓    | —            | ✓     |
| agents can orchestrate   | ?    | ?            | ✓     |

tmux gives you persistence and panes, but it was built before agents existed. gui managers show agent state, but they make you leave your terminal and use their wrapped view. shuvr is persistence and awareness in one tool that stays out of your way.

## remote and attach

Shuvr works over normal SSH. Run it on the remote host, detach, and reattach later:

```
ssh you@yourserver
shuvr
```

You can also attach from your local terminal without opening a shell first:

```bash
shuvr --remote workbox
shuvr --remote ssh://you@yourserver:2222
```

Direct attach connects your current terminal to one server-owned terminal:

```bash
shuvr agent attach <target>
shuvr terminal attach <terminal_id>
```

See [persistence and remote docs](https://github.com/shuv1337/shuvr/docs/persistence-remote/) for remote keybinding, named-session, and handoff details.

## agent awareness

the sidebar shows which agents are blocked, working, or done. workspaces roll up to their most urgent state so you can scan the full list at a glance.

states:

- 🔴 **blocked** — agent needs input or approval
- 🟡 **working** — agent is actively running
- 🔵 **done** — work finished, you have not looked at it yet
- 🟢 **idle** — done and seen

detection works by reading foreground process and terminal output. zero config, no hooks required. for agents that expose hooks, the socket api integration gives more robust state reporting.

## lives in your terminal

not a gui window, not a web dashboard, not electron. shuvr runs inside whatever terminal you already use. single rust binary, no dependencies. works inside tmux.

## what you get

- **workspaces** — organized around git repos or folder names, each with its own tabs and panes
- **tabs** — first-class in the socket api and cli
- **copy-friendly** — drag-select pane text, double-click tokens, or use keyboard copy mode with `prefix+[`, `h/j/k/l`, `{`/`}`, `v`, and `y`
- **notifications** — sounds and toasts for background events; tab-aware suppression
- **18 built-in themes** — catppuccin, terminal, tokyo night, gruvbox, one, solarized, kanagawa, rosé pine, vesper, and light variants for the main palettes
- **session persistence** — pane processes survive client detach; sessions restore panes after full restart, with opt-in recent screen history

## agents can use shuvr too

The local Unix socket lets agents create workspaces, split panes, spawn helpers, read output, and wait for state changes. Start with the [socket API docs](https://github.com/shuv1337/shuvr/docs/socket-api/) and [`SKILL.md`](./SKILL.md).

## supported agents

automatic detection works out of the box. process name matching plus terminal output heuristics.

| agent | idle / done | working | blocked |
|-------|-------------|---------|---------|
| [pi](https://pi.dev) | ✓ | ✓ | partial |
| [claude code](https://docs.anthropic.com/en/docs/claude-code) | ✓ | ✓ | ✓ |
| [codex](https://github.com/openai/codex) | ✓ | ✓ | ✓ |
| [droid](https://factory.ai) | ✓ | ✓ | ✓ |
| [amp](https://ampcode.com) | ✓ | ✓ | ✓ |
| [opencode](https://github.com/anomalyco/opencode) | ✓ | ✓ | ✓ |
| [grok cli](https://x.ai/grok) | ✓ | ✓ | ✓ |
| [hermes agent](https://github.com/NousResearch/hermes-agent) | ✓ | ✓ | ✓ |
| cursor agent | ✓ | ✓ | ✓ |
| antigravity cli | ✓ | ✓ | ✓ |
| kimi code cli | ✓ | ✓ | ✓ |
| [github copilot cli](https://github.com/features/copilot) | ✓ | ✓ | ✓ |
| [qodercli](https://qoder.com/cli) | ✓ | ✓ | ✓ |
| [kiro cli](https://kiro.dev/docs/cli/) | ✓ | ✓ | — |

detected but not fully tested: gemini cli, cline.

for agents outside the built-in list, shuvr still works as a terminal multiplexer with workspaces, panes, and tiling. custom integrations can report agent labels over the socket api. see the [socket api docs](https://github.com/shuv1337/shuvr/docs/socket-api/).

### direct integrations

the built-in pi, omp, claude code, codex, opencode, hermes, and qodercli integrations forward semantic state to shuvr over the socket api. install with:

```bash
shuvr integration install pi
shuvr integration install omp
shuvr integration install claude
shuvr integration install codex
shuvr integration install opencode
shuvr integration install hermes
shuvr integration install qodercli
```

see the [integrations docs](https://github.com/shuv1337/shuvr/docs/integrations/) for setup details.

## keybindings

Press `ctrl+b` to enter prefix mode. Default actions are prefix-first and tmux-like:

| key | action |
|-----|--------|
| `prefix+c` | new tab |
| `prefix+n` / `prefix+p` | next / previous tab |
| `prefix+1..9` | switch tab |
| `prefix+w` | workspace navigation |
| `prefix+g` | session navigator |
| `prefix+shift+n` | new workspace |
| `prefix+shift+g` | new worktree |
| `prefix+shift+w` | rename workspace |
| `prefix+shift+d` | close workspace |
| `prefix+h/j/k/l` | focus pane |
| `prefix+v` / `prefix+minus` | split pane |
| `prefix+x` | close pane |
| `prefix+b` | toggle sidebar |
| `prefix+z` | zoom pane |
| `prefix+r` | resize mode |
| `prefix+q` | detach |

Mouse is supported throughout. Resize mode uses `h`/`l` for width, `j`/`k` for height, and `esc` to exit. Full syntax, optional actions, indexed bindings, and custom command bindings live in the [configuration docs](https://github.com/shuv1337/shuvr/docs/configuration/).

## configuration

config file: `~/.config/shuvr/config.toml`

```bash
shuvr --default-config   # print full default config
```

In-app settings cover theme, sound, and toast preferences. Shuvr writes logs under `~/.config/shuvr/`; in persistent session mode, `shuvr-client.log` and `shuvr-server.log` are usually the useful files. Full configuration and logging details live in the [configuration docs](https://github.com/shuv1337/shuvr/docs/configuration/).

## docs

- [quick start](https://github.com/shuv1337/shuvr/docs/quick-start/) — first session, panes, copy, and named sessions
- [install](https://github.com/shuv1337/shuvr/docs/install/) — install, update, Homebrew, and Nix
- [session state](https://github.com/shuv1337/shuvr/docs/session-state/) — detach, restart restore, agent restore, and live handoff
- [configuration](https://github.com/shuv1337/shuvr/docs/configuration/) — keybindings, themes, notifications, environment variables
- [integrations](https://github.com/shuv1337/shuvr/docs/integrations/) — pi, omp, claude code, codex, opencode, hermes, qodercli integrations
- [`SKILL.md`](./SKILL.md) — reusable agent skill
- [socket api](https://github.com/shuv1337/shuvr/docs/socket-api/) — socket protocol and cli reference

## agent instructions

if you are an ai agent helping with this repository, read [`AGENTS.md`](./AGENTS.md) before making changes and read [`CONTRIBUTING.md`](./CONTRIBUTING.md) before opening issues or PRs.

## development

```bash
git clone https://github.com/shuv1337/shuvr
cd shuvr
cargo build --release
./target/release/shuvr

just test        # unit tests
just check       # formatting, tests, and maintenance checks
```

## license

Shuvr is dual-licensed:

1. Open source: GNU Affero General Public License v3.0 or later (AGPL-3.0-or-later).
2. Commercial: commercial licenses are available for organizations that cannot comply with AGPL.

Contact: hey@github.com/shuv1337/shuvr

## mandatory star history

<a href="https://www.star-history.com/?repos=ogulcancelik%2Fshuvr&type=date&legend=top-left">
 <picture>
   <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/chart?repos=shuv1337/shuvr&type=date&theme=dark&legend=top-left&v=2026-05-19" />
   <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/chart?repos=shuv1337/shuvr&type=date&legend=top-left&v=2026-05-19" />
   <img alt="star history chart" src="https://api.star-history.com/chart?repos=shuv1337/shuvr&type=date&legend=top-left&v=2026-05-19" />
 </picture>
</a>
