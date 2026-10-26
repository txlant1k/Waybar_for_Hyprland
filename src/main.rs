//! wired — модули для Waybar в стиле Lain.
//!
//! Режимы запуска:
//! - `wired sys` — загрузка процессора и занятая память;
//! - `wired net` — имя Wi-Fi сети и скорость загрузки/отдачи.
use std::fs::File;
use std::io::{BufRead, BufReader};
/// Читает `/proc/meminfo` и возвращает занятую и общую память в гигабайтах.
///
/// Занятая память считается как `MemTotal - MemAvailable`.
///
/// # Errors
///
/// Возвращает ошибку, если файл не удалось открыть или прочитать.
fn read_memory() -> std::io::Result<(f64, f64)> {
    let target_word1 = "MemTotal";
    let mut memory_total: u64 = 0;
    let target_word2 = "MemAvailable";
    let mut memory_available: u64 = 0;
    // Открываем файл и создаем объект BufReader для построчного чтения
    let file = File::open("/proc/meminfo")?;
    let reader = BufReader::new(file);
    // Цикл проходит весь файл и ищет строки, наинающиеся с заданных слов target_word1 
    // и target_word2 соответственно
    for line in reader.lines() {
        // проверка на возможность прочитать строку, в противном случае возвращаем ошибку
        let line = line?;
        if line.starts_with(target_word1) {
            if let Ok(value) = line.split_whitespace().nth(1).unwrap().parse() {
                memory_total = value;
            }
        }
        if line.starts_with(target_word2) {
            if let Ok(value) = line.split_whitespace().nth(1).unwrap().parse() {
                memory_available = value;
            }
        }
    }
    // находим количество занятой памяти и переводим его в гигабайты вместе с общим количеством оной
    let occupied_memory: f64 = ((memory_total - memory_available) as f64) / (1024.0 * 1024.0);
    let memory_total_f64 = memory_total as f64 / (1024.0 * 1024.0);
    // возвращаем пару значений (используется, всего)
    Ok((occupied_memory, memory_total_f64))
}
/// Читает первую строку `/proc/stat` и возвращает счётчики процессора.
///
/// Возвращает пару `(всего, простой)`, где «простой» — это сумма полей
/// `idle` и `iowait`. Счётчики растут с момента загрузки системы, поэтому
/// загрузку нужно считать по разнице двух замеров.
///
/// # Errors
///
/// Возвращает ошибку, если файл `/proc/stat` не удалось открыть или прочитать
fn read_cpu() -> std::io::Result<(u64, u64)> {
    let file = File::open("/proc/stat")?;
    let reader = BufReader::new(file);
    let proc_stat: Vec<u64>; // вектор для хранения счетчиков процессора
    if let Some(first_line) = reader.lines().next() {
        let line = first_line?;
        proc_stat = line.split_whitespace().skip(1).map(|s| s.parse().unwrap()).collect();
    } else {
        return Ok((0, 0));
    }
    let simple = proc_stat[3] + proc_stat[4];
    let all = proc_stat.into_iter().sum();
    Ok((all, simple))
}
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
fn read_battery(name: &str) -> std::io::Result<(u8, String)>{
    let filename1 = format!("/sys/class/power_supply/{}/capacity", name);
    let filename2 = format!("/sys/class/power_supply/{}/status", name);

    let capacity = std::fs::read_to_string(filename1)?;
    let percentage:u8 = capacity.trim().parse().unwrap_or(0);

    let status = std::fs::read_to_string(filename2)?.trim().to_string();
    Ok((percentage, status))


}
fn notify(urgency: &str, title: &str, body: &str){
    let _ = std::process::Command::new("notify-send").args(["-u", urgency, title, body]).status();
    
}
fn run_sys(){
    let (mut prev_total, mut prev_idle) = read_cpu().unwrap();
    loop {
        std::thread::sleep(std::time::Duration::from_secs(2));
        // процессор и память
        let (occupied_memory, total_memory) = read_memory().unwrap();
        let (total, idle) = read_cpu().unwrap();

        let d_total = total - prev_total;
        let d_idle = idle - prev_idle;

        let percentage_cpu = if d_total > 0 {
            (d_total - d_idle) as f64 / d_total as f64 * 100.0
        } else {
            0.0
        };

        prev_total = total;
        prev_idle = idle;
        let result = format!(
            "{{\"text\": \"CPU {2:.0}% · RAM {0:.1}/{1:.1}\", \"tooltip\": \"Процессор: {2:.0}%\\nПамять: занято {0:.1} GB из {1:.1} GB\"}}",
            occupied_memory, total_memory, percentage_cpu
        );
        println!("{}", result);
    }
}
fn run_net(){
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

fn run_bat(){
    let mut ten_notify: bool = false;
    let mut twenty_notify: bool = false;
    loop{
        
        let (percentage, status) = read_battery("BAT1").unwrap();
        if percentage <=10 && status.as_str() == "Discharging" && ten_notify == false{
            ten_notify = true;
            twenty_notify = true;
            notify("critical", "Батарея >=10%", "Срочно подключите зарядку пжпжпжпж");
        }
        if percentage <=20 && status.as_str() == "Discharging" && twenty_notify == false {
            twenty_notify = true;
            notify("critical", "Батарея >=20%", "Подключите зарядку пжпжпжпж");
        }
        let (label, status_ru) = match status.as_str() {
            "Charging" => ("CHG", "Заряжается"),
            "Discharging" => ("BAT", "Разряжается"),
            "Full" => ("BAT", "Заряжена"),
            _ => ("BAT", "Не заряжается"),
        };

        let s = format!(
            "{{\"text\": \"{0} {1}%\", \"tooltip\": \"{2}: {1}%\"}}",
            label, percentage, status_ru
        );
        if status.as_str() == "Charging"{
            ten_notify = false;
            twenty_notify = false;
        }
        println!("{}", s);
        std::thread::sleep(std::time::Duration::from_secs(10));
    }
}

fn main() {
    let mode = std::env::args().nth(1);
    match mode.as_deref(){
        Some("net") => run_net(),
        Some("sys") => run_sys(),
        Some("bat") => run_bat(),
        _=>run_sys()
    }
}