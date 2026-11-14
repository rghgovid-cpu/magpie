use crate::error::Result;
use std::io::{self, Write};

/// Marketing system prompt that frames Magpie as a marketing assistant.
const MARKETING_CONTEXT: &str = "You are Magpie, a marketing-focused AI assistant.\n\
Help the user with marketing tasks: copywriting, campaign planning, SEO, ads, email,\nsocial media, landing pages, brand voice, and go-to-market messaging.\n\
Prefer clear, persuasive, conversion-oriented output. When relevant, use installed\nmarketing skills to ground your work.";

/// Run Magpie in interactive agentic mode.
pub async fn run_interactive() -> Result<()> {
    println!("🪶 Magpie — marketing assistant (agentic mode)");
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

                // Build the full prompt with marketing context.
                let full_prompt = format!("{}\n\nUser request: {}", MARKETING_CONTEXT, input);

                // In a full build, this would call an LLM provider via the model API.
                // Here we provide a helpful scaffolding response.
                println!("\n[Magpie agent] Received: \"{}\"\n", input);
                println!("Full prompt sent to model:");
                println!("{}\n", full_prompt);
                println!("Suggested next steps for this marketing task:");
                println!("  1. Define the target audience and goal.");
                println!("  2. Choose a skill: magpie skills install <skill>");
                println!("  3. Draft copy using the selected skill.");
                println!("  4. Print to PDF/Word: magpie print draft.md --format pdf --size a4\n");
            }
            Err(e) => return Err(crate::error::MagpieError::Io(e)),
        }
    }
    Ok(())
}

/// One-shot run: send a message and get a response.
pub async fn run_one_shot(message: &str) -> Result<()> {
    let full_prompt = format!("{}\n\nUser request: {}", MARKETING_CONTEXT, message);
    println!("{}", full_prompt);
    println!("\n[Magpie agent] (LLM provider integration pending — wire in your preferred model here.)");
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
