//! Режим `bat`: заряд батареи и уведомления о низком заряде.

/// Порог первого предупреждения, в процентах.
const WARN_LEVEL: u8 = 20;
/// Порог критического предупреждения, в процентах.
const CRITICAL_LEVEL: u8 = 10;

/// Читает заряд и статус батареи `name` из `/sys/class/power_supply/`.
///
/// Возвращает пару `(процент, статус)`, где статус — например
/// `Charging`, `Discharging` или `Full`.
///
/// # Errors
///
/// Возвращает ошибку, если файлы `capacity` или `status` не удалось прочитать.
fn read_battery(name: &str) -> std::io::Result<(u8, String)> {
    let filename1 = format!("/sys/class/power_supply/{}/capacity", name);
    let filename2 = format!("/sys/class/power_supply/{}/status", name);

    let capacity = std::fs::read_to_string(filename1)?;
    let percentage: u8 = capacity.trim().parse().unwrap_or(0);

    let status = std::fs::read_to_string(filename2)?.trim().to_string();
    Ok((percentage, status))
}

/// Отправляет уведомление через `notify-send`.
///
/// Ошибки игнорируются: если уведомление не ушло, модуль всё равно
/// должен продолжать работать.
fn notify(urgency: &str, title: &str, body: &str) {
    let _ = std::process::Command::new("notify-send")
        .args(["-u", urgency, title, body])
        .status();
}

/// Ищет батарею среди устройств в `/sys/class/power_supply/`.
///
/// Возвращает имя первого устройства, у которого в файле `type`
/// записано `Battery`, или `None`, если батареи нет.
fn find_battery() -> Option<String> {
    let dirs = std::fs::read_dir("/sys/class/power_supply/").ok()?;
    for dir in dirs {
        let dir = match dir {
            Ok(e) => e,
            Err(_) => continue,
        };
        // в каждой папке устройства лежит файл type
        let type_path = dir.path().join("type");
        if let Ok(kind) = std::fs::read_to_string(type_path) {
            if kind.trim() == "Battery" {
                return dir.file_name().into_string().ok();
            }
        }
    }
    None
}

/// Бесконечный цикл: раз в 10 секунд печатает JSON для Waybar
/// и присылает уведомления при низком заряде.
///
/// Если батареи нет, печатает пустой текст (Waybar скрывает модуль) и завершается.
pub fn run() {
    let name = match find_battery() {
        Some(n) => n,
        None => {
            println!("{{\"text\": \"\"}}");
            return;
        }
    };

    // флаги живут между итерациями, чтобы уведомлять только один раз
    let mut ten_notify = false;
    let mut twenty_notify = false;

    loop {
        let (percentage, status) = read_battery(&name).unwrap();

        // сначала критический порог: если заряд сразу упал ниже 10%,
        // уведомление на 20% уже не нужно
        if percentage <= CRITICAL_LEVEL && status == "Discharging" && ten_notify == false {
            ten_notify = true;
            twenty_notify = true;
            let title = format!("Батарея {}%", percentage);
            notify("critical", &title, "Срочно подключите зарядку пжпжпжпж");
        }
        if percentage <= WARN_LEVEL && status == "Discharging" && twenty_notify == false{
            twenty_notify = true;
            let title = format!("Батарея {}%", percentage);
            notify("critical", &title, "Подключите зарядку пжпжпжпж");
        }
        // при зарядке сбрасываем флаги, чтобы после отключения уведомления сработали снова
        if status == "Charging" {
            ten_notify = false;
            twenty_notify = false;
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
        println!("{}", s);
        std::thread::sleep(std::time::Duration::from_secs(10));
    }
}