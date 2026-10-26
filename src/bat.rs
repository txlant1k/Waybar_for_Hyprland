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



pub fn run(){
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
