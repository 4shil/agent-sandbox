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
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    agent: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    Run { agent: String },
    #[command(alias = "ls")]
    List,
    Inspect { id: String },
    Replay { id: String },
    Export { id: String, #[arg(short, long, default_value = "session.tar.gz")] output: String },
    Import { file: String },
    Clean { #[arg(short, long, default_value = "30")] days: u64 },
    Init,
    Dashboard,
    Status,
    Completions { #[arg(value_enum)] shell: Shell },
    /// Tag a session for easy finding
    Tag { id: String, tag: String },
    /// Search sessions by keyword
    Search { query: String },
    /// Add a note to a session
    Note { id: String, note: String },
    /// Show session timeline
    Timeline,
    /// Show analytics/stats
    Stats,
    /// Watch sessions in real-time
    Watch,
}

#[derive(Clone, ValueEnum)]
enum Shell { Bash, Zsh, Fish }

const KNOWN_AGENTS: &[(&str, &str, &str)] = &[
    ("claude", "Claude Code", "npm i -g @anthropic-ai/claude-code"),
    ("codex", "OpenAI Codex CLI", "npm i -g @openai/codex"),
    ("opencode", "OpenCode", "npm i -g opencode-ai"),
    ("gemini", "Google Gemini CLI", "npm i -g @google/gemini-cli"),
    ("aider", "Aider", "pip install aider"),
    ("goose", "Block Goose", "pip install goose-ai"),
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
            Commands::Tag { id, tag } => session::tag_session(id, tag)?,
            Commands::Search { query } => session::search_sessions(query)?,
            Commands::Note { id, note } => session::add_note(id, note)?,
            Commands::Timeline => session::show_timeline()?,
            Commands::Stats => session::show_stats()?,
            Commands::Watch => session::watch_sessions()?,
        },
        None => {
            let agent = cli.agent.unwrap_or_default();
            if agent.is_empty() {
                ui::show_quick_help();
            } else {
                if which::which(&agent).is_err() {
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
    let workspace = PathBuf::from(&home).join(".agent-sandbox").join("workspaces").join(&sandbox_name);
    
    std::fs::create_dir_all(&workspace)?;
    std::fs::create_dir_all(workspace.join("logs"))?;

    let sfs = SandboxFs::new(&workspace)?;
    let _ = sfs.mount();

    let mut recorder = Recorder::new(&sandbox_name, agent, "interactive", &workspace.join("logs"))?;

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

    // If session > 5min, offer to notify
    if duration > 300_000 {
        println!("\n  [~] Long session ({}). Use 'abox tag {} work' to mark it.", 
            ui::format_size(duration), &sandbox_name[..sandbox_name.len().min(30)]);
    }

    ui::show_session_complete(&sandbox_name, duration, recorder.session_id());

    if let Ok(Some(code)) = status.map(|s| s.code()) {
        std::process::exit(code);
    }
    Ok(())
}

fn print_completions(shell: &Shell) -> Result<()> {
    let agents = KNOWN_AGENTS.iter().map(|(n, _, _)| *n).collect::<Vec<_>>().join(" ");
    match shell {
        Shell::Bash => println!(
            r#"_abox() {{
    local cur="${{COMP_WORDS[COMP_CWORD]}}"
    local cmds="list inspect replay export import clean init dashboard status tag search note timeline stats watch completions"
    local agents="{}"
    if [[ $COMP_CWORD -eq 1 ]]; then
        COMPREPLY=($(compgen -W "$cmds $agents" -- "$cur"))
    fi
}}
complete -F _abox abox"#, agents),
        Shell::Zsh => println!(
            r#"#compdef abox
_abox() {{ _arguments '1:command:((list inspect replay export import clean init dashboard status tag search note timeline stats watch))' '2:agent:({})' }}
compdef _abox abox"#, agents),
        Shell::Fish => println!(
            r#"complete -c abox -a "list inspect replay export import clean init dashboard status tag search note timeline stats watch" -d "Command"
complete -c abox -a "{}" -d "Agent""#, agents),
    }
    Ok(())
}
