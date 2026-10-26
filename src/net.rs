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
fn read_net(iface: &str) -> std::io::Result<(u64, u64)> {
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
fn read_ssid(iface: &str) -> String{
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
pub fn run(){
    let (mut prev_rx, mut prev_tx) = read_net("wlp5s0").unwrap();
    loop {
        std::thread::sleep(std::time::Duration::from_secs(2));
        // сеть
        let (rx, tx) = read_net("wlp5s0").unwrap();
        let download_kb = rx.saturating_sub(prev_rx) as f64 / 2.0 / 1024.0;
        let upload_kb = tx.saturating_sub(prev_tx) as f64 / 2.0 / 1024.0;

        prev_rx = rx;
        prev_tx = tx;
        let net_name = read_ssid("wlp5s0");

        let result = format!(
            "{{\"text\": \"{0} ↓ {1:.1} ↑ {2:.1} KB/s\", \"tooltip\": \"Сеть: {0}\\nЗагрузка: {1:.1} KB/s\\nОтдача: {2:.1} KB/s\"}}",
            net_name, download_kb, upload_kb
        );
        println!("{}", result);
    }
}