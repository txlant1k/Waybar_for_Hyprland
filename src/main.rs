//! wired — модули для Waybar в стиле Lain.
//!
//! Режимы запуска:
//! - `wired sys` — загрузка процессора и занятая память;
//! - `wired net` — имя Wi-Fi сети и скорость загрузки/отдачи.

mod sys;
mod net;
mod bat;


fn main() {
    let mode = std::env::args().nth(1);
    match mode.as_deref(){
        Some("net") => net::run(),
        Some("sys") => sys::run(),
        Some("bat") => bat::run(),
        _=>sys::run()
    }
}