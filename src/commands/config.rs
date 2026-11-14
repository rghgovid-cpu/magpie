use crate::config::{load_opencode_skill_sources, remove_opencode_skill_source, register_opencode_skill_source};
use crate::error::Result;
use clap::Args;
use clap::Subcommand;
use inquire::Confirm;

/// Manage Magpie / OpenCode skill sources.
#[derive(Subcommand, Debug)]
pub enum ConfigCommand {
    /// List configured OpenCode skill sources
    Sources(SourcesArgs),
    /// Register a skill catalog (URL or path) in OpenCode
    SourcesAdd(SourcesAddArgs),
    /// Remove a skill catalog from OpenCode config
    SourcesRemove(SourcesRemoveArgs),
}

#[derive(Args, Debug)]
pub struct SourcesArgs;

#[derive(Args, Debug)]
pub struct SourcesAddArgs {
    /// Skill catalog URL or path
    source: String,
}

#[derive(Args, Debug)]
pub struct SourcesRemoveArgs {
    /// Skill catalog URL or path
    source: String,
}

pub fn handle(cmd: ConfigCommand) -> Result<()> {
    match cmd {
        ConfigCommand::Sources(_) => {
            let sources = load_opencode_skill_sources()?;
            if sources.is_empty() {
                println!("No skill sources configured yet.");
                println!("Add one: magpie config sources-add https://example.com/skills/");
            } else {
                println!("\nConfigured skill sources:\n");
                for (i, s) in sources.iter().enumerate() {
                    println!("  {}. {}", i + 1, s);
                }
                println!();
            }
        }
        ConfigCommand::SourcesAdd(args) => {
            register_opencode_skill_source(&args.source)?;
            println!("✅ Registered skill source: {}", args.source);
        }
        ConfigCommand::SourcesRemove(args) => {
            let confirmed = Confirm::new(&format!("Remove skill source \"{}\"?", args.source))
                .with_default(false)
                .prompt()?;
            if confirmed {
                remove_opencode_skill_source(&args.source)?;
                println!("✅ Removed skill source: {}", args.source);
            } else {
                println!("Cancelled.");
            }
        }
    }
    Ok(())
}
