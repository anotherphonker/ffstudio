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

# PATH kısayolu: binary'yi PATH'teki bir yere kopyalar (Termux ve Linux'ta hedef farklı)
install_shortcut() {
    # NOT: "$PREFIX" bos ise "$PREFIX/bin" = "/bin" olur ve her Linux'ta
    # Termux dalina girilirdi; once PREFIX'in dolu oldugu kontrol edilir.
    if [ -n "$TERMUX_VERSION" ] || { [ -n "$PREFIX" ] && [ -d "$PREFIX/bin" ]; }; then
        # Termux
        TARGET="$PREFIX/bin/ffstudio"
    else
        # Genel Linux — ~/.local/bin, çoğu dağıtımda zaten PATH'te
        mkdir -p "$HOME/.local/bin"
        TARGET="$HOME/.local/bin/ffstudio"
    fi

    cp target/release/tui "$TARGET"
    chmod +x "$TARGET"
    echo "Kısayol kuruldu: '$TARGET'"
    echo "Artık her yerden 'ffstudio' yazıp Enter'layarak açabilirsin."

    # ~/.local/bin PATH'te değilse uyar (Termux'ta $PREFIX/bin zaten PATH'te, sorun yok)
    if [ "$TARGET" == "$HOME/.local/bin/ffstudio" ] && [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
        echo "UYARI: ~/.local/bin PATH'te değil. Shell config dosyana ekle:"
        echo '  export PATH="$HOME/.local/bin:$PATH"'
    fi

    # Termux:Widget (opsiyonel): ~/.shortcuts/ffstudio -> ana ekranda tek dokunuş ikonu
    if [ -n "$TERMUX_VERSION" ] || { [ -n "$PREFIX" ] && [ -d "$PREFIX/bin" ]; }; then
        mkdir -p "$HOME/.shortcuts"
        printf '#!/data/data/com.termux/files/usr/bin/sh\nexec ffstudio\n' > "$HOME/.shortcuts/ffstudio"
        chmod +x "$HOME/.shortcuts/ffstudio"
        echo "Termux:Widget kısayolu: '$HOME/.shortcuts/ffstudio'"
        echo "  (Termux:Widget uygulaması kuruluysa ana ekrana ikon olarak ekleyebilirsin)"
    fi
}

# Termux: depolama izni verilmemisse gezici /sdcard'i goremez -> simdi iste
# NOT: termux-setup-storage, kullanici izin vermezse hata kodu ile donebilir.
# set -e yuzunden betik yarida kesilmesin (kisayol yine kurulsun) diye bu blok
# boyunca set -e kapatilir; asagidaki satirlar aynen korunur.
set +e
if [ -n "$TERMUX_VERSION" ] && [ ! -d "$HOME/storage/shared" ]; then
    echo "Depolama izni henüz verilmemiş."
    echo "Şimdi izin isteği açılacak, ekrandaki dialogda 'İzin ver'e bas."
    termux-setup-storage
    # Android izin dialogu asenkron açılıyor, sembolik link oluşana kadar bekle
    for i in $(seq 1 10); do
        [ -d "$HOME/storage/shared" ] && break
        sleep 1
    done
    if [ ! -d "$HOME/storage/shared" ]; then
        echo "UYARI: İzin verilmedi ya da zaman aşımı. Manuel çalıştır: termux-setup-storage"
    fi
fi
set -e

install_shortcut

# build.bat run modunun karşılığı: ./build.sh run
if [ "$1" == "run" ]; then
    echo "Çalıştırılıyor..."
    ./target/release/tui
fi
