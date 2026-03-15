# 🛡️ abox

> Sandbox wrapper for AI coding agents

Just prefix your agent with `abox` — it runs in an isolated sandbox, records everything, and exits. Nothing else to learn.

## Install

```bash
curl -sSL https://raw.githubusercontent.com/4shil/agent-sandbox/main/install.sh | bash
```

## Usage

```bash
# Before
claude "Build a REST API"

# After (sandboxed, recorded)
abox claude "Build a REST API"
```

That's it.

```
$ abox opencode "Fix the bug"
🛡️ sandbox: 20260315-143825
[...agent output...]

$ abox codex "Add tests"
🛡️ sandbox: 20260315-144012
[...agent output...]

$ abox claude --memory 2gb "Build something"
🛡️ sandbox: 20260315-144130
MEM: 2.0GB
[...agent output...]
```

## Options

```bash
abox <agent> [task] [flags]

  --memory <2gb>       Memory limit
  --timeout <30m>      Timeout
  --no-network         Block internet
  --allow-domain       Whitelist domains
  --name <sandbox>     Custom sandbox name
  --stats              Show session info after
```

## Supported Agents

Any CLI binary works:

| Agent | Install |
|-------|---------|
| `claude` | `npm i -g @anthropic-ai/claude-code` |
| `opencode` | `npm i -g opencode-ai` |
| `codex` | OpenAI CLI |
| `gemini` | `npm i -g @google/gemini-cli` |
| `aider` | `pip install aider` |
| anything | just pass the binary name |

## What Happens

1. Creates isolated workspace (`~/.agent-sandbox/workspaces/<timestamp>/`)
2. Launches your agent with full env vars (API keys work)
3. Records every file change, command, action in background
4. Agent runs normally — stdin/stdout/stderr all passthrough
5. Session saved for replay/audit later

## Session Replay (background)

Sessions are auto-recorded. View them later:

```bash
# List sessions
ls ~/.agent-sandbox/workspaces/*/logs/

# Replay any session
abox replay <session-id>

# Export for sharing
abox export <session-id> -o share.tar.gz
```

## Why?

| Without abox | With abox |
|--------------|-----------|
| Agent runs as you | Isolated sandbox |
| No audit trail | Every action recorded |
| Messy commits | Clean diffs |
| Can't share sessions | Export as tar.gz |
| No resource limits | Memory/timeout limits |
| Permanent changes | Review before merge |

## License

MIT
