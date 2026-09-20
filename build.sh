#!/usr/bin/env bash
set -e

# cargo yoksa: rustup.rs uyarısı ver (build.bat'taki davranışın birebir aynısı)
if ! command -v cargo &> /dev/null; then
    echo "cargo bulunamadı. Termux'ta şunu çalıştır: pkg install rust"
    echo "Diğer Linux dağıtımları için: https://rustup.rs"
    exit 1
fi

# ffmpeg PATH kontrolü — TUI'de embedded ffmpeg YOK, sadece PATH'ten bulunur
if ! command -v ffmpeg &> /dev/null; then
    echo "UYARI: ffmpeg PATH'te bulunamadı."
    echo "Termux'ta: pkg install ffmpeg"
    echo "Build yine de devam edecek, ama uygulama çalışırken ffmpeg gerekecek."
fi

echo "TUI derleniyor (release)..."
cargo build --release -p tui

echo ""
echo "Derleme tamamlandı."
echo "Binary: target/release/tui"

# build.bat run modunun karşılığı: ./build.sh run
if [ "$1" == "run" ]; then
    echo "Çalıştırılıyor..."
    ./target/release/tui
fi
