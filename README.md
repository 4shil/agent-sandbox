#  Abox

> Isolated, Auditable, Portable AI Agent Runtime

**Docker for AI agents** — but lightweight, fast, and built for code generation.

[![CI](https://github.com/4shil/agent-sandbox/actions/workflows/ci.yml/badge.svg)](https://github.com/4shil/agent-sandbox/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Release](https://img.shields.io/github/v/release/4shil/agent-sandbox)](https://github.com/4shil/agent-sandbox/releases)

---

## Why Abox?

You let Claude Code / Cursor / Codex build something on your machine. It:

- 🔓 Runs as **you** — reads your SSH keys, env vars, everything
- 🗑️ Commits **47 times** with garbage messages
- ❌ Leaves **no audit trail** — "why did it delete this?"
- 📦 Can't be **shared** with teammates
- ⏱️ Eats **all your RAM** with no limits

**abox fixes all of this.**

```
$ abox init my-project --template node
$ abox run --agent claude "Build a REST API" --memory 2gb
$ abox replay my-project     ← see everything it did
$ abox export my-project -o share.tar.gz  ← share with team
```

---

## Install

### One-liner

```bash
curl -sSL https://raw.githubusercontent.com/4shil/agent-sandbox/main/install.sh | bash
```

### Manual

```bash
# Download from releases
wget https://github.com/4shil/agent-sandbox/releases/latest/download/abox-x86_64-unknown-linux-gnu.tar.gz
tar -xzf abox-*.tar.gz
sudo mv abox /usr/local/bin/
```

---

## Quick Start

### 1. Create a sandbox

```bash
$ abox init my-app --template rust

🛡️  Creating sandbox my-app
✅ Sandbox 'my-app' ready

  cd ~/.agent-sandbox/workspaces/my-app
  abox run "your task"
```

### 2. Run an agent

```bash
$ abox run --agent claude "Add user authentication" --memory 2gb --timeout 30m

🔌 Initializing sandbox filesystem...
📝 Starting session recorder...
🛡️  Resource limits: MEM: 2.0GB, TIME: 1800s
🌐 Network: OPEN
🤖 Launching claude with task: Add user authentication
   Session ID: abc123...

✅ Task completed in 45.2s
   Actions recorded: 127
```

### 3. Replay what happened

```bash
$ abox replay my-app

╔══════════════════════════════════════════════════════════╗
║  🔄 SESSION REPLAY                                       ║
║  Agent: claude                                           ║
║  Task: Add user authentication                           ║
║  Actions: 127                                            ║
╚══════════════════════════════════════════════════════════╝

┌─ [1/127] ─────────────────────────────────────────
│ Type:      task_start
│ Task: Add user authentication
└────────────────────────────────────────────────────
Actions: [n]ext, [p]rev, [j]ump, [d]etails, [q]uit >
```

### 4. Share with anyone

```bash
$ abox export my-app -o session.tar.gz
📦 Exporting session...
   Output: session.tar.gz (1.2MB)
✅ Export complete

# The tar.gz includes:
# - session.json    (full action log)
# - metadata.json   (agent, host, duration)
# - replay.html     (browser-based replay viewer)
# - files/          (all modified files)
```

---

## Commands

### `abox init`

Create a new sandbox workspace.

```bash
abox init <name> --template <node|python|rust|empty>
```

**Options:**
| Flag | Description | Default |
|------|-------------|---------|
| `--template` | Project template | `empty` |

**Templates:**
- `empty` — blank workspace
- `node` — package.json + index.js
- `python` — pyproject.toml + main.py
- `rust` — Cargo.toml + src/main.rs

---

### `abox status`

List all active sandboxes.

```bash
abox status
```

Output:
```
📦 Active Sandboxes

  NAME                 AGENT        CREATED
  ──────────────────────────────────────────────────
  my-app               claude       2026-03-15 08:37:31
  api-project          codex        2026-03-15 07:12:00
```

---

### `abox run`

Run an agent task in a sandbox.

```bash
abox run --agent <agent> <task> [options]
```

**Options:**
| Flag | Description | Default |
|------|-------------|---------|
| `-a, --agent` | Agent to use | `claude` |
| `-s, --sandbox` | Sandbox name | first active |
| `--memory` | Max memory (e.g., `2gb`, `512mb`) | `2gb` |
| `--cpu` | Max CPU time (e.g., `10m`, `1h`) | `10m` |
| `--timeout` | Max wall time (e.g., `30m`) | `30m` |
| `--no-network` | Block all network access | false |
| `--allow-domain` | Whitelist domain (repeatable) | — |

**Examples:**
```bash
# Basic
abox run --agent claude "Fix the bug in auth.js"

# With limits
abox run --agent codex "Refactor database layer" --memory 1gb --timeout 15m

# Offline (no network)
abox run --agent claude "Review code" --no-network

# Allow specific domains only
abox run --agent claude "Deploy" --no-network --allow-domain api.github.com
```

---

### `abox diff`

Show clean diff of all file changes from a session.

```bash
abox diff <session-id-or-name>
```

Output:
```
📊 Session Diff

📁 src/auth.ts
  + import bcrypt from 'bcrypt';
  + export async function hashPassword(password: string) {
  +   return bcrypt.hash(password, 10);
  + }

📁 src/routes.ts
  + import { hashPassword } from './auth';
  + app.post('/register', registerHandler);
```

---

### `abox replay`

Interactive step-through replay of a session.

```bash
abox replay <session-id-or-name>
```

**Keyboard controls:**
| Key | Action |
|-----|--------|
| `n` / `→` | Next action |
| `p` / `←` | Previous action |
| `j` | Jump to action # |
| `d` | Show full action data |
| `q` | Quit |

---

### `abox export`

Export a session as a portable tar.gz archive.

```bash
abox export <session-id-or-name> -o <output-file>
```

**Archive contents:**
```
session.tar.gz
├── session.json       # Full action log (JSON)
├── metadata.json      # Agent, host, duration
├── replay.html        # Browser-based replay viewer
└── files/             # All modified files
```

---

### `abox import`

Import a shared session archive.

```bash
abox import <file.tar.gz>
```

Extracts to `~/.agent-sandbox/imports/` and displays session info.

---

### `abox inspect`

View detailed session statistics and action breakdown.

```bash
abox inspect <session-id-or-name>
```

Output:
```
🔍 Session Inspector

  ID: abc123...
  Agent: claude
  Task: Build REST API
  Sandbox: my-app
  Duration: 45200ms
  Actions: 127
  OS: linux
  Arch: x86_64

  Action Breakdown:
    task_start           1 ████████████████████
    file_write          34 ████████████████████
    exec                45 ████████████████████
    llm_call            47 ████████████████████
```

---

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                      abox                           │
├─────────────┬─────────────┬─────────────────────────┤
│   Sandbox   │  Recorder   │         Limits          │
│   (FUSE)    │  (SQLite)   │     (cgroups)           │
├─────────────┼─────────────┼─────────────────────────┤
│  overlay FS │ file ops    │  memory: 2gb            │
│  upper dir  │ exec logs   │  cpu: 10m               │
│  merged     │ LLM calls   │  timeout: 30m           │
│             │ network     │  network: blocked       │
└─────────────┴─────────────┴─────────────────────────┘
```

---

## File Locations

| Path | Description |
|------|-------------|
| `~/.agent-sandbox/sandboxes.db` | SQLite database |
| `~/.agent-sandbox/workspaces/<name>/` | Workspace files |
| `~/.agent-sandbox/workspaces/<name>/logs/` | Session recordings |
| `~/.agent-sandbox/imports/` | Imported sessions |

---

## Tech Stack

| Layer | Tech | Why |
|-------|------|-----|
| CLI | **Rust** (clap) | Fast, single binary |
| FS Sandbox | **FUSE overlay** | Isolated filesystem |
| Recording | **SQLite** | Lightweight, queryable |
| Process Isolation | **Linux namespaces** | No Docker needed |
| Network Control | **iptables** | Fine-grained control |
| Replay Viewer | **HTML + htmx** | Zero-dependency |
| UI | **colored** crate | Colored terminal output |

---

## Use Cases

| Use Case | Command |
|----------|---------|
| 🎓 Education | `abox replay lesson-1` — watch how code was built |
| 🔍 Code Review | `abox diff pr-123` — see exactly what changed |
| 🐛 Debugging | `abox inspect broken-session` — find where it went wrong |
| 📝 Documentation | Export session → replay.html is auto-generated docs |
| 🔒 Security Audit | `abox inspect` — full audit trail of agent actions |
| 🤝 Collaboration | `abox export` → share → teammate imports and replays |

---

## Supported Agents

abox works with **any** AI coding agent:

| Agent | Command | Description |
|-------|---------|-------------|
| ✅ Claude Code | `--agent claude` | Anthropic Claude Code |
| ✅ Codex | `--agent codex` | OpenAI Codex CLI |
| ✅ OpenCode | `--agent opencode` | OpenCode AI agent |
| ✅ Cursor | `--agent cursor` | Cursor AI Editor |
| ✅ Gemini | `--agent gemini` | Google Gemini CLI |
| ✅ Aider | `--agent aider` | Aider AI pair programmer |
| ✅ Goose | `--agent goose` | Block Goose agent |
| ✅ Sweep | `--agent sweep` | Sweep AI |
| ✅ Any binary | `--agent myagent` | Custom agent support |

```bash
# List all supported agents
abox agents

# Use any supported agent
abox run --agent opencode "Build a REST API"
abox run --agent claude "Fix the bug" --memory 2gb
abox run --agent aider "Add tests" --timeout 15m
```

---

## License

MIT — see [LICENSE](LICENSE) for details.

## Contributing

PRs welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.
