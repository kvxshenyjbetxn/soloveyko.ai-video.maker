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
struct ChatUsage {
    cost: Option<f64>,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
    usage: Option<ChatUsage>,
}

/// Надсилає запит до OpenRouter Chat API та повертає текст відповіді та її вартість.
pub fn call_openrouter(
    key: &str,
    model: &str,
    user_content: String,
    temperature: f32,
    job_info: Option<(u64, String)>,
) -> Result<(String, Option<f64>), String> {
    let log = |msg: &str| {
        if let Some((id, ref name)) = job_info {
            crate::logger::log_job(id, name, msg);
        } else {
            crate::logger::log(msg);
        }
    };

    log(&format!("Starting OpenRouter translation. Model: {}, Temperature: {}", model, temperature));

    let _permit = crate::api::openrouter::OpenRouterLimiter::get().acquire();

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
        .send_json(ureq::serde_json::to_value(&request).map_err(|e| {
            let err_msg = format!("Serialization error: {}", e);
            log(&err_msg);
            err_msg
        })?)
        .map_err(|e| {
            let err_msg = format!("Network error: {}", e);
            log(&err_msg);
            err_msg
        })?;

    let data = res
        .into_json::<ChatResponse>()
        .map_err(|e| {
            let err_msg = format!("Response parsing error: {}", e);
            log(&err_msg);
            err_msg
        })?;

    log("OpenRouter translation completed successfully.");

    let cost = data.usage.as_ref().and_then(|u| u.cost);
    if let Some(c) = cost {
        log(&format!("Request cost: ${:.5}", c));
    } else {
        log("Request cost not provided in usage.cost.");
    }

    let text = data.choices.into_iter()
        .next()
        .map(|c| c.message.content)
        .unwrap_or_default();

    Ok((text, cost))
}
