use crate::config::load_magpie_config;
use crate::error::{MagpieError, Result};
use crate::llm::LLMClient;
use crate::registry::SkillRegistry;
use axum::{
    extract::State,
    response::{Html, IntoResponse},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::Path;
use std::process::Command;
use std::result::Result as StdResult;

/// Shared application state for the web server.
#[derive(Clone)]
pub struct AppState {
    pub llm: LLMClient,
}

/// Request body for the chat endpoint.
#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub message: String,
    #[serde(default)]
    pub stream: bool,
}

/// Response from the chat endpoint.
#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub response: String,
}

/// Skill list item for the API.
#[derive(Debug, Serialize)]
pub struct SkillItem {
    pub id: String,
    pub name: String,
    pub source: String,
    pub description: String,
}

/// Company setup request.
#[derive(Debug, Deserialize)]
pub struct CompanySetupRequest {
    pub company_name: String,
    pub website_url: Option<String>,
}

/// Company setup response.
#[derive(Debug, Serialize)]
pub struct CompanySetupResponse {
    pub success: bool,
    pub message: String,
}

/// Create the axum router with all routes.
pub fn router(llm: LLMClient) -> Router {
    Router::new()
        .route("/", get(serve_index))
        .route("/api/chat", post(chat_handler))
        .route("/api/skills", get(list_skills_handler))
        .route("/api/config/llm", get(llm_config_handler))
        .route("/api/company-setup", post(company_setup_handler))
        .with_state(AppState { llm })
}

/// Serve the main HTML page.
async fn serve_index() -> StdResult<Html<String>, StatusCode> {
    let html = include_str!("../web_ui.html");
    Ok(Html(html.to_string()))
}

/// Handle chat requests — calls the LLM and returns the response.
async fn chat_handler(
    State(state): State<AppState>,
    Json(payload): Json<ChatRequest>,
) -> StdResult<Json<ChatResponse>, StatusCode> {
    // Read marketing context from agent module
    let system_prompt = crate::agent::MARKETING_CONTEXT;

    let response = if state.llm.api_key.is_empty() {
        "⚠️ No LLM API key configured. Please run `magpie config llm-setup` to set up your AI provider, then try again.".to_string()
    } else {
        state
            .llm
            .chat(system_prompt, &payload.message)
            .await
            .map_err(|_e: MagpieError| StatusCode::INTERNAL_SERVER_ERROR)?
    };

    Ok(Json(ChatResponse { response }))
}

/// List available skills.
async fn list_skills_handler() -> StdResult<Json<Vec<SkillItem>>, StatusCode> {
    let registry = SkillRegistry::new()
        .map_err(|_e: MagpieError| StatusCode::INTERNAL_SERVER_ERROR)?;
    let skills = crate::registry::list_skills(&registry, None, None)
        .await
        .map_err(|_e: MagpieError| StatusCode::INTERNAL_SERVER_ERROR)?;

    let items: Vec<SkillItem> = skills
        .into_iter()
        .map(|s| SkillItem {
            id: s.id().to_string(),
            name: s.name().to_string(),
            source: s.source().to_string(),
            description: s.description().to_string(),
        })
        .collect();

    Ok(Json(items))
}

/// Show LLM configuration status.
async fn llm_config_handler() -> StdResult<Json<serde_json::Value>, StatusCode> {
    let config = load_magpie_config()
        .map_err(|_e: MagpieError| StatusCode::INTERNAL_SERVER_ERROR)?;
    let llm = &config.llm_provider;

    Ok(Json(serde_json::json!({
        "configured": llm.api_key.is_some(),
        "base_url": llm.base_url.as_deref().unwrap_or("https://api.openai.com/v1"),
        "model": llm.model.as_deref().unwrap_or("gpt-4o-mini"),
    })))
}

/// Handle company setup — saves company info and runs crawler/brand analysis/RAG ingest.
async fn company_setup_handler(
    Json(payload): Json<CompanySetupRequest>,
) -> StdResult<Json<CompanySetupResponse>, StatusCode> {
    let company_name = payload.company_name.trim();
    if company_name.is_empty() {
        return Ok(Json(CompanySetupResponse {
            success: false,
            message: "Company name is required".to_string(),
        }));
    }

    // Save company name to knowledge/company-name.txt
    let knowledge_dir = Path::new("knowledge");
    if !knowledge_dir.exists() {
        std::fs::create_dir_all(knowledge_dir)
            .map_err(|_e| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    let company_file = knowledge_dir.join("company-name.txt");
    std::fs::write(&company_file, company_name)
        .map_err(|_e| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut messages: Vec<String> = Vec::new();
    messages.push(format!("✅ Company name saved: {}", company_name));

    // If website URL provided, run crawl + brand analysis + RAG ingest
    if let Some(url) = &payload.website_url {
        let url_trimmed = url.trim();
        if !url_trimmed.is_empty() {
            let url_with_scheme = if !url_trimmed.starts_with("http://")
                && !url_trimmed.starts_with("https://")
            {
                format!("https://{}", url_trimmed)
            } else {
                url_trimmed.to_string()
            };

            // Crawl the website
            let crawl_output = Command::new("python3")
                .arg("skills/crawl/magpie-crawl.py")
                .arg(&url_with_scheme)
                .arg("--format")
                .arg("json")
                .arg("--out")
                .arg("knowledge/crawled-pages.json")
                .output();

            match crawl_output {
                Ok(out) if out.status.success() => {
                    messages.push(format!("✅ Crawled: {}", url_with_scheme));
                }
                _ => {
                    messages.push(format!("⚠️  Crawl failed for: {}", url_with_scheme));
                }
            }

            // Brand analysis (needs API key)
            let api_key = std::env::var("MAGPIE_API_KEY")
                .or_else(|_| std::env::var("OPENAI_API_KEY"))
                .unwrap_or_default();
            if !api_key.is_empty() {
                let brand_output = Command::new("python3")
                    .arg("skills/brand-analysis/magpie-brand-analysis.py")
                    .arg(&url_with_scheme)
                    .arg("--format")
                    .arg("md")
                    .arg("--out")
                    .arg("knowledge/brand-analysis.md")
                    .output();

                match brand_output {
                    Ok(out) if out.status.success() => {
                        messages.push(format!("✅ Brand analysis: {}", url_with_scheme));
                    }
                    _ => {
                        messages.push(format!("⚠️  Brand analysis failed for: {}", url_with_scheme));
                    }
                }
            } else {
                messages.push("⚠️  Brand analysis skipped (no MAGPIE_API_KEY)".to_string());
            }
        }
    }

    // Run RAG ingest
    let rag_output = Command::new("python3")
        .arg("skills/rag/magpie-rag.py")
        .arg("ingest")
        .output();

    match rag_output {
        Ok(out) if out.status.success() => {
            messages.push("✅ RAG index built".to_string());
        }
        _ => {
            messages.push("⚠️  RAG ingest failed".to_string());
        }
    }

    Ok(Json(CompanySetupResponse {
        success: true,
        message: messages.join("\n"),
    }))
}

/// Start the web server on port 7070.
pub async fn start_web_server() -> Result<()> {
    // Try to load LLM config, but don't fail if not configured yet.
    let llm = LLMClient::from_config().unwrap_or_else(|_| {
        // Return a client that will produce a helpful error when used.
        LLMClient {
            client: reqwest::Client::new(),
            base_url: "https://api.openai.com/v1".to_string(),
            api_key: String::new(),
            model: "gpt-4o-mini".to_string(),
            temperature: 0.7,
        }
    });
    let app = router(llm);

    let addr: SocketAddr = "0.0.0.0:7070".parse().expect("invalid socket address");
    let listener = tokio::net::TcpListener::bind(addr).await.map_err(|e| {
        MagpieError::Custom(format!("Failed to bind to port 7070: {e}"))
    })?;

    println!("🪶 Magpie web interface running at http://{addr}");
    println!("   Open your browser to http://localhost:7070");

    axum::serve(listener, app)
        .await
        .map_err(|e| MagpieError::Custom(format!("Web server error: {e}")))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_item_serialization() {
        let item = SkillItem {
            id: "launch-email".to_string(),
            name: "Launch Email".to_string(),
            source: "magpie".to_string(),
            description: "Write launch emails".to_string(),
        };
        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("launch-email"));
    }
}
