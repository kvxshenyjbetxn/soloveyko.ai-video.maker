use std::process::Command;

/// Викликає Claude CLI для виконання перекладу сценарію або іншого тексту та повертає результат.
///
/// Використовує прапорці `--model` та `-p` для отримання результату від Claude Code.
pub fn call_claude_code(
    model: &str,
    user_content: &str,
) -> Result<String, String> {
    crate::logger::log(&format!("Запуск Claude CLI для перекладу. Модель: {}", model));

    #[cfg(target_os = "windows")]
    let mut cmd = Command::new("cmd");
    #[cfg(target_os = "windows")]
    cmd.args(&["/C", "claude"]);

    #[cfg(not(target_os = "windows"))]
    let mut cmd = Command::new("claude");

    // Запускаємо: claude --model <model> -p "<prompt>"
    cmd.arg("--model")
        .arg(model)
        .arg("-p")
        .arg(user_content);

    // Записуємо інформацію про команду у лог
    let debug_command = format!(
        "claude --model {} -p \"[текст промпту та сценарію]\"",
        model
    );
    crate::logger::log(&format!("Виконується: {}", debug_command));

    let output = cmd.output().map_err(|e| {
        let err_msg = format!("Не вдалося запустити claude cli: {}. Перевірте, чи встановлено claude CLI та чи додано його в PATH.", e);
        crate::logger::log(&err_msg);
        err_msg
    })?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        crate::logger::log("Claude CLI успішно виконав переклад.");
        Ok(stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let err_msg = format!(
            "Claude CLI помилка (код статусу: {:?}).\n--- STDERR ---\n{}\n--- STDOUT ---\n{}",
            output.status.code(),
            stderr,
            stdout
        );
        crate::logger::log(&err_msg);
        Err(format!("Claude CLI помилка: {}", stderr))
    }
}
