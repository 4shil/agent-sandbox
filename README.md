# 🛡️ Agent Sandbox

> Isolated, Auditable, Portable AI Agent Runtime

**Docker for AI agent sessions** — but lighter, faster, and built for code-generation agents.

[![CI](https://github.com/4shil/agent-sandbox/actions/workflows/ci.yml/badge.svg)](https://github.com/4shil/agent-sandbox/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

---

## The Problem

When you let an AI agent (Claude Code, Cursor, Copilot) work on your machine:

| Risk | What Happens |
|------|--------------|
| 🔓 **No sandbox** | Agent runs as YOU — can access SSH keys, env vars |
| 🗑️ **Messy commits** | 47 garbage commits, no clear diff |
| ❌ **No audit trail** | "Why did the agent delete this?" impossible to trace |
| 🔁 **Not reproducible** | Can't replay what the agent did |
| 📦 **No sharing** | Can't send your session to a friend |
| ⏱️ **No limits** | Agent eats all your RAM |

## The Solution

Agent Sandbox wraps any AI agent in a safe, recordable, replayable environment:

```
$ agent-sandbox init my-project --template node
✅ Sandbox 'my-project' created with Node template

$ agent-sandbox run --agent claude "Build a todo API" --memory 2gb --timeout 30m
🔌 Initializing sandbox filesystem...
📝 Starting session recorder...
🛡️  Resource limits: MEM: 2.0GB, TIME: 1800s
✅ Task completed in 45.2s
   Actions recorded: 127

$ agent-sandbox diff my-project
📁 src/index.ts
  + import express from 'express';
  + const app = express();

$ agent-sandbox replay my-project
╔══════════════════════════════════════════════════════════╗
║  🔄 SESSION REPLAY                                       ║
║  Agent: claude                                           ║
║  Task: Build a todo API                                  ║
║  Actions: 127                                            ║
╚══════════════════════════════════════════════════════════╝
[n]ext, [p]rev, [j]ump, [d]etails, [q]uit >

$ agent-sandbox export my-project -o session.tar.gz
📦 Exporting session...
   Output: session.tar.gz (1.2MB)
✅ Export complete
```

## Features

| Feature | Description |
|---------|-------------|
| 📁 **FUSE Sandbox** | Agent sees isolated filesystem, real changes recorded |
| 🔄 **Session Replay** | Record every action, replay step-by-step |
| 📦 **Portable Sessions** | Export as tar.gz, share anywhere |
| 🛡️ **Resource Limits** | CPU, memory, timeout, disk constraints |
| 🌐 **Network Control** | Block or whitelist specific domains |
| 📊 **Clean Diffs** | One clean diff, not 47 garbage commits |
| 🌐 **HTML Viewer** | Browser-based replay, works offline |

## Quick Start

### Install

```bash
curl -sSL https://raw.githubusercontent.com/4shil/agent-sandbox/main/install.sh | bash
```

### Usage

```bash
# Create a sandbox
agent-sandbox init my-project --template node

# Run an agent task
agent-sandbox run --agent claude "Build a REST API"

# Review changes
agent-sandbox diff my-project

# Replay the session
agent-sandbox replay my-project

# Share with others
agent-sandbox export my-project -o session.tar.gz
```

## Templates

- `empty` — blank workspace
- `node` — package.json + index.js
- `python` — pyproject.toml + main.py
- `rust` — Cargo.toml + src/main.rs

## Resource Limits

```bash
agent-sandbox run --agent claude "task" \
  --memory 2gb \
  --cpu 10m \
  --timeout 30m
```

## Network Control

```bash
# Block all network
agent-sandbox run --agent claude "task" --no-network

# Block all except specific domains
agent-sandbox run --agent claude "task" \
  --no-network \
  --allow-domain api.openai.com \
  --allow-domain github.com
```

## Use Cases

- 🎓 **Education** — Replay agent sessions to learn coding patterns
- 🔍 **Code Review** — Review exactly what the agent did
- 🐛 **Debugging** — Find where the agent broke things
- 📝 **Docs** — Auto-generate "how this was built" from replay
- 🔒 **Security** — Full audit trail of agent actions

## Tech Stack

- **Rust** — Fast CLI, single binary
- **FUSE** — Filesystem sandboxing
- **SQLite** — Session recording
- **Linux namespaces** — Process isolation
- **htmx** — Browser replay viewer

## License

MIT — see [LICENSE](LICENSE) for details.
