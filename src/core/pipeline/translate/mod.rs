mod translate;

/// Синхронно виконує переклад тексту та повертає результат.
/// Підставляє `{{text}}` у промт перед викликом сервісу.
pub fn translate_text(
    service: &str,
    key: &str,
    model: &str,
    prompt: &str,
    text: &str,
    temperature: f32,
    job_info: Option<(u64, String)>,
) -> Result<String, String> {
    let user_content = if prompt.contains("{{text}}") {
        prompt.replace("{{text}}", text)
    } else if !prompt.is_empty() {
        format!("{}\n\n{}", prompt, text)
    } else {
        text.to_string()
    };

    if service == "Claude Code" {
        crate::api::claude::call_claude_code(model, &user_content, job_info)
    } else if service == "Gemini CLI" {
        crate::api::gemini::call_gemini_cli(model, &user_content, job_info)
    } else {
        translate::call_openrouter(key, model, user_content, temperature, job_info)
    }
}
