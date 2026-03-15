mod cli;
mod db;
mod sandbox;
mod recorder;

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
        Commands::Run { agent, sandbox, task } => {
            let sandbox_name = sandbox.unwrap_or_else(|| {
                db::list_sandboxes()
                    .ok()
                    .and_then(|s| s.into_iter().next().map(|sb| sb.name))
                    .unwrap_or_else(|| "default".to_string())
            });
            cli::run::run_agent(&agent, &task, &sandbox_name)?;
        }
        Commands::Diff { session } => {
            cli::diff::show_diff(&session)?;
        }
        Commands::Replay { session } => {
            println!("🔁 Replay session: {}", session);
            println!("   (Phase 2 feature)");
        }
        Commands::Export { session, output } => {
            println!("📦 Export session: {} -> {}", session, output);
            println!("   (Phase 3 feature)");
        }
        Commands::Import { file } => {
            println!("📥 Import session: {}", file);
            println!("   (Phase 3 feature)");
        }
    }

    Ok(())
}
