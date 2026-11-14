use crate::error::Result;
use clap::Args;

/// List available models (like `opencode models`).
#[derive(Args, Debug)]
pub struct ModelsArgs {
    /// Filter by provider
    provider: Option<String>,
}

pub fn handle(args: ModelsArgs) -> Result<()> {
    let models = crate::model::list_models(args.provider.as_deref())?;
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
        width_id = max_id,
        width_prov = max_prov
    );
    println!(
        "  {:<width_id$}  {:<width_prov$}",
        "-", "-",
        width_id = max_id,
        width_prov = max_prov
    );
    for m in &models {
        println!(
            "  {:<width_id$}  {:<width_prov$}",
            m.id, m.provider,
            width_id = max_id,
            width_prov = max_prov
        );
    }
    Ok(())
}
