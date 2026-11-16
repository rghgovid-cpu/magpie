use crate::config::load_magpie_config;
use crate::error::Result;
use crate::llm::LLMClient;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

/// Marketing system prompt that frames Magpie as a marketing assistant.
pub const MARKETING_CONTEXT: &str = "You are Magpie, a marketing-focused AI assistant.\n\
Help the user with marketing tasks: copywriting, campaign planning, SEO, ads, email,\nsocial media, landing pages, brand voice, and go-to-market messaging.\n\
Prefer clear, persuasive, conversion-oriented output. When relevant, use installed\nmarketing skills to ground your work.";

/// Default knowledge directory relative to CWD.
const DEFAULT_KNOWLEDGE_DIR: &str = "knowledge";

/// Default RAG script path relative to CWD.
const DEFAULT_RAG_SCRIPT: &str = "skills/rag/magpie-rag.py";

/// Retrieve RAG context for a query if knowledge directory and RAG script exist.
/// Returns the context string to prepend to the system prompt, or empty string if
/// RAG is not configured.
fn get_rag_context(query: &str) -> String {
    let knowledge_dir = Path::new(DEFAULT_KNOWLEDGE_DIR);
    let rag_script = Path::new(DEFAULT_RAG_SCRIPT);

    // Only use RAG if both knowledge dir and script exist
    if !knowledge_dir.is_dir() || !rag_script.is_file() {
        return String::new();
    }

    // Check if index exists
    let index_file = knowledge_dir.join(".rag-index.json");
    if !index_file.exists() {
        // Try to ingest first
        let _ = Command::new("python3")
            .arg(rag_script)
            .arg("ingest")
            .output();
    }

    // Run search
    let output = Command::new("python3")
        .arg(rag_script)
        .arg("search")
        .arg(query)
        .arg("--top-k")
        .arg("5")
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if stdout.trim().is_empty() {
                return String::new();
            }
            format!(
                "\n\n## Company Knowledge (retrieved from knowledge/)\n{}\n\nUse the above company knowledge to make your response accurate, on-brand, and specific to this company.",
                stdout.trim()
            )
        }
        _ => String::new(),
    }
}

/// Build the full system prompt with optional RAG context.
fn build_system_prompt(query: &str) -> String {
    let rag_context = get_rag_context(query);
    if rag_context.is_empty() {
        MARKETING_CONTEXT.to_string()
    } else {
        format!("{}{}", MARKETING_CONTEXT, rag_context)
    }
}

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

                let system_prompt = build_system_prompt(input);

                if stream {
                    llm.chat_stream(&system_prompt, input).await?;
                } else {
                    let response = llm.chat(&system_prompt, input).await?;
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

    let system_prompt = build_system_prompt(message);

    if stream {
        llm.chat_stream(&system_prompt, message).await?;
    } else {
        let response = llm.chat(&system_prompt, message).await?;
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
    println!("RAG (company knowledge):");
    println!("  Drop documents into knowledge/ and Magpie will use them automatically.");
    println!("  Run 'magpie run \"rag ingest\"' to index your documents.");
    println!("");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_marketing_context_non_empty() {
        assert!(!super::MARKETING_CONTEXT.is_empty());
    }

    #[test]
    fn test_build_system_prompt_no_rag() {
        // Without knowledge/ directory, prompt should be unchanged
        let prompt = super::build_system_prompt("test query");
        assert_eq!(prompt, super::MARKETING_CONTEXT);
    }
}
