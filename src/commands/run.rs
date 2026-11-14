use crate::error::Result;
use clap::Args;

/// Run Magpie with a marketing context (agentic search & writing).
#[derive(Args, Debug)]
pub struct RunArgs {
    /// Message to send (one-shot mode). Omit for interactive TUI.
    message: Option<String>,
}

pub async fn handle(args: RunArgs) -> Result<()> {
    match args.message {
        Some(msg) => crate::agent::run_one_shot(&msg).await,
        None => crate::agent::run_interactive().await,
    }
}
