use crate::error::Result;
use clap::Args;

/// Run Magpie with a marketing context (agentic search & writing).
#[derive(Args, Debug)]
pub struct RunArgs {
    /// Message to send (one-shot mode). Omit for interactive TUI.
    message: Option<String>,
    /// Stream the LLM response token by token
    #[arg(short = 's', long = "stream")]
    stream: bool,
}

pub async fn handle(args: RunArgs) -> Result<()> {
    match args.message {
        Some(msg) => crate::agent::run_one_shot(&msg, args.stream).await,
        None => crate::agent::run_interactive(args.stream).await,
    }
}
