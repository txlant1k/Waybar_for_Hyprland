#!/bin/bash
set -e
cd ~/project/wired
cargo build --release
killall waybar 2>/dev/null || true
pkill -x wired 2>/dev/null || true
cp target/release/wired ~/.local/bin/
waybar > /dev/null 2>&1 & disown
echo "готово"
