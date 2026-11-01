//! wired — модули для Waybar в стиле Lain.
//!
//! Режимы запуска:
//! - `wired sys` — загрузка процессора и занятая память;
//! - `wired net` — имя Wi-Fi сети и скорость загрузки/отдачи.

mod sys;
mod net;
mod bat;
mod player;
mod util;
fn run_sys_or_net(){
const INTERVAL_SECS: f64 = 1.0;
loop{
    let (mut prev_total, mut prev_idle) = sys::read_cpu().unwrap();
    for i in 0..5 {
        std::thread::sleep(std::time::Duration::from_secs(1));
        // процессор и память
        let (occupied_memory, total_memory) = sys::read_memory().unwrap();
        let (total, idle) = sys::read_cpu().unwrap();

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
            "{{\"text\": \"cpu>>·{2:.0}% · memry>>·{0:.1}/{1:.1} gB\", \"tooltip\": \"Процессор: {2:.0}%\\nПамять: занято {0:.1} GB из {1:.1} GB\"}}",
            occupied_memory, total_memory, percentage_cpu
        );
        println!("{}", result);
    }
    
   let name = match net::find_wifi() {
        Some(n) => n,
        None => {
            continue;
        }
    };
    let (mut prev_rx, mut prev_tx) = net::read_net(&name).unwrap();
    for i in 0..5 {
        std::thread::sleep(std::time::Duration::from_secs(1));
        // сеть
        let (rx, tx) = net::read_net(&name).unwrap();
        let download_kb = rx.saturating_sub(prev_rx) as f64 / INTERVAL_SECS / 1024.0;
        let upload_kb = tx.saturating_sub(prev_tx) as f64 / INTERVAL_SECS / 1024.0;

        prev_rx = rx;
        prev_tx = tx;
        let raw_name = net::read_ssid(&name);
        // сначала обрезаем до 7 символов, потом экранируем: иначе обрезка
        // может разрезать пару \" пополам и оставить в конце одинокий \
        let short_name: String = raw_name.chars().take(7).collect();
        let short_name = util::escape_json(&short_name);
        let full_name = util::escape_json(&raw_name);

        let result = format!(
            "{{\"text\": \"wired>> {0} ↓ {2:.1} ↑ {3:.1} kB/s\", \"tooltip\": \"Сеть: {1}\\nЗагрузка: {2:.1} KB/s\\nОтдача: {3:.1} KB/s\"}}",
            short_name, full_name, download_kb, upload_kb
        );
        println!("{}", result);
    }
    }
}


fn main() {
    let mode = std::env::args().nth(1);
    match mode.as_deref(){
        Some("sys_net") => run_sys_or_net(),
        Some("bat") => bat::run(),
        Some("player") => player::run(),
        _=>run_sys_or_net()
    }
}