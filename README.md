# 🛡️ abox

> Transparent sandbox for AI coding agents

Your agent works the same. abox just wraps it in a sandbox and records everything.

```
$ abox claude
$ abox opencode
$ abox codex
$ abox gemini
```

## Install

```bash
curl -sSL https://raw.githubusercontent.com/4shil/agent-sandbox/main/install.sh | bash
```

## Usage

```bash
abox <agent>
```

That's it. The agent launches, you work normally, session is recorded in the background.

No new commands to learn. No flags. Just `abox` before your agent name.

## What it does

- 🛡️ Creates an isolated workspace
- 📝 Records all file changes silently
- 🔄 Saves session for replay later
- ✅ Agent runs exactly as usual — no changes

## Sessions

Sessions are auto-saved. Find them later:

```bash
ls ~/.agent-sandbox/workspaces/
```

## Supported agents

Any binary on your PATH works:

```
claude, codex, opencode, gemini, aider, goose, or anything
```

## License

MIT
