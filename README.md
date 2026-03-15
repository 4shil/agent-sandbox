# 🛡️ abox

> Transparent sandbox for AI coding agents

Your agent works the same. abox just wraps it in a sandbox and records everything.

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
$ abox claude              # launches Claude in sandbox
$ abox opencode            # launches OpenCode in sandbox
$ abox codex               # launches Codex in sandbox
```

You interact with the agent normally. Session is recorded silently in the background.

---

### Session Management

#### List sessions

```bash
$ abox list

📦 Sessions

  opencode-20260315-144537            1 sessions
  claude-20260315-123000              3 sessions
  codex-20260315-110000               2 sessions
```

#### Inspect a session

```bash
$ abox inspect opencode-20260315-144537

🔍 Session Details

  ID:        a1b2c3d4-...
  Agent:     opencode
  Sandbox:   opencode-20260315-144537
  Duration:  45.2s
  Actions:   12

  Actions:
    session              1
    file_modified        8
    task_end             1
```

#### Replay a session

```bash
$ abox replay opencode-20260315-144537

🔄 Session Replay
   Agent: opencode | Actions: 12

┌─ [1/12] ──────────────────────────────
│ Type: session
├───────────────────────────────────────
│ agent: opencode
│ duration_ms: 45234
└───────────────────────────────────────

[n]ext [p]rev [d]etails [q]uit >
```

---

### Export & Share

#### Export a session

```bash
$ abox export opencode-20260315-144537 -o my-session.tar.gz

📦 Exporting session...
   opencode-20260315-144537 → my-session.tar.gz (1.2MB)
```

#### Import a shared session

```bash
$ abox import my-session.tar.gz

📥 Importing session...
   Agent: opencode
   Actions: 12
   Imported to: ~/.agent-sandbox/imports
```

---

### Cleanup

#### Clean old sessions

```bash
$ abox clean --days 7
🗑️  Removed 5 old sessions (older than 7 days)

$ abox clean               # default: 30 days
🗑️  Removed 12 old sessions (older than 30 days)
```

---

## Quick Reference

| Command | Description |
|---------|-------------|
| `abox <agent>` | Launch agent in sandbox |
| `abox list` | List all sessions |
| `abox inspect <id>` | Show session details |
| `abox replay <id>` | Step-through replay |
| `abox export <id> -o file.tar.gz` | Export session |
| `abox import <file.tar.gz>` | Import session |
| `abox clean --days 7` | Clean old sessions |

---

## How it works

```
$ abox opencode

1. Creates isolated workspace
   ~/.agent-sandbox/workspaces/opencode-20260315-144537/

2. Launches opencode inside sandbox
   (all env vars passed through — API keys work)

3. You work normally
   (stdin/stdout/stderr all passthrough)

4. On exit, session is recorded
   ~/.agent-sandbox/workspaces/.../logs/<session-id>.json
```

---

## Sessions Location

```bash
~/.agent-sandbox/
├── workspaces/
│   ├── opencode-20260315-144537/
│   │   ├── .sandbox-merged/      # agent workspace
│   │   └── logs/
│   │       └── <session-id>.json  # recorded session
│   └── claude-20260315-123000/
│       └── logs/
├── imports/                       # imported sessions
└── sandboxes.db                   # sqlite database
```

---

## Supported Agents

Any CLI binary works:

- `abox claude` — Claude Code
- `abox codex` — OpenAI Codex
- `abox opencode` — OpenCode
- `abox gemini` — Google Gemini
- `abox aider` — Aider
- `abox <anything>` — any binary

---

## License

MIT
