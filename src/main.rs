use std::fs::File;
use std::io::{BufRead, BufReader};

fn read_memory() -> std::io::Result<(f64, f64)> {
    let target_word1 = "MemTotal";
    let mut memory_total: u64 = 0;
    let target_word2 = "MemAvailable";
    let mut memory_available: u64 = 0;
    let file = File::open("/proc/meminfo")?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
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
    let occupied_memory: f64 = ((memory_total - memory_available) as f64) / (1024.0 * 1024.0);
    let memory_total_f64 = memory_total as f64 / (1024.0 * 1024.0);
    Ok((occupied_memory, memory_total_f64))
}

fn read_cpu() -> std::io::Result<(u64, u64)> {
    let file = File::open("/proc/stat")?;
    let reader = BufReader::new(file);
    let proc_stat: Vec<u64>;
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

fn read_net(iface: &str) -> std::io::Result<(u64, u64)> {
    let file = File::open("/proc/net/dev")?;
    let reader = BufReader::new(file);
    let mut net_stats: Vec<u64> = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().starts_with(&format!("{}:", iface)) {
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
fn main() {
    let mode = std::env::args().nth(1);
    match mode.as_deref(){
        Some("net") => run_net(),
        Some("sys") => run_sys(),
        _=>run_sys()
    }
}