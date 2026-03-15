mod db;
mod sandbox;
mod recorder;
mod session;
mod ui;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::SystemTime;
use colored::Colorize;

use crate::sandbox::SandboxFs;
use crate::recorder::Recorder;

#[derive(Parser)]
#[command(name = "abox")]
#[command(about = "sandbox for ai coding agents")]
#[command(long_about = "Launch any AI agent inside an isolated sandbox.\n\n  abox claude          launch Claude\n  abox opencode        launch OpenCode\n  abox list            show recorded sessions\n  abox replay <id>     replay a session")]
#[command(after_help = "SESSIONS\n  abox list              list all sessions\n  abox inspect <id>      show session details\n  abox replay <id>       step-through replay\n  abox export <id> -o f  export as tar.gz\n  abox import <file>     import shared session\n  abox clean --days 7    remove old sessions")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Agent to launch (when no subcommand)
    agent: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Launch agent in sandbox
    Run {
        /// Agent to launch
        agent: String,
    },
    /// List recorded sessions
    #[command(alias = "ls")]
    List,
    /// Show session details
    Inspect {
        /// Session ID or workspace name
        id: String,
    },
    /// Replay session in terminal
    Replay {
        /// Session ID or workspace name
        id: String,
    },
    /// Export session as tar.gz
    Export {
        /// Session ID or workspace name
        id: String,

        /// Output file
        #[arg(short, long, default_value = "session.tar.gz")]
        output: String,
    },
    /// Import a shared session
    Import {
        /// Path to session archive
        file: String,
    },
    /// Clean old sessions
    Clean {
        /// Delete sessions older than N days
        #[arg(short, long, default_value = "30")]
        days: u64,
    },
    /// First-run setup wizard
    Init,
    /// Show dashboard
    Dashboard,
    /// Show agent status
    Status,
    /// Install shell completions
    Completions {
        /// Shell type
        #[arg(value_enum)]
        shell: Shell,
    },
}

#[derive(Clone, ValueEnum)]
enum Shell {
    Bash,
    Zsh,
    Fish,
}

// Known agents and how to detect/install them
const KNOWN_AGENTS: &[(&str, &str, &str, &[&str])] = &[
    ("claude", "Claude Code", "npm i -g @anthropic-ai/claude-code", &["claude"]),
    ("codex", "OpenAI Codex CLI", "npm i -g @openai/codex", &["codex"]),
    ("opencode", "OpenCode", "npm i -g opencode-ai", &["opencode"]),
    ("gemini", "Google Gemini CLI", "npm i -g @google/gemini-cli", &["gemini"]),
    ("aider", "Aider", "pip install aider", &["aider"]),
    ("goose", "Block Goose", "pip install goose-ai", &["goose"]),
];

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Some(cmd) => match cmd {
            Commands::Run { agent } => run_agent(agent)?,
            Commands::List => session::list_sessions()?,
            Commands::Inspect { id } => session::inspect_session(id)?,
            Commands::Replay { id } => session::replay_session(id)?,
            Commands::Export { id, output } => session::export_session(id, output)?,
            Commands::Import { file } => session::import_session(file)?,
            Commands::Clean { days } => session::clean_sessions(*days)?,
            Commands::Init => ui::run_wizard()?,
            Commands::Dashboard => ui::show_dashboard()?,
            Commands::Status => ui::show_status()?,
            Commands::Completions { shell } => print_completions(shell)?,
        },
        None => {
            let agent = cli.agent.unwrap_or_default();
            if agent.is_empty() {
                // No args — show friendly help
                ui::show_quick_help();
            } else {
                // Check if agent exists, give helpful error
                if let Err(e) = which::which(&agent) {
                    ui::show_agent_not_found(&agent);
                    std::process::exit(1);
                }
                run_agent(&agent)?;
            }
        }
    }

    Ok(())
}

fn run_agent(agent: &str) -> Result<()> {
    let home = std::env::var("HOME")?;
    let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
    let sandbox_name = format!("{}-{}", agent, timestamp);
    let workspace = PathBuf::from(&home)
        .join(".agent-sandbox")
        .join("workspaces")
        .join(&sandbox_name);
    
    std::fs::create_dir_all(&workspace)?;
    std::fs::create_dir_all(workspace.join("logs"))?;

    let sfs = SandboxFs::new(&workspace)?;
    let _ = sfs.mount();

    let logs_dir = workspace.join("logs");
    let mut recorder = Recorder::new(&sandbox_name, agent, "interactive", &logs_dir)?;

    // Show first-run tip if no sessions exist
    if !session::has_sessions() {
        ui::show_first_run_tip(agent, &sandbox_name);
    }

    let mut cmd = Command::new(agent);
    cmd.current_dir(sfs.agent_root());
    cmd.envs(std::env::vars());
    cmd.stdin(Stdio::inherit());
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());

    let start = SystemTime::now();
    let status = cmd.status();
    let duration = start.elapsed()?.as_millis() as u64;

    let _ = recorder.record_action("session", serde_json::json!({
        "duration_ms": duration,
        "exit_code": status.as_ref().ok().and_then(|s| s.code()),
    }));

    for file in sfs.modified_files().unwrap_or_default() {
        let _ = recorder.record_action("file_modified", serde_json::json!({
            "path": file.strip_prefix(&workspace).unwrap_or(&file).to_string_lossy(),
        }));
    }

    let _ = recorder.finish();

    // Show session summary
    ui::show_session_complete(&sandbox_name, duration, recorder.session_id());

    if let Ok(Some(code)) = status.map(|s| s.code()) {
        std::process::exit(code);
    }

    Ok(())
}

fn print_completions(shell: &Shell) -> Result<()> {
    let bin = "abox";
    match shell {
        Shell::Bash => println!(
            r#"_abox() {{
    local cur="${{COMP_WORDS[COMP_CWORD]}}"
    local cmds="list inspect replay export import clean init dashboard status completions run"
    local agents="claude codex opencode gemini aider goose"
    if [[ $COMP_CWORD -eq 1 ]]; then
        COMPREPLY=($(compgen -W "$cmds $agents" -- "$cur"))
    fi
}}
complete -F _abox abox"#
        ),
        Shell::Zsh => println!(
            r#"#compdef abox

_abox() {{
    _arguments \
        '1:command:((list inspect replay export import clean init dashboard status completions run))' \
        '2:agent:({})'
}}

compdef _abox abox"#,
            KNOWN_AGENTS.iter().map(|(n, _, _, _)| *n).collect::<Vec<_>>().join(" ")
        ),
        Shell::Fish => println!(
            r#"complete -c abox -a "list inspect replay export import clean init dashboard status run" -d "Command"
complete -c abox -a "claude codex opencode gemini aider goose" -d "Agent""#
        ),
    }
    Ok(())
}
