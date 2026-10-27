use std::fs::File;
use std::io::{BufRead, BufReader};
/// Читает файл /proc/net/dev/ и возвращает счетчики байт 
/// 
/// Возвращает счетчики байт (получено, отправлено ) с момента подключения интерфейса
/// Скорость считается в вызывающем коде по разнице двух замеров
/// 
/// # Errors 
/// 
/// Возвращает ошибку, если вышеупомянутый файл не удалось открыть или прочитать
pub fn read_net(iface: &str) -> std::io::Result<(u64, u64)> {
    let file = File::open("/proc/net/dev")?;
    let reader = BufReader::new(file);
    let mut net_stats: Vec<u64> = Vec::new();
    for line in reader.lines() {
        let line = line?;
        // проверка начинается ли строка с определенного слова (интерфейса сети)
        if line.trim().starts_with(&format!("{}:", iface)) {
            // при большом трафике числа прилипают к имени интерфейса без пробела, поэтому нужно сделать сплит по двоеточи
            net_stats = line.split(':').nth(1).unwrap().split_whitespace().map(|s| s.parse().unwrap()).collect();
        }
    }
    if net_stats.is_empty() {
        return Ok((0, 0));
    }
    let got = net_stats[0];
    let sent = net_stats[8];
    Ok((got, sent))
}
pub fn read_ssid(iface: &str) -> String{
    let output = std::process::Command::new("nmcli")
    .args(["-t", "-g", "GENERAL.CONNECTION", "device", "show", iface])
    .output();
    match output{
        Ok(result) => { 
        let name = String::from_utf8_lossy(&result.stdout).trim().to_string();
            if name.is_empty() {
                String::from("Нет сети :(")
            } else {
                name
            }
        }
        Err(_) => return String::from("Нет сети :(")
        }
}
pub fn find_wifi() -> Option<String> {
    let dirs = std::fs::read_dir("/sys/class/net/").ok()?;
    for entry in dirs {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        if entry.path().join("wireless").exists() {
            return entry.file_name().into_string().ok();
        }
    }
    None
}
    


