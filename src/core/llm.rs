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
    // OpenRouter може повернути null коли модель відмовила або повернула порожній результат
    content: Option<String>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessageContent,
    finish_reason: Option<String>,
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
/// При порожній відповіді повторює до 5 разів.
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

    const MAX_RETRIES: u32 = 5;

    let agent = ureq::AgentBuilder::new()
        .timeout_connect(std::time::Duration::from_secs(15))
        .timeout(std::time::Duration::from_secs(120))
        .build();

    let request = ChatRequest {
        model: model.to_string(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: user_content,
        }],
        temperature,
    };

    let request_value = ureq::serde_json::to_value(&request).map_err(|e| {
        format!("Serialization error: {}", e)
    })?;

    for attempt in 1..=MAX_RETRIES {
        log(&format!("OpenRouter request. Model: {}, attempt {}/{}", model, attempt, MAX_RETRIES));

        let _permit = crate::api::openrouter::OpenRouterLimiter::get().acquire();

        let res = match agent
            .post("https://openrouter.ai/api/v1/chat/completions")
            .set("Authorization", &format!("Bearer {}", key))
            .set("Content-Type", "application/json")
            .send_json(request_value.clone())
        {
            Ok(r) => r,
            Err(e) => {
                log(&format!("Network error (attempt {}): {}", attempt, e));
                continue;
            }
        };

        let data = match res.into_json::<ChatResponse>() {
            Ok(d) => d,
            Err(e) => {
                log(&format!("Response parsing error (attempt {}): {}", attempt, e));
                continue;
            }
        };

        let cost = data.usage.as_ref().and_then(|u| u.cost);

        let choice = data.choices.into_iter().next();
        let finish_reason = choice.as_ref()
            .and_then(|c| c.finish_reason.as_deref())
            .unwrap_or("unknown")
            .to_string();
        let text = choice.and_then(|c| c.message.content).unwrap_or_default();

        if text.trim().is_empty() {
            log(&format!("Порожня відповідь (attempt {}/{}, finish_reason: {}), повтор...", attempt, MAX_RETRIES, finish_reason));
            continue;
        }

        if let Some(c) = cost {
            log(&format!("Request cost: ${:.5}", c));
        }

        return Ok((text, cost));
    }

    Err(format!("LLM не повернув відповідь після {} спроб", MAX_RETRIES))
}

/// Викликає LLM-сервіс із підстановкою `{{text}}` у промт і повертає текст відповіді та вартість.
///
/// `allow_tools` = true потрібно для агентного режиму (Claude Code з `--allowedTools Bash,Write,Read`).
pub fn call_llm(
    service: &str,
    key: &str,
    model: &str,
    prompt: &str,
    text: &str,
    temperature: f32,
    job_info: Option<(u64, String)>,
    working_dir: Option<&str>,
    allow_tools: bool,
) -> Result<(String, Option<f64>), String> {
    let user_content = if prompt.contains("{{text}}") {
        prompt.replace("{{text}}", text)
    } else if !prompt.is_empty() {
        format!("{}\n\n{}", prompt, text)
    } else {
        text.to_string()
    };

    if service == "Claude Code" {
        crate::api::claude::call_claude_code(model, &user_content, job_info, working_dir, allow_tools).map(|res| (res, None))
    } else if service == "Gemini CLI" {
        crate::api::gemini::call_gemini_cli(model, &user_content, job_info, working_dir, allow_tools).map(|res| (res, None))
    } else if service == "Codex CLI" {
        crate::api::codex::call_codex(model, &user_content, job_info, working_dir, allow_tools).map(|res| (res, None))
    } else if service == "AGY CLI" {
        crate::api::agy::call_agy_cli(model, &user_content, job_info, working_dir, allow_tools).map(|res| (res, None))
    } else if service == "Pi CLI" {
        crate::api::pi::call_pi_cli(model, &user_content, job_info, working_dir, allow_tools).map(|res| (res, None))
    } else {
        call_openrouter(key, model, user_content, temperature, job_info)
    }
}
