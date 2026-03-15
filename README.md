# 🛡️ Agent Sandbox

> Isolated, Auditable, Portable AI Agent Runtime

**Docker for AI agent sessions** — but lighter, faster, and purpose-built for code-generation agents.

## The Problem

When you let an AI agent (Claude Code, Cursor, Copilot) work on your machine:

- 🔓 **No sandbox** — agent runs as YOU, can access SSH keys, env vars
- 🗑️ **Messy commits** — 47 garbage commits, no clear diff
- ❌ **No audit trail** — "Why did the agent delete this?" impossible to trace
- 🔁 **Not reproducible** — can't replay what the agent did
- 📦 **No sharing** — can't send your session to a friend
- ⏱️ **No resource limits** — agent eats all your RAM

## The Solution

Agent Sandbox wraps any AI agent in a safe, recordable, replayable environment:

| Feature | What It Does |
|---------|--------------|
| **FUSE Sandboxing** | Agent sees fake filesystem, real changes isolated |
| **Deterministic Replay** | Record every tool call, replay later |
| **Portable Sessions** | Export tar.gz, share, replay anywhere |
| **Resource Limits** | CPU, memory, network, time constraints |
| **Clean Diffs** | One clean diff, not 47 garbage commits |

## Quick Start

```bash
# Install
curl -sSL https://raw.githubusercontent.com/4shil/agent-sandbox/main/install.sh | bash

# Initialize sandbox
agent-sandbox init my-project --template node

# Run agent
agent-sandbox run --agent claude "Build a todo API"

# Review
agent-sandbox diff my-project
agent-sandbox replay my-project

# Share
agent-sandbox export my-project --output session.tar.gz
```

## CLI Commands

```
agent-sandbox init <name>     Create sandbox workspace
agent-sandbox run             Run agent in sandbox
agent-sandbox status          Show active sandboxes
agent-sandbox diff <session>  Show clean diff
agent-sandbox replay <session> Replay session
agent-sandbox export          Package session for sharing
agent-sandbox import          Load shared session
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
- **HTML + htmx** — Browser replay viewer

## Status

🚧 **In Development** — MVP coming soon

## License

MIT
