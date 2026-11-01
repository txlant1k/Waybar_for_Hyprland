use std::process::Command;

/// Убирает из названия всё, что стоит в круглых и квадратных скобках.
///
/// Например, `Song (feat. X) [Live]` превращается в `Song`.
/// Счётчик глубины нужен для вложенных скобок: `Song (Remix (Extended))`.
fn strip_brackets(s: &str) -> String {
    let mut result = String::new();
    // насколько глубоко мы внутри скобок; 0 — снаружи
    let mut depth: u32 = 0;
    for c in s.chars() {
        match c {
            '(' | '[' => depth += 1,
            // лишняя закрывающая скобка без пары не уводит счётчик в минус
            ')' | ']' => depth = depth.saturating_sub(1),
            _ if depth == 0 => result.push(c),
            _ => {}
        }
    }
    // на месте скобок остаются лишние пробелы: схлопываем их в один
    result.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn read_music() -> std::io::Result<String> {
    let output = Command::new("playerctl")
        .args(["metadata", "--format", "{{artist}} - {{title}}"])
        .output();
        
    match output {
        Ok(out) if out.status.success() => {
            let song = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if song.is_empty() {
                // Если ничего не играет, возвращаем JSON для скрытия или статуса
                Ok("{\"text\": \"nothing is here ⏸\"}".to_string())
            } else {
                // Получаем статус (играет/пауза) для иконки
                let status_output = Command::new("playerctl")
                    .arg("status")
                    .output();
                
                let icon = match status_output {
                    Ok(s) if String::from_utf8_lossy(&s.stdout).trim() == "Playing" => "🎵",
                    _ => "⏸",
                };

                // сначала убираем скобки, потом экранируем: незачем экранировать
                // то, что всё равно будет выброшено
                let song = strip_brackets(&song);
                let song = crate::util::escape_json(&song);
                Ok(format!(
                    "{{\"text\": \"{} {}\", \"tooltip\": \"{}\"}}",
                    icon, song, song
                ))
            }
        }
        _ => Ok("{\"text\": \"nothing is here ⏸\"}".to_string()),
    }
}



pub fn run() {
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        if let Ok(json) = read_music() {
            println!("{}", json);
        }
    }
}