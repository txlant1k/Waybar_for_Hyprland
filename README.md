# wired

Модули для [Waybar](https://github.com/Alexays/Waybar) на Rust в стиле Serial Experiments Lain.

![панель](screenshot.jpg)

## Возможности

- **sys_net** — загрузка процессора и занятая память, раз в 5 секунд сменяются именем Wi-Fi сети и скоростью загрузки/отдачи
- **bat** — заряд батареи и уведомления при 20% и 10%; если батареи нет, модуль скрывается
- **player** — текущий трек через `playerctl`, без лишнего в скобках (`feat.`, `Live` и т.п.)

Данные читаются напрямую из `/proc` и `/sys`, Wi-Fi интерфейс и батарея находятся автоматически.

## Зависимости

- Rust (cargo)
- Waybar
- NetworkManager (`nmcli`) — имя сети
- `playerctl` — плеер
- `libnotify` (`notify-send`) — уведомления о батарее

На Arch:

```bash
sudo pacman -S rust waybar networkmanager playerctl libnotify
```

## Установка

```bash
git clone https://github.com/txlant1k/Waybar_for_Hyprland.git
cd Waybar_for_Hyprland
./deploy.sh
```

Скрипт собирает программу, копирует её в `~/.local/bin/` и перезапускает Waybar.

## Подключение к Waybar

Добавьте модули в `~/.config/waybar/config.jsonc`:

```jsonc
"custom/sys_net": {
    "exec": "~/.local/bin/wired sys_net",
    "return-type": "json"
},
"custom/bat": {
    "exec": "~/.local/bin/wired bat",
    "return-type": "json"
},
"custom/player": {
    "exec": "~/.local/bin/wired player",
    "return-type": "json"
}
```

и их имена в `modules-left` / `modules-right`. Полный пример конфига и стилей — в папке [`examples/`](examples/).

## Тесты

```bash
cargo test
```