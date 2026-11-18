use crate::config::load_magpie_config;
use crate::error::Result;
use crate::llm::LLMClient;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use regex::Regex;

/// Marketing system prompt that frames Magpie as a marketing assistant.
pub const MARKETING_CONTEXT: &str = "You are Magpie, a marketing-focused AI assistant.\n\
Help the user with marketing tasks: copywriting, campaign planning, SEO, ads, email,\nsocial media, landing pages, brand voice, and go-to-market messaging.\n\
Prefer clear, persuasive, conversion-oriented output. When relevant, use installed\nmarketing skills to ground your work.";

/// Resolve the base directory: prefer MAGPIE_HOME, then binary dir, then current dir.
fn magpie_base_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("MAGPIE_HOME") {
        return PathBuf::from(dir);
    }
    // Try binary location first
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            if parent.join("knowledge").is_dir() {
                return parent.to_path_buf();
            }
        }
    }
    // Check current dir and parents for knowledge/
    if let Ok(cwd) = std::env::current_dir() {
        let mut dir = cwd.clone();
        loop {
            if dir.join("knowledge").is_dir() {
                return dir;
            }
            if !dir.pop() {
                break;
            }
        }
        return cwd;
    }
    PathBuf::from(".")
}

/// Default knowledge directory.
fn knowledge_dir() -> PathBuf {
    magpie_base_dir().join("knowledge")
}

/// Default RAG script path.
fn rag_script_path() -> PathBuf {
    magpie_base_dir().join("skills/rag/magpie-rag.py")
}

/// Default brand analysis script path.
fn brand_script_path() -> PathBuf {
    magpie_base_dir().join("skills/brand-analysis/magpie-brand-analysis.py")
}

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
pub fn get_rag_context(query: &str) -> String {
    let knowledge_dir = knowledge_dir();
    let rag_script = rag_script_path();

    if !knowledge_dir.is_dir() {
        return String::new();
    }
    if !rag_script.is_file() {
        return String::new();
    }

    let index_file = knowledge_dir.join(".rag-index.json");
    if !index_file.exists() {
        // Auto-ingest if no index exists
        let _ = Command::new("python3")
            .arg(rag_script.clone())
            .arg("ingest")
            .output();
    }

    let output = Command::new("python3")
        .arg(rag_script.clone())
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

/// Load static company context files (company-name.txt, company-summary.md,
/// brand-analysis.md, crawled-pages.json) that are always included.
pub fn get_static_company_context() -> String {
    let knowledge_dir = knowledge_dir();
    if !knowledge_dir.is_dir() {
        return String::new();
    }

    let mut context_parts: Vec<String> = Vec::new();

    // Company name (most important - show first)
    let company_name_file = knowledge_dir.join("company-name.txt");
    if let Ok(name) = std::fs::read_to_string(&company_name_file) {
        let trimmed = name.trim();
        if !trimmed.is_empty() {
            context_parts.push(format!("## Company: {}", trimmed));
        }
    }

    // Company summary
    let summary_file = knowledge_dir.join("company-summary.md");
    if summary_file.exists() {
        if let Ok(content) = std::fs::read_to_string(&summary_file) {
            if !content.trim().is_empty() {
                context_parts.push(format!("## Company Summary\n{}", content.trim()));
            }
        }
    }

    // Brand analysis (colors, fonts, logo from website)
    let brand_file = knowledge_dir.join("brand-analysis.md");
    if brand_file.exists() {
        if let Ok(content) = std::fs::read_to_string(&brand_file) {
            if !content.trim().is_empty() {
                context_parts.push(format!("## Brand Identity (from website)\n{}", content.trim()));
            }
        }
    }

    // Crawled website content
    let crawled_file = knowledge_dir.join("crawled-pages.json");
    if crawled_file.exists() {
        if let Ok(content) = std::fs::read_to_string(&crawled_file) {
            if !content.trim().is_empty() {
                context_parts.push(format!("## Crawled Website Data\n{}", content.trim()));
            }
        }
    }

    if context_parts.is_empty() {
        String::new()
    } else {
        format!(
            "\n\n## COMPANY CONTEXT (use this for all marketing)\n{}",
            context_parts.join("\n\n")
        )
    }
}

/// Automatically analyze websites mentioned in user input and return brand context.
fn get_brand_context(query: &str) -> String {
    let urls = extract_urls(query);
    if urls.is_empty() {
        return String::new();
    }

    let brand_script = brand_script_path();
    if !brand_script.is_file() {
        return String::new();
    }

    let mut all_brand_context = String::new();

    for url in urls {
        let url_with_scheme = if !url.starts_with("http://") && !url.starts_with("https://") {
            format!("https://{}", url)
        } else {
            url.clone()
        };

        let output = Command::new("python3")
            .arg(brand_script.clone())
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

/// Build the full system prompt with company context as the PRIMARY focus.
pub fn build_system_prompt(query: &str) -> String {
    let static_context = get_static_company_context();
    let rag_context = get_rag_context(query);
    let brand_context = get_brand_context(query);

    let mut prompt = MARKETING_CONTEXT.to_string();

    // Company context comes FIRST - it's the most important
    if !static_context.is_empty() {
        prompt.push_str(&static_context);
    }

    // RAG context from knowledge base
    if !rag_context.is_empty() {
        prompt.push_str(&rag_context);
    }

    // Brand context from website analysis
    if !brand_context.is_empty() {
        prompt.push_str(&brand_context);
    }

    // If no company context at all, warn the user
    if static_context.is_empty() && rag_context.is_empty() && brand_context.is_empty() {
        prompt.push_str("\n\n## WARNING: No company context found\n");
        prompt.push_str("Run 'magpie company-setup --website https://yourcompany.com' to set up company context.\n");
        prompt.push_str("Without company context, all marketing will be generic.\n");
    }

    // Debug: print what context is being used (visible in terminal)
    eprintln!("\n[Magpie Debug] System prompt context loaded:");
    eprintln!("  - Static company context: {} chars", static_context.len());
    eprintln!("  - RAG context: {} chars", rag_context.len());
    eprintln!("  - Brand context: {} chars", brand_context.len());
    eprintln!("  - Knowledge dir: {:?}", knowledge_dir());
    eprintln!("  - Knowledge dir exists: {}", knowledge_dir().is_dir());

    prompt
}

/// Run Magpie in interactive agentic mode with real LLM responses.
pub async fn run_interactive(stream: bool) -> Result<()> {
    let llm = LLMClient::from_config()?;

    println!("🪶 Magpie — marketing assistant (agentic mode)");
    println!("Model: {} | Base URL: {}", llm.model, llm.base_url);
    println!("Knowledge dir: {:?}", knowledge_dir());
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
    println!("Company context:");
    println!("  Drop documents into knowledge/ and Magpie will use them automatically.");
    println!("  Run 'magpie company-setup --website https://yourcompany.com' to set up from a website.");
    println!("");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_marketing_context_non_empty() {
        assert!(!super::MARKETING_CONTEXT.is_empty());
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
