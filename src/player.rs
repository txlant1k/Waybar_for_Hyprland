use std::process::Command;

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