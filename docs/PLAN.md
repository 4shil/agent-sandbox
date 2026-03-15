# Agent Sandbox — Full Development Plan

> Isolated, Auditable, Portable AI Agent Runtime

**Created:** 2026-03-15
**Status:** Planning
**Difficulty:** Medium-Hard
**Target Timeline:** 2-3 weeks to MVP

---

## 1. Vision

An open-source runtime that wraps any AI coding agent (Claude Code, Cursor, Copilot, Codex, OpenClaw) in a safe, recordable, replayable, and shareable execution environment.

Think: **Docker for AI agent sessions** — but lighter, faster, and purpose-built for code-generation agents.

---

## 2. Core Problem

When you let an AI agent work on your machine today:

| Risk | What Happens |
|------|--------------|
| 🔓 No sandbox | Agent runs as YOU — can access SSH keys, env vars, everything |
| 🗑️ Messy commits | Agent dumps 47 commits, half broken, no clear diff |
| ❌ No audit trail | "Why did the agent delete this?" — impossible to trace |
| 🔁 Not reproducible | Can't replay what the agent did |
| 📦 No sharing | Can't send your agent session to a friend |
| ⏱️ No resource limits | Agent eats 8GB RAM with `npm install` |

---

## 3. Core Features

### 3.1 Filesystem Sandbox
- FUSE overlay filesystem — agent sees fake FS, real changes isolated
- Agent writes go to overlay, real FS untouched until approved
- Support for whitelisted paths (e.g., project directory)

### 3.2 Deterministic Replay
- Record every tool call, LLM response, file mutation
- Replay sessions step-by-step
- HTML viewer for browser-based replay

### 3.3 Portable Sessions
- Export sessions as tar.gz
- Import and replay on any machine
- Includes: action log, diffs, metadata, optional file states

### 3.4 Resource Limits
- CPU, memory, network, time constraints per task
- Linux cgroups for enforcement
- Network blocking (whitelist domains)

### 3.5 Clean Diffs
- Agent produces clean diffs, not garbage commits
- One diff per session showing all changes
- Optional auto-commit with clean messages

---

## 4. Tech Stack

| Layer | Tech | Why |
|-------|------|-----|
| CLI | **Rust** (clap) | Fast startup, single binary |
| FUSE | **fuser** crate | Filesystem overlay |
| Recording | **SQLite** | Lightweight, queryable |
| Process isolation | **Linux namespaces + cgroups** | No Docker dependency |
| Network | **iptables/nftables** wrapper | Block/allow per-domain |
| Replay viewer | **HTML + htmx** | Zero-dependency browser replay |
| Streaming | **WebSocket** | Real-time TUI output |

---

## 5. Architecture

```
                        ┌─────────────────────────┐
                        │      agent-sandbox       │
                        │    CLI / SDK / Daemon     │
                        └────────────┬────────────┘
                                     │
          ┌──────────────────────────┼──────────────────────────┐
          │                          │                          │
   ┌──────▼──────┐          ┌───────▼───────┐          ┌──────▼──────┐
   │  Sandboxing  │          │   Recording   │          │  Execution  │
   │   Engine     │          │    Engine     │          │   Engine    │
   └──────┬──────┘          └───────┬───────┘          └──────┬──────┘
          │                          │                          │
   ┌──────┴──────┐          ┌───────┴───────┐          ┌──────┴──────┐
   │             │          │               │          │             │
┌──▼──┐    ┌────▼────┐  ┌──▼──┐      ┌────▼────┐  ┌──▼──┐    ┌────▼────┐
│FUSE │    │Network  │  │SQLite│      │Session  │  │Process│   │Resource │
│FS   │    │Blocker  │  │ Log  │      │Exporter │  │Manager│   │Limiter  │
└─────┘    └─────────┘  └──────┘      └─────────┘  └──────┘   └─────────┘
```

---

## 6. CLI Commands

```
agent-sandbox
├── init <name>          Create sandbox workspace
├── run --agent <name>   Run agent in sandbox
├── status              Show active sandboxes
├── diff <session>      Show clean diff
├── replay <session>    Replay session (TUI or HTML)
├── export <session>    Package session for sharing
├── import <file>       Load shared session
├── inspect <session>   Deep-dive into action log
├── kill <sandbox>      Force-stop running sandbox
└── config              Manage settings
```

---

## 7. Workflow

### Step 1: Init
```bash
agent-sandbox init my-project --template node
```
Creates: FUSE overlay, SQLite logger, resource limits, network rules.

### Step 2: Run
```bash
agent-sandbox run --agent claude "Build a todo API with Express"
```
Everything logged: file writes, commands, outputs, durations.

### Step 3: Review
```bash
agent-sandbox diff my-project      # Clean diff only
agent-sandbox replay my-project    # Step-by-step replay
```

### Step 4: Export & Share
```bash
agent-sandbox export my-project --output session.tar.gz
```

---

## 8. Session Export Format

```
session.tar.gz
├── session.json           # Full action log
├── diff.patch             # Clean diff
├── metadata.json          # Agent, model, duration, cost
├── files/
│   ├── initial/           # Starting state
│   └── final/             # Ending state
└── replay.html            # Interactive replay viewer
```

---

## 9. Use Cases

### 🎓 Education
Professor sends agent session: "Watch Claude build a binary tree" — students replay and learn

### 🔍 Code Review
"Here's what the agent did to refactor auth" — reviewer replays every decision

### 🐛 Debugging
Agent broke production? Replay the session, find exactly where it went wrong

### 📝 Documentation
Auto-generate "how this was built" docs from replay

### 🤝 Collaboration
"Try my agent session but make it handle edge cases" — fork and continue

### 🔒 Security Audit
Full filesystem/network audit trail of what the agent accessed

---

## 10. MVP Scope (2-3 weeks)

### Week 1: Core Infrastructure
- [ ] Project scaffold (Rust, clap CLI)
- [ ] FUSE overlay filesystem (read/write overlay)
- [ ] Session SQLite schema + logger
- [ ] Basic `init` and `run` commands

### Week 2: Recording + Replay
- [ ] Tool call interception (file ops, exec)
- [ ] Session recorder (log every action)
- [ ] `diff` command (clean diff output)
- [ ] `replay` TUI (terminal step-by-step)

### Week 3: Polish + Export
- [ ] Session export/import (tar.gz)
- [ ] HTML replay viewer (standalone)
- [ ] Resource limits (cgroups)
- [ ] Network blocking
- [ ] README + demo.gif
- [ ] Publish to GitHub

---

## 11. Post-MVP Features

- [ ] Docker backend option (for macOS/Windows)
- [ ] VS Code extension (replay in editor)
- [ ] GitHub Action (auto-replay PR agent sessions)
- [ ] Cloud sync (share sessions via URL)
- [ ] Agent-agnostic adapter (works with any LLM)
- [ ] Cost tracking per session
- [ ] Session comparison (diff two sessions)

---

## 12. Why

| Factor | Why |
|--------|-----|
| Timing | Agent safety is THE hot topic |
| Audience | Every AI dev needs this |
| Demo | Screen recording = instant virality |
| Competition | Nothing OSS does all 5 features |
| Tags | ai, sandbox, agents, security, reproducibility |

---

## 13. Repo Structure

```
agent-sandbox/
├── Cargo.toml
├── README.md
├── assets/
│   └── demo.gif
├── src/
│   ├── main.rs
│   ├── cli/
│   │   ├── mod.rs
│   │   ├── init.rs
│   │   ├── run.rs
│   │   ├── diff.rs
│   │   ├── replay.rs
│   │   ├── export.rs
│   │   └── import.rs
│   ├── sandbox/
│   │   ├── mod.rs
│   │   ├── fuse_overlay.rs
│   │   ├── namespace.rs
│   │   └── network.rs
│   ├── recorder/
│   │   ├── mod.rs
│   │   ├── schema.rs
│   │   └── logger.rs
│   ├── replay/
│   │   ├── mod.rs
│   │   └── tui.rs
│   └── export/
│       ├── mod.rs
│       └── packaging.rs
├── templates/
│   ├── node/
│   ├── python/
│   └── rust/
└── viewer/
    └── replay.html
```
