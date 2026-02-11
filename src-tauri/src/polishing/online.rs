use serde::{Deserialize, Serialize};

/// Online polishing via OpenAI-compatible chat completions API.
/// Works with Step-Fun, DeepSeek, Qwen-Turbo, Groq, OpenAI, etc.
pub struct OnlinePolisher {
    agent: ureq::Agent,
    endpoint: String,
    api_key: String,
    model: String,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatResponseMessage,
}

#[derive(Deserialize)]
struct ChatResponseMessage {
    content: String,
}

impl OnlinePolisher {
    pub fn new(endpoint: &str, api_key: &str, model: &str) -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout_read(std::time::Duration::from_secs(10))
            .timeout_write(std::time::Duration::from_secs(5))
            .build();

        // Ensure endpoint ends with /chat/completions
        let endpoint = if endpoint.ends_with("/chat/completions") {
            endpoint.to_string()
        } else {
            let base = endpoint.trim_end_matches('/');
            format!("{}/chat/completions", base)
        };

        Self {
            agent,
            endpoint,
            api_key: api_key.to_string(),
            model: model.to_string(),
        }
    }

    /// Polish text via online API. The prompt_template should contain {text} and {lang} placeholders.
    pub fn polish(
        &self,
        text: &str,
        prompt_template: &str,
        lang: &str,
    ) -> anyhow::Result<String> {
        if text.trim().is_empty() {
            return Ok(text.to_string());
        }

        // Build user message from template
        let user_content = prompt_template
            .replace("{text}", text)
            .replace("{lang}", lang);

        let request = ChatRequest {
            model: self.model.clone(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: user_content,
            }],
            temperature: 0.1,
            max_tokens: 512,
        };

        println!(
            "[polish:api] POST {} model={} input={} chars",
            self.endpoint,
            self.model,
            text.len()
        );

        let start = std::time::Instant::now();
        let response = self
            .agent
            .post(&self.endpoint)
            .set("Authorization", &format!("Bearer {}", self.api_key))
            .set("Content-Type", "application/json")
            .send_json(ureq::json!({
                "model": request.model,
                "messages": request.messages.iter().map(|m| {
                    serde_json::json!({ "role": m.role, "content": m.content })
                }).collect::<Vec<_>>(),
                "temperature": request.temperature,
                "max_tokens": request.max_tokens,
            }))
            .map_err(|e| anyhow::anyhow!("API request failed: {}", e))?;

        let body: ChatResponse = response
            .into_json()
            .map_err(|e| anyhow::anyhow!("Failed to parse API response: {}", e))?;

        let result = body
            .choices
            .first()
            .map(|c| c.message.content.trim().to_string())
            .unwrap_or_default();

        println!(
            "[polish:api] result: '{}' ({:.1}s)",
            &result[..result.len().min(80)],
            start.elapsed().as_secs_f64()
        );

        if result.is_empty() {
            anyhow::bail!("API returned empty response");
        }

        Ok(result)
    }
}
