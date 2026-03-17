# abox Multi-Agent Sandbox Architecture

## Concept

Run multiple AI assistant instances (OpenClaw, etc.) in isolated sandboxes simultaneously,
each with its own workspace as its entire visible filesystem.

## User Flow

```
$ abox

  ╔══════════════════════════════════════╗
  ║         abox — select mode           ║
  ╠══════════════════════════════════════╣
  ║                                      ║
  ║   [1] 🤖 AI Assistant                ║
  ║       Run OpenClaw in sandbox        ║
  ║       Multiple instances allowed     ║
  ║                                      ║
  ║   [2] 💻 Coding Agent                ║
  ║       Claude / Codex / etc.          ║
  ║       Current sandbox behavior       ║
  ║                                      ║
  ╚══════════════════════════════════════╝
```

### Coding Agent path → Current behavior
```
abox claude → sandbox workspace → claude runs in it → session recorded
```

### AI Assistant path → New behavior
```
abox assistant → pick agent (openclaw) → isolated workspace → agent sees ONLY that workspace
                                        → terminal launched inside
                                        → HOME=/workspace (can't see outside)
                                        → can run 10+ instances simultaneously
```

---

## Architecture Options

### Option A: Environment Variable Isolation (Simplest)

Redirect all "home" pointers to workspace subdirectories.

```
Workspace: ~/.agent-sandbox/assistants/openclaw-instance-1/

Environment set:
  HOME=~/.agent-sandbox/assistants/openclaw-instance-1/
  XDG_CONFIG_HOME=~/.agent-sandbox/assistants/openclaw-instance-1/.config
  XDG_DATA_HOME=~/.agent-sandbox/assistants/openclaw-instance-1/.local/share
  XDG_STATE_HOME=~/.agent-sandbox/assistants/openclaw-instance-1/.local/state
  OPENCLAW_STATE_DIR=~/.agent-sandbox/assistants/openclaw-instance-1/.openclaw
```

**Pros:**
- Zero dependencies, works everywhere
- No root needed
- Easy to implement
- OpenClaw respects all these vars

**Cons:**
- Relies on the application respecting env vars
- Not true filesystem-level isolation
- A rogue `cd /home/ashil` in the terminal still works (but the shell's HOME is workspace)

**Mitigation:**
- Launch shell with restricted config (no `.bashrc` sourcing)
- Use `chdir()` to workspace as working directory
- Combine with sandboxed shell profile

**Verdict:** Good enough for 95% of cases. Fastest to implement.

---

### Option B: Linux Namespaces (Proper Isolation)

Use `unshare` to create mount + PID namespaces.

```
unshare --mount --pid --fork --
  # Mount workspace as new root
  mount --bind workspace /mnt
  cd /mnt
  # Hide host filesystem
  mount -t tmpfs tmpfs /mnt
  # Pivot to workspace root
  pivot_root /mnt /mnt/host
  umount /mnt/host
  # Now / IS the workspace
  exec openclaw
```

**Pros:**
- True filesystem isolation
- Agent literally cannot see outside workspace
- No trust required in the application

**Cons:**
- Requires `unshare` capability (usually available on modern Linux)
- May need `--map-root-user` for user namespaces
- More complex setup
- Needs to handle `/dev`, `/proc`, `/tmp` mounts

**Verdict:** Best isolation but more complex. Good for v2.

---

### Option C: Bubblewrap (bwrap) — Sweet Spot

Uses `bubblewrap` (sandboxing tool, available on most Linux distros).

```bash
bwrap \
  --ro-bind / / \
  --dev /dev \
  --proc /proc \
  --tmpfs /tmp \
  --bind workspace /home/user \
  --chdir /home/user \
  --unshare-pid \
  --die-with-parent \
  -- setuid openclaw
```

**Pros:**
- Purpose-built for sandboxing
- No root needed (setuid wrapper)
- Clean, well-tested approach
- Handles all the edge cases

**Cons:**
- Requires `bubblewrap` package installed
- Extra dependency

**Verdict:** Best if bwrap is available. Fallback to Option A.

---

## Recommended Architecture: Hybrid (A → B → C fallback)

```
┌─────────────────────────────────────────────────────┐
│                    abox main                         │
│                                                      │
│  ┌──────────────┐    ┌────────────────────────────┐ │
│  │ Coding Agent  │    │    AI Assistant Mode       │ │
│  │              │    │                            │ │
│  │ Current flow │    │ 1. Detect isolation method │ │
│  │ abox claude  │    │    bwrap > unshare > env   │ │
│  │ abox codex   │    │                            │ │
│  │              │    │ 2. Create workspace        │ │
│  └──────────────┘    │    ~/.agent-sandbox/       │ │
│                      │    assistants/<name>-<id>/ │ │
│                      │                            │ │
│                      │ 3. Setup isolation         │ │
│                      │    (best available method) │ │
│                      │                            │ │
│                      │ 4. Launch agent            │ │
│                      │    with terminal           │ │
│                      │                            │ │
│                      │ 5. Record session          │ │
│                      └────────────────────────────┘ │
└─────────────────────────────────────────────────────┘
```

---

## Directory Structure

```
~/.agent-sandbox/
├── workspaces/          # Coding agent sandboxes (current)
│   ├── claude-20260317-120000/
│   └── codex-20260317-130000/
│
├── assistants/          # AI assistant sandboxes (NEW)
│   ├── openclaw-main/           # "main" instance
│   │   ├── .openclaw/           # OpenClaw state (isolated)
│   │   ├── .config/             # App configs
│   │   ├── .local/share/        # App data
│   │   ├── workspace/           # Working files
│   │   └── session.jsonl        # Recorded session
│   │
│   ├── openclaw-coding/         # "coding" instance
│   │   ├── .openclaw/
│   │   └── ...
│   │
│   └── openclaw-research/       # "research" instance
│       ├── .openclaw/
│       └── ...
│
└── logs/                # Session logs
```

---

## New CLI Commands

```
# Launch assistant mode
abox assistant                    # Interactive picker
abox assistant openclaw           # Direct launch
abox assistant openclaw --name main  # Named instance

# Manage instances
abox assistant list               # List running instances
abox assistant kill <name>        # Kill instance
abox assistant logs <name>        # View session logs
abox assistant attach <name>      # Reattach to running instance

# Dashboard integration
abox dashboard                    # Shows both coding + assistant sessions
```

---

## Isolation Levels (Implementation)

### Level 1: Environment Isolation (Minimum viable)
```rust
fn setup_env_isolation(workspace: &Path) {
    std::env::set_var("HOME", workspace);
    std::env::set_var("XDG_CONFIG_HOME", workspace.join(".config"));
    std::env::set_var("XDG_DATA_HOME", workspace.join(".local/share"));
    std::env::set_var("XDG_STATE_HOME", workspace.join(".local/state"));
    std::env::set_var("OPENCLAW_STATE_DIR", workspace.join(".openclaw"));
}
```

### Level 2: Shell Sandbox (Prevent cd out)
```rust
fn launch_sandboxed_shell(workspace: &Path) -> Command {
    let mut cmd = Command::new("bash");
    cmd.arg("--norc");           // Don't load user's .bashrc
    cmd.arg("--noprofile");      // Don't load user's .profile
    cmd.arg("-c").arg(format!(
        "export HOME={}; cd {}; exec openclaw",
        workspace.display(),
        workspace.display(),
    ));
    cmd.current_dir(workspace);
    cmd
}
```

### Level 3: Filesystem Sandbox (True isolation)
```rust
fn launch_with_bwrap(workspace: &Path, agent: &str) -> Command {
    let mut cmd = Command::new("bwrap");
    cmd.args([
        "--ro-bind", "/", "/",           // Read-only host root
        "--bind", workspace, workspace,  // Writable workspace
        "--dev", "/dev",
        "--proc", "/proc",
        "--tmpfs", "/tmp",
        "--chdir", workspace,
        "--unshare-pid",
        "--die-with-parent",
        "--setenv", "HOME", workspace,
        "--", agent,
    ]);
    cmd
}
```

---

## Instance Management

```rust
struct AssistantInstance {
    name: String,           // e.g., "main", "coding", "research"
    agent: String,          // e.g., "openclaw"
    workspace: PathBuf,     // ~/.agent-sandbox/assistants/openclaw-main/
    pid: Option<u32>,       // Process ID
    started_at: DateTime,
    isolation_level: IsolationLevel,
}

impl AssistantInstance {
    fn launch(name: &str, agent: &str) -> Result<Self> {
        let workspace = create_workspace(name, agent)?;
        setup_isolation(&workspace)?;
        
        let child = Command::new(agent)
            .current_dir(&workspace)
            .env("HOME", &workspace)
            .env("OPENCLAW_STATE_DIR", workspace.join(".openclaw"))
            .spawn()?;
        
        Ok(Self {
            name: name.to_string(),
            agent: agent.to_string(),
            workspace,
            pid: Some(child.id()),
            started_at: Utc::now(),
            isolation_level: detect_best_isolation(),
        })
    }
}
```

---

## Dashboard View (abox dashboard)

```
╔═══════════════════════════════════════════════════════════╗
║  abox dashboard                              v1.1.0      ║
╠═══════════════════════════════════════════════════════════╣
║                                                           ║
║  🤖 AI Assistants                                        ║
║  ┌──────────────┬─────────┬───────────┬─────────────────┐ ║
║  │ Name         │ Agent   │ Uptime    │ Status          │ ║
║  ├──────────────┼─────────┼───────────┼─────────────────┤ ║
║  │ main         │ openclaw│ 2h 34m    │ ● Running       │ ║
║  │ coding       │ openclaw│ 45m       │ ● Running       │ ║
║  │ research     │ openclaw│ -         │ ○ Stopped       │ ║
║  └──────────────┴─────────┴───────────┴─────────────────┘ ║
║                                                           ║
║  💻 Coding Sessions                                       ║
║  ┌──────────────────────┬─────────┬──────────┬──────────┐ ║
║  │ Session              │ Agent   │ Duration │ Actions  │ ║
║  ├──────────────────────┼─────────┼──────────┼──────────┤ ║
║  │ claude-20260317-1200 │ claude  │ 45m      │ 23       │ ║
║  │ opencode-20260317..  │ opencode│ 1h 2m    │ 67       │ ║
║  └──────────────────────┴─────────┴──────────┴──────────┘ ║
║                                                           ║
║  ↑↓ Navigate │ Enter Open │ A New Assistant │ Q Quit     ║
╚═══════════════════════════════════════════════════════════╝
```

---

## Technical Feasibility Analysis

| Approach | Isolation | Complexity | Root Needed | Linux Support |
|----------|-----------|------------|-------------|---------------|
| Env vars | Low | ★☆☆ | No | All |
| Sandboxed shell | Medium | ★★☆ | No | All |
| bwrap | High | ★★☆ | setuid | Most |
| unshare | High | ★★★ | No (user ns) | 5.6+ |
| Docker/Podman | Full | ★★★ | Yes | All |
| Firecracker | Full | ★★★★★ | Yes | KVM hosts |

**Recommendation:** Start with env vars + sandboxed shell (Level 1+2), add bwrap detection for Level 3.

---

## Implementation Plan

### Phase 1: Core assistant mode (1-2 hours)
- New `assistant` subcommand in CLI
- Create workspace with isolated home structure
- Launch agent with env var isolation
- Basic terminal handoff

### Phase 2: Instance management (1 hour)
- Named instances
- `assistant list` / `assistant kill`
- Instance state persistence
- Reattach support

### Phase 3: bwrap isolation (1-2 hours)
- Detect bwrap availability
- Fallback chain: bwrap → unshare → env-only
- Sandbox profile for each agent type

### Phase 4: Dashboard integration (1-2 hours)
- Unified dashboard view
- Separate assistant vs coding sections
- Instance status monitoring

---

## Security Model

```
Level 1 (Env):  App respects HOME → can't accidentally write elsewhere
                App ignores HOME → writes to real home (acceptable for trusted agents)

Level 2 (Shell): Shell can't cd out → prevents accidental access
                 Escape via exec → still possible (acceptable for trusted agents)

Level 3 (bwrap): True filesystem isolation → even malicious code can't escape
                 Network still shared → acceptable for AI assistants

Full isolation (future): + network namespace + PID namespace + cgroups
```

For OpenClaw (trusted codebase), Level 2 is sufficient.
Level 3 is nice-to-have for running untrusted agents.

---

## Open Questions

1. Do you want the assistant to have a full terminal, or just API access?
2. Should instances share network (internet access) or be restricted?
3. Do you want persistent named instances (restart where you left off)?
4. Should the dashboard show real-time logs from running instances?
