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
        ("opencode/nemotron-3.5-super-vl-free", "opencode"),
        ("opencode/nemotron-3.5-lightning-free", "opencode"),
        ("opencode/deepseek-v4.1-flash-free", "opencode"),
        ("opencode/longcat-2.5-free", "opencode"),
        ("opencode/minimax-m3.1-flash-free", "opencode"),
        ("opencode/glm-5.3-flash-free", "opencode"),
        ("opencode/inkling-2-small-free", "opencode"),
        ("opencode/flint-40b-free", "opencode"),
        ("opencode/big-pickle", "opencode"),
        ("opencode/beam-free", "opencode"),
        ("opencode/iceman-free", "opencode"),
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
