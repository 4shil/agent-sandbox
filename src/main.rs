mod cli;
mod db;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { name, template } => {
            let tpl = format!("{:?}", template).to_lowercase();
            db::init_workspace(&name, &tpl)?;
            println!("✅ Sandbox '{}' created with {:?} template", name, template);
        }
        Commands::Status => {
            let sandboxes = db::list_sandboxes()?;
            if sandboxes.is_empty() {
                println!("No active sandboxes");
            } else {
                println!("{:<20} {:<10} {:<20}", "NAME", "AGENT", "CREATED");
                println!("{}", "-".repeat(50));
                for sb in sandboxes {
                    println!("{:<20} {:<10} {:<20}", sb.name, sb.agent, sb.created_at);
                }
            }
        }
        Commands::Run { agent, task } => {
            println!("🏃 Running '{}' with agent: {}", task, agent);
            // Phase 1, commit 5
        }
        Commands::Diff { session } => {
            println!("📊 Diff for session: {}", session);
            // Phase 2
        }
        Commands::Replay { session } => {
            println!("🔁 Replay session: {}", session);
            // Phase 2
        }
        Commands::Export { session, output } => {
            println!("📦 Export session: {} -> {}", session, output);
            // Phase 3
        }
        Commands::Import { file } => {
            println!("📥 Import session: {}", file);
            // Phase 3
        }
    }

    Ok(())
}
