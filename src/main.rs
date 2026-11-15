use clap::{Parser, Subcommand};
use magpie::{
    agent, commands, error::Result, model, print, registry, skill,
};
use std::path::PathBuf;

/// Magpie — a marketing-focused, open-source OpenCode alternative in Rust.
///
/// Features agentic search & writing, a skill marketplace (skill.sh) listed
/// like models, skill install with confirmation + auto-configure, and
/// PDF / Word printing in A4 and US Letter.
#[derive(Parser, Debug)]
#[command(name = "magpie", version = "0.1.0", author = "Magpie")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// List and install marketing skills (like `opencode models` lists models).
    Skills {
        /// Skill source: magpie | skill.sh
        #[arg(long)]
        source: Option<String>,
        /// Filter skill.sh skills by owner/repo
        #[arg(long)]
        owner: Option<String>,
        /// Search skills by id, name, description or tag
        #[arg(short = 'q', long = "query")]
        query: Option<String>,
        /// Install a specific skill id
        #[arg(short = 'i', long = "install")]
        install: Option<String>,
        /// Skip the install confirmation prompt
        #[arg(short = 'y', long = "yes")]
        yes: bool,
    },
    /// Print a file to PDF or Word (A4 & US Letter).
    Print {
        /// File to print (markdown or plain text)
        input: PathBuf,
        /// Output format: pdf | word
        #[arg(short = 'f', long = "format", default_value = "pdf")]
        format: String,
        /// Page size: a4 | letter
        #[arg(short = 's', long = "size", default_value = "a4")]
        size: String,
        /// Output file path
        #[arg(short = 'o', long = "out")]
        out: Option<PathBuf>,
    },
    /// List available models (like `opencode models`).
    Models {
        /// Filter by provider
        #[arg(short = 'p', long = "provider")]
        provider: Option<String>,
    },
    /// Run Magpie with a marketing context (agentic search & writing).
    Run {
        /// Message to send (one-shot mode). Omit for interactive TUI.
        message: Option<String>,
        /// Stream the LLM response token by token
        #[arg(short = 's', long = "stream")]
        stream: bool,
    },
    /// Manage Magpie / OpenCode skill sources.
    Config {
        #[command(subcommand)]
        command: commands::config::ConfigCommand,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(cli).await {
        eprintln!("❌ Magpie error: {e}");
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Skills {
            source,
            owner: _,
            query,
            install,
            yes,
        } => {
            let registry = registry::SkillRegistry::new()?;
            if let Some(id) = install {
                let entries =
                    registry::list_skills(&registry, source.as_deref(), None).await?;
                let entry = entries
                    .iter()
                    .find(|e| e.id() == id)
                    .ok_or_else(|| magpie::error::MagpieError::SkillNotFound(id))?;
                skill::install_skill(entry, yes).await?;
            } else {
                let entries =
                    registry::list_skills(&registry, source.as_deref(), query.as_deref())
                        .await?;
                if entries.is_empty() {
                    println!("No skills found.");
                    return Ok(());
                }
                println!("\n🪶 Magpie skills ({})\n", entries.len());
                let max_id = entries.iter().map(|e| e.id().len()).max().unwrap_or(4);
                let max_name = entries.iter().map(|e| e.name().len()).max().unwrap_or(4);
                let max_src = entries
                    .iter()
                    .map(|e| e.source().len())
                    .max()
                    .unwrap_or(6);
                println!(
                    "  {:<width_id$}  {:<width_name$}  {:<width_src$}  DESCRIPTION",
                    "ID", "NAME", "SOURCE",
                    width_id = max_id, width_name = max_name, width_src = max_src
                );
                println!(
                    "  {:<width_id$}  {:<width_name$}  {:<width_src$}  -----------",
                    "-", "-", "-",
                    width_id = max_id, width_name = max_name, width_src = max_src
                );
                for entry in &entries {
                    let desc = entry.description();
                    let desc = if desc.len() > 60 {
                        format!("{}...", &desc[..57])
                    } else {
                        desc.to_string()
                    };
                    println!(
                        "  {:<width_id$}  {:<width_name$}  {:<width_src$}  {}",
                        entry.id(),
                        entry.name(),
                        entry.source(),
                        desc,
                        width_id = max_id,
                        width_name = max_name,
                        width_src = max_src
                    );
                }
                let source_filter = source.as_deref().unwrap_or("magpie/skill.sh");
                let query_note = query.map(|q| format!(" (query: {q})")).unwrap_or_default();
                println!(
                    "\nUse: magpie skills --install <id>\n  Source filter: {source_filter}{query_note}"
                );

                let pick = inquire::Confirm::new("Would you like to install a skill now?")
                    .with_default(false)
                    .prompt()?;
                if pick {
                    let id: String =
                        inquire::Text::new("Enter skill ID to install:").prompt()?;
                    let entries =
                        registry::list_skills(&registry, source.as_deref(), None).await?;
                    let entry = entries
                        .iter()
                        .find(|e| e.id() == id)
                        .ok_or_else(|| magpie::error::MagpieError::SkillNotFound(id))?;
                    skill::install_skill(entry, yes).await?;
                }
            }
        }
        Commands::Print { input, format, size, out } => {
            let format = print::PrintFormat::from_str(&format)?;
            let size = print::PageSize::from_str(&size)?;
            print::print_file(&input, format, size, out.as_deref())?;
        }
        Commands::Models { provider } => {
            let models = model::list_models(provider.as_deref())?;
            if models.is_empty() {
                println!("No models found.");
                return Ok(());
            }
            println!("\n🪶 Magpie models ({})\n", models.len());
            let max_id = models.iter().map(|m| m.id.len()).max().unwrap_or(4);
            let max_prov = models.iter().map(|m| m.provider.len()).max().unwrap_or(8);
            println!(
                "  {:<width_id$}  {:<width_prov$}",
                "ID", "PROVIDER",
                width_id = max_id, width_prov = max_prov
            );
            println!(
                "  {:<width_id$}  {:<width_prov$}",
                "-", "-",
                width_id = max_id, width_prov = max_prov
            );
            for m in &models {
                println!(
                    "  {:<width_id$}  {:<width_prov$}",
                    m.id, m.provider,
                    width_id = max_id, width_prov = max_prov
                );
            }
        }
        Commands::Run { message, stream } => {
            match message {
                Some(msg) => agent::run_one_shot(&msg, stream).await?,
                None => agent::run_interactive(stream).await?,
            }
        }
        Commands::Config { command } => commands::config::handle(command)?,
    }
    Ok(())
}
