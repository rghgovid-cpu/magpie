use crate::config::load_magpie_config;
use crate::error::Result;
use crate::llm::LLMClient;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;
use regex::Regex;

/// Marketing system prompt that frames Magpie as a marketing assistant.
pub const MARKETING_CONTEXT: &str = "You are Magpie, a marketing-focused AI assistant.\n\
Help the user with marketing tasks: copywriting, campaign planning, SEO, ads, email,\nsocial media, landing pages, brand voice, and go-to-market messaging.\n\
Prefer clear, persuasive, conversion-oriented output. When relevant, use installed\nmarketing skills to ground your work.";

/// Default knowledge directory relative to CWD.
const DEFAULT_KNOWLEDGE_DIR: &str = "knowledge";

/// Default RAG script path relative to CWD.
const DEFAULT_RAG_SCRIPT: &str = "skills/rag/magpie-rag.py";

/// Default brand analysis script path relative to CWD.
const DEFAULT_BRAND_SCRIPT: &str = "skills/brand-analysis/magpie-brand-analysis.py";

/// Regex to detect URLs in user input.
lazy_static::lazy_static! {
    static ref URL_REGEX: Regex = Regex::new(
        r#"https?://[^\s<>"{}|\\^`\[\]]+"#
    ).unwrap();
}

/// Extract URLs from user input.
fn extract_urls(text: &str) -> Vec<String> {
    URL_REGEX.find_iter(text).map(|m| m.as_str().to_string()).collect()
}

/// Retrieve RAG context for a query if knowledge directory and RAG script exist.
fn get_rag_context(query: &str) -> String {
    let knowledge_dir = Path::new(DEFAULT_KNOWLEDGE_DIR);
    let rag_script = Path::new(DEFAULT_RAG_SCRIPT);

    if !knowledge_dir.is_dir() || !rag_script.is_file() {
        return String::new();
    }

    let index_file = knowledge_dir.join(".rag-index.json");
    if !index_file.exists() {
        let _ = Command::new("python3")
            .arg(rag_script)
            .arg("ingest")
            .output();
    }

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
                "\n\n## Company Knowledge (retrieved from knowledge/)\n{}\n",
                stdout.trim()
            )
        }
        _ => String::new(),
    }
}

/// Load static company context files (company-summary.md, brand-analysis.md)
/// that are always included in every marketing request.
fn get_static_company_context() -> String {
    let knowledge_dir = Path::new(DEFAULT_KNOWLEDGE_DIR);
    if !knowledge_dir.is_dir() {
        return String::new();
    }

    let mut context = String::new();

    // Company summary
    let summary_file = knowledge_dir.join("company-summary.md");
    if summary_file.exists() {
        if let Ok(content) = std::fs::read_to_string(&summary_file) {
            if !content.trim().is_empty() {
                context.push_str(&format!(
                    "\n\n## Company Summary\n{}\n",
                    content.trim()
                ));
            }
        }
    }

    // Brand analysis
    let brand_file = knowledge_dir.join("brand-analysis.md");
    if brand_file.exists() {
        if let Ok(content) = std::fs::read_to_string(&brand_file) {
            if !content.trim().is_empty() {
                context.push_str(&format!(
                    "\n\n## Brand Identity (from website analysis)\n{}\n",
                    content.trim()
                ));
            }
        }
    }

    if context.is_empty() {
        String::new()
    } else {
        format!("\n\n## Company Context (always applied){}", context)
    }
}

/// Automatically analyze websites mentioned in user input and return brand context.
/// This runs brand analysis on any URLs found in the input, then injects the
/// brand identity (colors, fonts, logo) into the system prompt.
fn get_brand_context(query: &str) -> String {
    let urls = extract_urls(query);
    if urls.is_empty() {
        return String::new();
    }

    let brand_script = Path::new(DEFAULT_BRAND_SCRIPT);
    if !brand_script.is_file() {
        return String::new();
    }

    let mut all_brand_context = String::new();

    for url in urls {
        // Ensure URL has scheme
        let url_with_scheme = if !url.starts_with("http://") && !url.starts_with("https://") {
            format!("https://{}", url)
        } else {
            url.clone()
        };

        let output = Command::new("python3")
            .arg(brand_script)
            .arg(&url_with_scheme)
            .arg("--format")
            .arg("css")
            .output();

        match output {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                if stdout.trim().is_empty() {
                    continue;
                }
                all_brand_context.push_str(&format!(
                    "\n## Brand Analysis: {}\n{}\n",
                    url_with_scheme,
                    stdout.trim()
                ));
            }
            _ => {
                // Brand analysis failed for this URL, skip silently
                continue;
            }
        }
    }

    if all_brand_context.is_empty() {
        return String::new();
    }

    format!(
        "\n\n## Brand Identity (automatically extracted from website)\n{}\nUse the above brand colors, fonts, and style to make your response match this company's visual identity.",
        all_brand_context
    )
}

/// Build the full system prompt with RAG context, static company context, and brand context.
fn build_system_prompt(query: &str) -> String {
    let rag_context = get_rag_context(query);
    let static_context = get_static_company_context();
    let brand_context = get_brand_context(query);

    let mut prompt = MARKETING_CONTEXT.to_string();
    prompt.push_str(&static_context);
    prompt.push_str(&rag_context);
    prompt.push_str(&brand_context);
    prompt
}

/// Run Magpie in interactive agentic mode with real LLM responses.
pub async fn run_interactive(stream: bool) -> Result<()> {
    let llm = LLMClient::from_config()?;

    println!("🪶 Magpie — marketing assistant (agentic mode)");
    println!("Model: {} | Base URL: {}", llm.model, llm.base_url);
    println!("Type your request. Press Ctrl+C or type /exit to quit.\n");
    println!("💡 Magpie automatically analyzes websites in your request for brand identity.\n");

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

                println!("\n[Magpie agent] Analyzing context...\n");

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
    println!("Automatic website analysis:");
    println!("  \"write a landing page for https://example.com\"");
    println!("  \"write ad copy like https://competitor.com\"");
    println!("  Magpie will automatically extract brand colors, fonts, and logo");
    println!("");
    println!("Company setup:");
    println!("  magpie company-setup --website https://yourcompany.com");
    println!("  magpie company-setup --pdf brochure.pdf");
    println!("  magpie company-setup --website https://yourcompany.com --pdf brochure.pdf");
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
    fn test_build_system_prompt_no_context() {
        // Without knowledge/ directory or URLs, prompt should be unchanged
        let prompt = super::build_system_prompt("test query");
        assert_eq!(prompt, super::MARKETING_CONTEXT);
    }

    #[test]
    fn test_extract_urls() {
        let urls = super::extract_urls("Check out https://example.com and https://test.com/page");
        assert_eq!(urls.len(), 2);
        assert!(urls.contains(&"https://example.com".to_string()));
        assert!(urls.contains(&"https://test.com/page".to_string()));
    }

    #[test]
    fn test_extract_urls_no_urls() {
        let urls = super::extract_urls("No URLs here");
        assert!(urls.is_empty());
    }
}
