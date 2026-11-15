use crate::config::{
    load_magpie_config, load_opencode_skill_sources, remove_opencode_skill_source,
    register_opencode_skill_source, save_magpie_config,
};
use crate::error::Result;
use clap::Args;
use clap::Subcommand;
use inquire::Confirm;
use inquire::Select;
use inquire::Text;

/// Manage Magpie / OpenCode skill sources and LLM provider settings.
#[derive(Subcommand, Debug)]
pub enum ConfigCommand {
    /// List configured OpenCode skill sources
    Sources(SourcesArgs),
    /// Register a skill catalog (URL or path) in OpenCode
    SourcesAdd(SourcesAddArgs),
    /// Remove a skill catalog from OpenCode config
    SourcesRemove(SourcesRemoveArgs),
    /// Configure the LLM provider (API key, base URL, model)
    LLMSetup,
    /// Show current LLM provider configuration
    LLMShow,
}

#[derive(Args, Debug)]
pub struct SourcesArgs;

#[derive(Args, Debug)]
pub struct SourcesAddArgs {
    /// Skill catalog URL or path
    source: String,
}

#[derive(Args, Debug)]
pub struct SourcesRemoveArgs {
    /// Skill catalog URL or path
    source: String,
}

pub fn handle(cmd: ConfigCommand) -> Result<()> {
    match cmd {
        ConfigCommand::Sources(_) => {
            let sources = load_opencode_skill_sources()?;
            if sources.is_empty() {
                println!("No skill sources configured yet.");
                println!("Add one: magpie config sources-add https://example.com/skills/");
            } else {
                println!("\nConfigured skill sources:\n");
                for (i, s) in sources.iter().enumerate() {
                    println!("  {}. {}", i + 1, s);
                }
                println!();
            }
        }
        ConfigCommand::SourcesAdd(args) => {
            register_opencode_skill_source(&args.source)?;
            println!("✅ Registered skill source: {}", args.source);
        }
        ConfigCommand::SourcesRemove(args) => {
            let confirmed = Confirm::new(&format!("Remove skill source \"{}\"?", args.source))
                .with_default(false)
                .prompt()?;
            if confirmed {
                remove_opencode_skill_source(&args.source)?;
                println!("✅ Removed skill source: {}", args.source);
            } else {
                println!("Cancelled.");
            }
        }
        ConfigCommand::LLMSetup => {
            let mut config = load_magpie_config()?;

            let providers = [
                "OpenAI (https://api.openai.com/v1)",
                "OpenRouter (https://openrouter.ai/api/v1)",
                "Ollama (http://localhost:11434/v1)",
                "LM Studio (http://localhost:1234/v1)",
                "Custom",
            ];
            let choice = Select::new("Select LLM provider:", providers.to_vec()).prompt()?;

            let base_url = match choice {
                "OpenAI (https://api.openai.com/v1)" => "https://api.openai.com/v1".to_string(),
                "OpenRouter (https://openrouter.ai/api/v1)" => {
                    "https://openrouter.ai/api/v1".to_string()
                }
                "Ollama (http://localhost:11434/v1)" => {
                    "http://localhost:11434/v1".to_string()
                }
                "LM Studio (http://localhost:1234/v1)" => {
                    "http://localhost:1234/v1".to_string()
                }
                "Custom" => {
                    Text::new("Enter custom base URL:").prompt()?
                }
                _ => "https://api.openai.com/v1".to_string(),
            };

            let api_key = if choice.starts_with("Ollama") || choice.starts_with("LM Studio") {
                // Local providers often don't need a real key
                Text::new("Enter API key (or leave blank for local):")
                    .with_help_message("Local LLMs often accept any value")
                    .prompt()?
            } else {
                Text::new("Enter API key:").prompt()?
            };
            let api_key = if api_key.is_empty() { None } else { Some(api_key) };

            let model = Text::new("Enter model name (e.g. gpt-4o-mini, llama3.1):")
                .with_initial_value("gpt-4o-mini")
                .prompt()?;
            let model = if model.is_empty() { None } else { Some(model) };

            let temp_text = Text::new("Enter temperature (0.0-2.0, default 0.7):")
                .with_initial_value("0.7")
                .prompt()?;
            let temperature: Option<f32> = if temp_text.is_empty() {
                None
            } else {
                temp_text.parse().ok()
            };

            config.llm_provider = crate::config::LLMProviderConfig {
                base_url: Some(base_url),
                api_key,
                model,
                temperature,
            };

            save_magpie_config(&config)?;
            println!("✅ LLM provider configured. You can now run: magpie run \"your message\"");
        }
        ConfigCommand::LLMShow => {
            let config = load_magpie_config()?;
            let llm = &config.llm_provider;
            println!("\n🔧 LLM Provider Configuration:\n");
            println!("  Base URL:  {}", llm.base_url.as_deref().unwrap_or("<not set>"));
            println!("  Model:     {}", llm.model.as_deref().unwrap_or("<not set>"));
            println!(
                "  Temperature: {}",
                llm.temperature.unwrap_or(0.7)
            );
            if llm.api_key.is_some() {
                println!("  API Key:   ***set***");
            } else {
                println!("  API Key:   <not set>");
            }
            println!();
        }
    }
    Ok(())
}
