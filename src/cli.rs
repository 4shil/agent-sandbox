use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "agent-sandbox")]
#[command(about = "Isolated, Auditable, Portable AI Agent Runtime")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new sandbox workspace
    Init {
        /// Name of the sandbox
        name: String,

        /// Template to use
        #[arg(short, long, default_value = "empty")]
        template: Template,
    },

    /// List active sandboxes
    Status,

    /// Run an agent task in sandbox
    Run {
        /// Agent to use
        #[arg(short, long, default_value = "claude")]
        agent: String,

        /// Task description
        task: String,
    },

    /// Show diff of a session
    Diff {
        /// Session name or ID
        session: String,
    },

    /// Replay a session
    Replay {
        /// Session name or ID
        session: String,
    },

    /// Export a session for sharing
    Export {
        /// Session name or ID
        session: String,

        /// Output file
        #[arg(short, long, default_value = "session.tar.gz")]
        output: String,
    },

    /// Import a shared session
    Import {
        /// Input file
        file: String,
    },
}

#[derive(Clone, ValueEnum, Debug)]
pub enum Template {
    Empty,
    Node,
    Python,
    Rust,
}
