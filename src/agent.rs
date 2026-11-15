use crate::config::load_magpie_config;
use crate::error::Result;
use crate::llm::LLMClient;
use std::io::{self, Write};

/// Marketing system prompt that frames Magpie as a marketing assistant.
const MARKETING_CONTEXT: &str = "You are Magpie, a marketing-focused AI assistant.\n\
Help the user with marketing tasks: copywriting, campaign planning, SEO, ads, email,\nsocial media, landing pages, brand voice, and go-to-market messaging.\n\
Prefer clear, persuasive, conversion-oriented output. When relevant, use installed\nmarketing skills to ground your work.";

/// Run Magpie in interactive agentic mode with real LLM responses.
pub async fn run_interactive(stream: bool) -> Result<()> {
    let llm = LLMClient::from_config()?;

    println!("🪶 Magpie — marketing assistant (agentic mode)");
    println!("Model: {} | Base URL: {}", llm.model, llm.base_url);
    println!("Type your request. Press Ctrl+C or type /exit to quit.\n");

    loop {
        print!("> ");
        io::stdout().flush()?;
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => break, // EOF
            Ok(_) => {
                let input = input.trim();
                if input.is_empty() {
                    continue;
                }
                if input == "/exit" || input == "/quit" {
                    println!("Goodbye!");
                    break;
                }
                if input == "/help" {
                    print_help();
                    continue;
                }

                println!("\n[Magpie agent] Thinking...\n");

                if stream {
                    llm.chat_stream(MARKETING_CONTEXT, input).await?;
                } else {
                    let response = llm.chat(MARKETING_CONTEXT, input).await?;
                    println!("{}", response);
                }
                println!();
            }
            Err(e) => return Err(crate::error::MagpieError::Io(e)),
        }
    }
    Ok(())
}

/// One-shot run: send a message and get a response from the LLM.
pub async fn run_one_shot(message: &str, stream: bool) -> Result<()> {
    let llm = LLMClient::from_config()?;

    if stream {
        llm.chat_stream(MARKETING_CONTEXT, message).await?;
    } else {
        let response = llm.chat(MARKETING_CONTEXT, message).await?;
        println!("{}", response);
    }
    Ok(())
}

fn print_help() {
    println!("Magpie agent commands:");
    println!("  /help   - show this help");
    println!("  /exit   - quit Magpie");
    println!("");
    println!("Example marketing requests:");
    println!("  \"write a product launch email\"");
    println!("  \"create a landing page for our new campaign\"");
    println!("  \"write ad copy for a Facebook campaign\"");
    println!("  \"optimise this blog post for SEO\"");
    println!("");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_marketing_context_non_empty() {
        assert!(!super::MARKETING_CONTEXT.is_empty());
    }
}
