use crate::error::Result;
use clap::Args;
use std::process::Command;

/// Set up Magpie with company knowledge from website or PDFs.
///
/// This is the starting point for company-focused Magpie usage.
/// Pass a website URL or PDF files, and Magpie will crawl/analyze them,
/// save to knowledge/, ingest into RAG, and make all subsequent commands company-focused.
#[derive(Args, Debug)]
pub struct CompanySetupArgs {
    /// Company website URL(s) to crawl and analyze
    #[arg(long)]
    pub website: Option<Vec<String>>,
    /// Company PDF file(s) to ingest
    #[arg(long)]
    pub pdf: Option<Vec<String>>,
    /// Vision model for brand analysis (default: nemotron-3.5-super-vl-free)
    #[arg(long)]
    pub model: Option<String>,
    /// Interactive mode (prompt for website/PDFs)
    #[arg(short = 'i', long = "interactive")]
    pub interactive: bool,
}

pub fn handle(args: CompanySetupArgs) -> Result<()> {
    let mut cmd_args: Vec<String> = Vec::new();

    if let Some(urls) = &args.website {
        cmd_args.push("--website".to_string());
        cmd_args.extend(urls.iter().cloned());
    }

    if let Some(pdfs) = &args.pdf {
        cmd_args.push("--pdf".to_string());
        cmd_args.extend(pdfs.iter().cloned());
    }

    if let Some(model) = &args.model {
        cmd_args.push("--model".to_string());
        cmd_args.push(model.clone());
    }

    if args.interactive {
        cmd_args.push("--interactive".to_string());
    }

    if cmd_args.is_empty() {
        eprintln!("❌ Error: Provide at least --website or --pdf");
        eprintln!("   Example: magpie company-setup --website https://example.com");
        eprintln!("   Example: magpie company-setup --pdf brochure.pdf");
        std::process::exit(1);
    }

    let script_dir = std::env::current_dir()?.join("skills/rag");
    let script = script_dir.join("magpie-company-setup.py");

    if !script.exists() {
        return Err(crate::error::MagpieError::Custom(
            "RAG company setup script not found. Make sure skills/rag/magpie-company-setup.py exists."
                .to_string(),
        ));
    }

    let status = Command::new("python3")
        .arg(&script)
        .args(&cmd_args)
        .status()?;

    if !status.success() {
        return Err(crate::error::MagpieError::Custom(
            "Company setup failed".to_string(),
        ));
    }

    Ok(())
}
