use crate::config::load_magpie_config;
use crate::error::{MagpieError, Result};
use futures_util::stream::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

/// LLM chat completion request (OpenAI-compatible).
#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: Option<f32>,
    stream: Option<bool>,
    max_tokens: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ChatMessage {
    role: String,
    content: String,
}

/// LLM chat completion response.
#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    #[allow(dead_code)]
    id: Option<String>,
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ChatMessage,
    #[allow(dead_code)]
    finish_reason: Option<String>,
}

/// Streaming chunk (SSE).
#[derive(Debug, Deserialize)]
struct StreamChunk {
    choices: Vec<StreamChoice>,
}

#[derive(Debug, Deserialize)]
struct StreamChoice {
    #[allow(dead_code)]
    index: Option<u32>,
    delta: StreamDelta,
}

#[derive(Debug, Deserialize, Default)]
struct StreamDelta {
    #[allow(dead_code)]
    role: Option<String>,
    content: Option<String>,
}

/// LLM client that talks to any OpenAI-compatible chat/completions API.
pub struct LLMClient {
    pub client: Client,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub temperature: f32,
}

impl LLMClient {
    /// Create a client from Magpie config, falling back to environment variables.
    pub fn from_config() -> Result<Self> {
        let config = load_magpie_config()?;
        let llm = &config.llm_provider;

        let base_url = llm
            .base_url
            .as_deref()
            .unwrap_or("https://api.openai.com/v1")
            .trim_end_matches('/')
            .to_string();

        let api_key = llm
            .api_key
            .clone()
            .or_else(|| env::var("MAGPIE_API_KEY").ok())
            .or_else(|| env::var("OPENAI_API_KEY").ok())
            .ok_or_else(|| {
                MagpieError::Custom(
                    "No LLM API key configured. Set MAGPIE_API_KEY or OPENAI_API_KEY env var, or run `magpie config llm-setup`."
                        .to_string(),
                )
            })?;

        let model = llm
            .model
            .clone()
            .unwrap_or_else(|| "gpt-4o-mini".to_string());

        let temperature = llm.temperature.unwrap_or(0.7);

        Ok(Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()?,
            base_url,
            api_key,
            model,
            temperature,
        })
    }

    /// Send a chat completion and return the assistant's text.
    pub async fn chat(&self, system_prompt: &str, user_message: &str) -> Result<String> {
        let body = ChatCompletionRequest {
            model: self.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: user_message.to_string(),
                },
            ],
            temperature: Some(self.temperature),
            stream: Some(false),
            max_tokens: None,
        };

        let url = format!("{}/chat/completions", self.base_url);

        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?
            .error_for_status()?;

        let data: ChatCompletionResponse = resp.json().await?;

        data.choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| MagpieError::Custom("LLM returned no response".to_string()))
    }

    /// Send a chat completion with streaming output.
    pub async fn chat_stream(
        &self,
        system_prompt: &str,
        user_message: &str,
    ) -> Result<()> {
        use std::io::Write;

        let body = ChatCompletionRequest {
            model: self.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: user_message.to_string(),
                },
            ],
            temperature: Some(self.temperature),
            stream: Some(true),
            max_tokens: None,
        };

        let url = format!("{}/chat/completions", self.base_url);

        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?
            .error_for_status()?;

        let stream = resp.bytes_stream();
        let mut stream = Box::pin(stream);

        while let Some(chunk) = stream.next().await {
            let bytes = chunk.map_err(|e| MagpieError::Custom(format!("stream error: {e}")))?;
            let text = std::str::from_utf8(&bytes).unwrap_or("");

            for line in text.lines() {
                let line = line.trim();
                if let Some(data) = line.strip_prefix("data: ") {
                    if data == "[DONE]" {
                        continue;
                    }
                    if let Ok(chunk) = serde_json::from_str::<StreamChunk>(data) {
                        if let Some(content) = &chunk.choices.first().and_then(|c| c.delta.content.clone()) {
                            print!("{content}");
                            std::io::stdout().flush()?;
                        }
                    }
                }
            }
        }
        println!();
        Ok(())
    }
}

impl Default for LLMClient {
    fn default() -> Self {
        // This is only used as a placeholder; prefer from_config().
        Self {
            client: Client::new(),
            base_url: "https://api.openai.com/v1".to_string(),
            api_key: String::new(),
            model: "gpt-4o-mini".to_string(),
            temperature: 0.7,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_message_serialization() {
        let msg = ChatMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("Hello"));
    }
}
