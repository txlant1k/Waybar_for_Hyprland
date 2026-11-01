//! Общие вспомогательные функции для всех модулей.

/// Экранирует строку для вставки внутрь JSON-строки.
///
/// Заменяет `\` на `\\` и `"` на `\"`, чтобы названия песен или сетей
/// с кавычками не ломали JSON, который читает Waybar.
pub fn escape_json(s: &str) -> String {
    let mut result = String::new();
    for c in s.chars() {
        match c {
            '\\' => result.push_str("\\\\"),
            '"' => result.push_str("\\\""),
            _ => result.push(c),
        }
    }
    result
}