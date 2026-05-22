use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
}

#[derive(Deserialize)]
struct ChatMessageContent {
    content: String,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessageContent,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

/// Надсилає запит до OpenRouter Chat API та повертає текст відповіді.
pub fn call_openrouter(
    key: &str,
    model: &str,
    user_content: String,
    temperature: f32,
) -> Result<String, String> {
    let request = ChatRequest {
        model: model.to_string(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: user_content,
        }],
        temperature,
    };

    let agent = ureq::AgentBuilder::new()
        .timeout_connect(std::time::Duration::from_secs(15))
        .timeout(std::time::Duration::from_secs(120))
        .build();

    let res = agent
        .post("https://openrouter.ai/api/v1/chat/completions")
        .set("Authorization", &format!("Bearer {}", key))
        .set("Content-Type", "application/json")
        .send_json(ureq::serde_json::to_value(&request).map_err(|e| e.to_string())?)
        .map_err(|e| format!("Помилка мережі: {}", e))?;

    let data = res
        .into_json::<ChatResponse>()
        .map_err(|e| format!("Помилка парсингу відповіді: {}", e))?;

    Ok(data.choices.into_iter()
        .next()
        .map(|c| c.message.content)
        .unwrap_or_default())
}
