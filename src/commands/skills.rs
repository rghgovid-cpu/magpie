use crate::error::Result;
use clap::Args;

/// List and install marketing skills (like `opencode models` lists models).
#[derive(Args, Debug)]
pub struct SkillsArgs {
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
    install_id: Option<String>,
    /// Skip the install confirmation prompt
    #[arg(short = 'y', long = "yes")]
    yes: bool,
}

pub async fn handle(args: SkillsArgs) -> Result<()> {
    let registry = crate::registry::SkillRegistry::new()?;

    // If --install is given, install directly.
    if let Some(id) = args.install_id {
        let entries = crate::registry::list_skills(&registry, args.source.as_deref(), None).await?;
        let entry = entries
            .iter()
            .find(|e| e.id() == id)
            .ok_or_else(|| crate::error::MagpieError::SkillNotFound(id.clone()))?;
        crate::skill::install_skill(entry, args.yes).await?;
        return Ok(());
    }

    // Otherwise list skills.
    let entries =
        crate::registry::list_skills(&registry, args.source.as_deref(), args.query.as_deref())
            .await?;

    if entries.is_empty() {
        println!("No skills found.");
        return Ok(());
    }

    println!("\n🪶 Magpie skills ({})\n", entries.len());
    // Print a table-like list.
    let max_id = entries.iter().map(|e| e.id().len()).max().unwrap_or(4);
    let max_name = entries.iter().map(|e| e.name().len()).max().unwrap_or(4);
    let max_src = entries.iter().map(|e| e.source().len()).max().unwrap_or(6);

    println!(
        "  {:<width_id$}  {:<width_name$}  {:<width_src$}  DESCRIPTION",
        "ID", "NAME", "SOURCE",
        width_id = max_id,
        width_name = max_name,
        width_src = max_src
    );
    println!(
        "  {:<width_id$}  {:<width_name$}  {:<width_src$}  -----------",
        "-", "-", "-",
        width_id = max_id,
        width_name = max_name,
        width_src = max_src
    );
    for entry in &entries {
        let desc = entry.description();
        let desc = if desc.len() > 60 { format!("{}...", &desc[..57]) } else { desc.to_string() };
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

    let source_filter = args.source.as_deref().unwrap_or("magpie/skill.sh");
    let query_note = args.query.map(|q| format!(" (query: {q})")).unwrap_or_default();
    println!(
        "\nUse: magpie skills --install <id>   (or run `magpie skills --interactive` to pick)\n  Source filter: {source_filter}{query_note}"
    );

    // Interactive picker if no install id given but user might want to pick.
    // We offer an interactive install prompt.
    if !args.yes {
        let pick = inquire::Confirm::new("Would you like to install a skill now?")
            .with_default(false)
            .prompt()?;
        if pick {
            let id: String = inquire::Text::new("Enter skill ID to install:").prompt()?;
            let entries = crate::registry::list_skills(&registry, args.source.as_deref(), None).await?;
            let entry = entries
                .iter()
                .find(|e| e.id() == id)
                .ok_or_else(|| crate::error::MagpieError::SkillNotFound(id))?;
            crate::skill::install_skill(entry, false).await?;
        }
    }

    Ok(())
}
