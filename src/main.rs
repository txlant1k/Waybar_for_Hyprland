//! wired — модули для Waybar в стиле Lain.
//!
//! Режимы запуска:
//! - `wired sys` — загрузка процессора и занятая память;
//! - `wired net` — имя Wi-Fi сети и скорость загрузки/отдачи.

mod sys;
mod net;
mod bat;
fn run_sys_or_net(){
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
            println!("{{\"text\": \"\"}}");
            return;
        }
    };
    let (mut prev_rx, mut prev_tx) = net::read_net(&name).unwrap();
    for i in 0..5 {
        std::thread::sleep(std::time::Duration::from_secs(1));
        // сеть
        let (rx, tx) = net::read_net(&name).unwrap();
        let download_kb = rx.saturating_sub(prev_rx) as f64 / 2.0 / 1024.0;
        let upload_kb = tx.saturating_sub(prev_tx) as f64 / 2.0 / 1024.0;

        prev_rx = rx;
        prev_tx = tx;
        let net_name = net::read_ssid(&name);

        let result = format!(
            "{{\"text\": \"wired>>{0:.6} ↓ {1:.1} ↑ {2:.1} kB/s\", \"tooltip\": \"Сеть: {0}\\nЗагрузка: {1:.1} KB/s\\nОтдача: {2:.1} KB/s\"}}",
            net_name, download_kb, upload_kb
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
        _=>run_sys_or_net()
    }
}