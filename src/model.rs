use crate::error::Result;

/// A model available to Magpie (same list style as `opencode models`).
#[derive(Debug, Clone)]
pub struct Model {
    pub id: String,
    pub provider: String,
}

/// List available models (placeholder - in a full build this queries providers).
pub fn list_models(provider: Option<&str>) -> Result<Vec<Model>> {
    let models = vec![
        // NVIDIA NIM (default)
        ("nvidia/nemotron-3.5-super-vl-free", "nvidia"),
        ("nvidia/nemotron-3.5-lightning-free", "nvidia"),
        ("nvidia/nemotron-4-340b-reward", "nvidia"),
        ("nvidia/llama-3.1-nemotron-70b-instruct", "nvidia"),
        ("nvidia/llama-3.1-nemotron-8b-instruct", "nvidia"),
        // OpenAI-compatible
        ("openai/gpt-4o-mini", "openai"),
        ("openai/gpt-4o", "openai"),
        ("openai/gpt-3.5-turbo", "openai"),
        // OpenRouter
        ("openrouter/anthropic/claude-3.5-sonnet", "openrouter"),
        ("openrouter/meta-llama/llama-3.1-405b-instruct", "openrouter"),
        // Local
        ("ollama/llama3.1", "ollama"),
        ("lmstudio/local-model", "lmstudio"),
    ];

    let mut out = Vec::new();
    for (id, prov) in models {
        if let Some(p) = provider {
            if prov != p {
                continue;
            }
        }
        out.push(Model {
            id: id.to_string(),
            provider: prov.to_string(),
        });
    }
    Ok(out)
}
