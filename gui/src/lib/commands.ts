// CLI Command mappings for GUI
// Generated from agent-sandbox CLI definition

export interface CliCommand {
  name: string
  description: string
  args?: CommandArg[]
  aliases?: string[]
}

export interface CommandArg {
  name: string
  description: string
  required: boolean
  default?: string
}

export const CLI_COMMANDS: CliCommand[] = [
  {
    name: 'run',
    description: 'Launch an agent in a sandbox',
    args: [
      { name: 'agent', description: 'Agent name (claude, opencode, codex, gemini)', required: true },
    ],
  },
  {
    name: 'list',
    aliases: ['ls'],
    description: 'List recorded sessions',
  },
  {
    name: 'inspect',
    description: 'Inspect a session',
    args: [
      { name: 'id', description: 'Session ID or prefix', required: true },
    ],
  },
  {
    name: 'replay',
    description: 'Replay a session',
    args: [
      { name: 'id', description: 'Session ID or prefix', required: true },
    ],
  },
  {
    name: 'export',
    description: 'Export a session as tar.gz',
    args: [
      { name: 'id', description: 'Session ID', required: true },
      { name: 'output', description: 'Output file', required: false, default: 'session.tar.gz' },
    ],
  },
  {
    name: 'import',
    description: 'Import a session from tar.gz',
    args: [
      { name: 'file', description: 'Tar.gz file', required: true },
    ],
  },
  {
    name: 'clean',
    description: 'Clean sessions older than N days',
    args: [
      { name: 'days', description: 'Number of days', required: false, default: '30' },
    ],
  },
  {
    name: 'init',
    description: 'Initialize config',
  },
  {
    name: 'dashboard',
    description: 'Open interactive TUI dashboard',
  },
  {
    name: 'status',
    description: 'Show status',
  },
  {
    name: 'completions',
    description: 'Generate shell completions',
    args: [
      { name: 'shell', description: 'Shell (bash, zsh, fish)', required: true },
    ],
  },
  {
    name: 'tag',
    description: 'Tag a session',
    args: [
      { name: 'id', required: true },
      { name: 'tag', required: true },
    ],
  },
  {
    name: 'search',
    description: 'Search sessions by keyword',
    args: [
      { name: 'query', required: true },
    ],
  },
  {
    name: 'note',
    description: 'Add a note to a session',
    args: [
      { name: 'id', required: true },
      { name: 'note', required: true },
    ],
  },
  {
    name: 'timeline',
    description: 'Show session timeline',
  },
  {
    name: 'stats',
    description: 'Show statistics',
  },
  {
    name: 'watch',
    description: 'Watch sessions in real-time',
  },
]

// Helper to map CLI command to GUI action
export function getCommand(name: string): CliCommand | undefined {
  return CLI_COMMANDS.find(c => c.name === name || c.aliases?.includes(name))
}
