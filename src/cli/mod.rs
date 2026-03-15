pub mod run;
pub mod diff;

pub use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "abox")]
#[command(about = "🛡️  Isolated, Auditable, Portable AI Agent Runtime")]
#[command(long_about = "abox wraps AI coding agents in a sandbox that records everything.\n\nRecord, replay, audit, and share agent sessions.")]
#[command(after_help = "
EXAMPLES
  abox init my-project --template node     Create a new sandbox
  abox run --agent claude \"build an API\"   Run an agent task
  abox diff my-project                     Show file changes
  abox replay my-project                   Interactive session replay
  abox export my-project -o session.tar.gz Share a session
  abox inspect my-project                  Detailed session info

DOCS
  https://github.com/4shil/agent-sandbox
")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new sandbox workspace
    #[command(after_help = "EXAMPLES\n  abox init my-project\n  abox init my-api --template rust\n  abox init webapp --template node")]
    Init {
        /// Name for the sandbox
        name: String,

        /// Project template to use
        #[arg(short, long, default_value = "empty", value_enum)]
        template: Template,
    },

    /// List all sandboxes
    #[command(after_help = "Shows all active sandboxes with their agent, creation date, and status.")]
    Status,

    /// Run an agent task in a sandbox
    #[command(after_help = "EXAMPLES\n  abox run --agent claude \"fix the bug\"\n  abox run --agent codex \"add tests\" --memory 1gb\n  abox run --agent claude \"deploy\" --no-network")]
    Run {
        /// Agent to use (claude, codex, etc.)
        #[arg(short, long, default_value = "claude")]
        agent: String,

        /// Sandbox name (defaults to most recent)
        #[arg(short, long)]
        sandbox: Option<String>,

        /// Task description for the agent
        task: String,

        /// Max memory (e.g., "2gb", "512mb")
        #[arg(long)]
        memory: Option<String>,

        /// Max CPU time (e.g., "10m", "1h")
        #[arg(long)]
        cpu: Option<String>,

        /// Max wall-clock timeout (e.g., "30m", "1h")
        #[arg(long)]
        timeout: Option<String>,

        /// Block all network access
        #[arg(long)]
        no_network: bool,

        /// Allow network to these domains (repeatable)
        #[arg(long)]
        allow_domain: Vec<String>,
    },

    /// Show file changes from a session
    #[command(after_help = "Displays a clean diff of all file modifications made during a session.")]
    Diff {
        /// Session ID or sandbox name
        session: String,
    },

    /// Replay a session step-by-step
    #[command(after_help = "Interactive TUI for stepping through every action in a session.\n\nKeys:\n  n / →    Next action\n  p / ←    Previous action\n  j        Jump to action number\n  d        Show full action details\n  q        Quit")]
    Replay {
        /// Session ID or sandbox name
        session: String,
    },

    /// Export a session for sharing
    #[command(after_help = "EXAMPLES\n  abox export my-project -o session.tar.gz\n  abox export abc123 -o share.tar.gz")]
    Export {
        /// Session ID or sandbox name
        session: String,

        /// Output file path
        #[arg(short, long, default_value = "session.tar.gz")]
        output: String,
    },

    /// Import a shared session
    #[command(after_help = "EXAMPLES\n  abox import session.tar.gz")]
    Import {
        /// Path to the exported session file
        file: String,
    },

    /// Inspect session details and statistics
    #[command(after_help = "Shows detailed info: agent, task, duration, action breakdown, host info.")]
    Inspect {
        /// Session ID or sandbox name
        session: String,
    },
}

#[derive(Clone, ValueEnum, Debug)]
pub enum Template {
    /// Empty workspace
    Empty,
    /// Node.js (package.json + index.js)
    Node,
    /// Python (pyproject.toml + main.py)
    Python,
    /// Rust (Cargo.toml + src/main.rs)
    Rust,
}
