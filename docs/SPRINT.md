# Agent Sandbox — Sprint Plan

> 10 commits, 3 phases, 2-3 weeks to MVP

**Created:** 2026-03-15
**Total Commits:** ~25-30
**Branch:** `main`

---

## Phase 1: Foundation (Commits 1-8) 🔴 PRIORITY: CRITICAL

> Get the skeleton working — CLI, FUSE overlay, basic logging

### Commit 1: Project Scaffold
**Priority:** 🔴 P0
```
- cargo init agent-sandbox
- Add dependencies: clap, tokio, serde, sqlite
- Basic main.rs with clap CLI
- README with usage stub
```

### Commit 2: SQLite Session Schema
**Priority:** 🔴 P0
```
- Create sessions table (id, name, agent, started_at, ended_at)
- Create actions table (id, session_id, type, timestamp, data)
- Create files table (id, session_id, path, content, action)
- Migration logic in code
```

### Commit 3: `init` Command
**Priority:** 🔴 P0
```
- agent-sandbox init <name> --template <node|python|rust>
- Creates workspace at ~/.agent-sandbox/workspaces/<name>/
- Copies template files
- Initializes SQLite database
- Creates config.json
```

### Commit 4: FUSE Overlay Filesystem
**Priority:** 🔴 P0
```
- Lower dir: original project
- Upper dir: agent writes go here
- Work dir: FUSE internal
- Agent sees unified view, real FS untouched
```

### Commit 5: `run` Command (Basic)
**Priority:** 🔴 P0
```
- agent-sandbox run --agent <claude|codex|cursor> "task"
- Spawns agent process inside sandbox
- Captures stdout/stderr
- Logs start/end times
```

### Commit 6: Action Logger
**Priority:** 🔴 P0
```
- Intercept file write operations
- Log to SQLite: timestamp, action_type, path, diff
- Log exec commands + outputs
- Log LLM tool calls
```

### Commit 7: `status` Command
**Priority:** 🟡 P1
```
- agent-sandbox status
- List active sandboxes
- Show: name, agent, duration, actions count
- Show resource usage (CPU, memory)
```

### Commit 8: Phase 1 Testing + Cleanup
**Priority:** 🟡 P1
```
- Test init → run → status flow
- Error handling improvements
- Unit tests for SQLite operations
- Commit: phase-1-complete
```

---

## Phase 2: Recording + Replay (Commits 9-16) 🔴 PRIORITY: HIGH

> Make sessions actually useful — record everything, replay anything

### Commit 9: Full Session Recorder
**Priority:** 🔴 P0
```
- Record ALL tool calls (file read/write, exec, network)
- Record LLM responses + token usage
- Record file mutations with before/after
- JSON structured logging
```

### Commit 10: `diff` Command
**Priority:** 🔴 P0
```
- agent-sandbox diff <session>
- Show clean unified diff of all changes
- Filter by file path
- Colorized output
```

### Commit 11: `replay` TUI (Terminal)
**Priority:** 🔴 P0
```
- agent-sandbox replay <session>
- Step through actions one by one
- Show file state at each step
- Keyboard navigation (n=next, p=prev, q=quit)
- Use ratatui for TUI
```

### Commit 12: Session Inspector
**Priority:** 🟡 P1
```
- agent-sandbox inspect <session>
- Show full action log
- Filter by action type
- Search by file path or content
- JSON output option
```

### Commit 13: Resource Limits (cgroups)
**Priority:** 🟡 P1
```
- --memory 2GB flag
- --cpu 1 flag
- --timeout 30m flag
- Kill process if limits exceeded
- Log resource violations
```

### Commit 14: Network Blocking
**Priority:** 🟡 P1
```
- --no-network flag (block all)
- --allow <domain> flag (whitelist)
- Uses iptables/nftables under the hood
- Log network attempts
```

### Commit 15: Phase 2 Integration Test
**Priority:** 🟡 P1
```
- Full workflow test: init → run → diff → replay
- Test with real Claude Code session
- Performance: ensure <100ms overhead per action
- Fix edge cases
```

### Commit 16: Phase 2 Polish
**Priority:** 🟢 P2
```
- Help text for all commands
- Better error messages
- Progress indicators for long operations
- Man pages
```

---

## Phase 3: Export + Ship (Commits 17-24) 🟡 PRIORITY: MEDIUM

> Make it shareable, pretty, and ready for GitHub

### Commit 17: Session Export
**Priority:** 🔴 P0
```
- agent-sandbox export <session> -o session.tar.gz
- Package: session.json, diff.patch, metadata.json, files/
- Compression with tar + gzip
- Optional: include initial/final file states
```

### Commit 18: Session Import
**Priority:** 🔴 P0
```
- agent-sandbox import session.tar.gz
- Extract and register in local sessions
- Validate session integrity (checksum)
- List imported sessions
```

### Commit 19: HTML Replay Viewer
**Priority:** 🟡 P1
```
- agent-sandbox export --html session.tar.gz
- Standalone replay.html in package
- htmx + minimal CSS
- Step-through UI with file diff viewer
- Works offline, no server needed
```

### Commit 20: Templates
**Priority:** 🟢 P2
```
- Node.js template (package.json, index.js)
- Python template (pyproject.toml, main.py)
- Rust template (Cargo.toml, main.rs)
- Empty template
```

### Commit 21: Install Script
**Priority:** 🟡 P1
```
- curl install.sh | bash
- Detects architecture (x86_64, aarch64)
- Downloads correct binary from GitHub releases
- Adds to PATH
```

### Commit 22: GitHub Actions CI
**Priority:** 🟡 P1
```
- Build on push (Linux, macOS)
- Run tests
- Create release on tag
- Upload binaries
```

### Commit 23: README + Demo
**Priority:** 🔴 P0
```
- Full README with examples
- Demo GIF (record terminal session)
- Architecture diagram
- Comparison table (vs Docker, vs nothing)
- Contributing guide
```

### Commit 24: v0.1.0 Release 🚀
**Priority:** 🔴 P0
```
- Tag v0.1.0
- GitHub release with binaries
- Post on HN, Reddit, Twitter
- Submit to "Awesome AI Tools" lists
```

---

## Phase 4: Post-MVP (Backlog) 🟢 PRIORITY: LOW

> Nice-to-haves for v0.2+

| Feature | Priority | Effort |
|---------|----------|--------|
| macOS/Windows support (Docker backend) | 🟢 P2 | Large |
| VS Code extension | 🟢 P2 | Medium |
| GitHub Action (auto-replay PRs) | 🟡 P1 | Small |
| Cloud sync (share via URL) | 🟢 P2 | Large |
| Session comparison (diff two sessions) | 🟡 P1 | Small |
| Cost tracking per session | 🟢 P2 | Small |
| TUI dashboard for all sandboxes | 🟢 P2 | Medium |

---

## Priority Legend

| Icon | Priority | Meaning |
|------|----------|---------|
| 🔴 | P0 | Must have — blocks MVP |
| 🟡 | P1 | Should have — important for usability |
| 🟢 | P2 | Nice to have — post-MVP |

---

## Commit Convention

```
feat: add FUSE overlay filesystem
fix: handle race condition in session logger
docs: update README with install instructions
test: add integration tests for init flow
refactor: extract SQLite operations to module
```

---

## Timeline

```
Week 1:  Commits 1-8   → Foundation (CLI + FUSE + logging)
Week 2:  Commits 9-16  → Recording + Replay
Week 3:  Commits 17-24 → Export + Ship + v0.1.0
```

---

## Definition of Done (per commit)

- [ ] Code compiles without warnings
- [ ] Manual test passes
- [ ] Commit message follows convention
- [ ] Pushed to `main`
