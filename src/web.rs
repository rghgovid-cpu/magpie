use crate::config::load_magpie_config;
use crate::error::{MagpieError, Result};
use crate::llm::LLMClient;
use crate::registry::SkillRegistry;
use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
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

/// Create the axum router with all routes.
pub fn router(llm: LLMClient) -> Router {
    Router::new()
        .route("/", get(serve_index))
        .route("/api/chat", post(chat_handler))
        .route("/api/skills", get(list_skills_handler))
        .route("/api/config/llm", get(llm_config_handler))
        .with_state(AppState { llm })
}

/// Serve the main HTML page.
async fn serve_index() -> StdResult<(StatusCode, String), StatusCode> {
    let html = include_str!("../web_ui.html");
    Ok((StatusCode::OK, html.to_string()))
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
