# abox

```
    _____
   /     \
  | () () |    abox
   \  ^  /    ─────────────────────
    |||||     sandbox for ai agents
    |||||
```

> Transparent sandbox for AI coding agents

Your agent works the same. abox wraps it in a sandbox and records everything.

```
$ abox claude
$ abox opencode
$ abox codex
```

## Install

```bash
curl -sSL https://raw.githubusercontent.com/4shil/agent-sandbox/main/install.sh | bash
```

---

## Usage

### Launch agent

```bash
$ abox claude              # launch Claude
$ abox opencode            # launch OpenCode
$ abox codex               # launch Codex
```

Just works. Session recorded in the background.

### No args? Auto-detect.

```bash
$ abox

    _____
   /     \
  | () () |    abox
   \  ^  /    ─────────────────────
    |||||     sandbox for ai agents

  Usage: abox <agent>

  Detected agents:
    [ ] claude       Claude Code
    [✓] opencode     OpenCode
    [✓] gemini       Google Gemini CLI

  Commands:
    abox init           setup wizard
    abox list           show sessions
    abox dashboard      interactive dashboard
    abox status         quick status
```

---

## Session Management

### List sessions

```bash
$ abox list

  +------------------------------------------+
  |  SESSION                     FILES       |
  +------------------------------------------+
  |  opencode-20260315-144537          1      |
  |  claude-20260315-123000            3      |
  |  codex-20260315-110000             2      |
  +------------------------------------------+
```

### Inspect a session

```bash
$ abox inspect opencode-20260315-144537

  +------------------------------------------+
  |  Session Details                         |
  +------------------------------------------+
  |  ID:       a1b2c3d4...
  |  Agent:    opencode
  |  Duration: 45.2s
  |  Actions:  12
  +------------------------------------------+
```

### Replay a session

```bash
$ abox replay opencode-20260315-144537

  [~] Session Replay
  Agent: opencode | Actions: 12

  ┌─ [1/12]
  │ session
  ├─────────────────────────────
  │ agent: opencode
  │ duration_ms: 45234
  └─────────────────────────────

  [n]ext  [p]rev  [d]etails  [q]uit >
```

---

## Export & Share

```bash
$ abox export opencode-20260315-144537 -o session.tar.gz
  [*] Exporting...
  [+] Done → session.tar.gz

$ abox import session.tar.gz
  [*] Importing...
  Agent: opencode
  Actions: 12
```

---

## Dashboard

```bash
$ abox dashboard

  +========================================+
  |   abox dashboard                       |
  +========================================+
  | Sessions:                            12 |
  | Disk used:                        8.9 MB |
  +========================================+

  Recent sessions:
    opencode-20260315-144537            1 sessions
    claude-20260315-123000              3 sessions

  [L]ist  [C]lean  [R]un  [Q]uit
```

---

## Quick Reference

| Command | Description |
|---------|-------------|
| `abox <agent>` | Launch agent in sandbox |
| `abox init` | First-run setup wizard |
| `abox status` | Quick status |
| `abox dashboard` | Interactive dashboard |
| `abox list` | List all sessions |
| `abox inspect <id>` | Show session details |
| `abox replay <id>` | Step-through replay |
| `abox export <id> -o f` | Export as tar.gz |
| `abox import <file>` | Import shared session |
| `abox clean --days 7` | Clean old sessions |
| `abox completions bash` | Shell tab completions |

---

## Config

First run `abox init` or create `~/.aboxrc`:

```bash
ABOX_DEFAULT_AGENT="opencode"
ABOX_MEMORY_LIMIT=""
ABOX_TIMEOUT=""
```

---

## How it works

```
$ abox opencode

1. Creates ~/.agent-sandbox/workspaces/opencode-<timestamp>/
2. Launches opencode (all env vars passed through)
3. You work normally
4. Session saved on exit
```

---

## License

MIT
