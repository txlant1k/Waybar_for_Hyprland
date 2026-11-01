#!/bin/bash
set -e
# переходим в папку, где лежит сам скрипт
cd "$(dirname "$0")"
cargo build --release
killall waybar 2>/dev/null || true
pkill -x wired 2>/dev/null || true
mkdir -p ~/.local/bin
cp target/release/wired ~/.local/bin/
waybar > /dev/null 2>&1 & disown
echo "готово"