//! FF Studio TUI — binary girisi.
//!
//! Mantik `ffstudio_tui` kutuphanesinde; burasi yalnizca acilis/kapanis,
//! komut satiri bayraklari ve cikis kodu.

use ffstudio_core::lang::Lang;

/// Surum + bulunan ffmpeg + config yolu (GUI'deki "Hakkinda" panelinin CLI karsiligi).
fn print_version() {
    println!("ffstudio TUI {}", env!("CARGO_PKG_VERSION"));
    match ffstudio_core::ffmpeg::Ffmpeg::locate(Lang::Tr) {
        Ok(f) => {
            let v = f
                .version_line()
                .trim_start_matches("ffmpeg version ")
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_string();
            println!("ffmpeg:  {} ({v})", f.ffmpeg.display());
        }
        Err(_) => println!("ffmpeg:  bulunamadi (Termux'ta: pkg install ffmpeg)"),
    }
    match ffstudio_tui::config::config_path() {
        Some(p) => println!("config:  {}", p.display()),
        None => println!("config:  (ev klasoru bulunamadi)"),
    }
}

fn print_help() {
    let tr = ffstudio_tui::config::Settings::load().lang() == Lang::Tr;
    if tr {
        println!("FF Studio TUI — Termux / Linux terminal istemcisi");
        println!();
        println!("KULLANIM:");
        println!("  ffstudio             arayuzu baslat");
        println!("  ffstudio --help      bu yardimi goster");
        println!("  ffstudio --version   surum + ffmpeg + config yolu");
        println!();
        println!("KISAYOLLAR (arayuz icinde):");
        println!("  a / d                dosya sec / klasor sec (recursive tarama)");
        println!("  Space                gezicide coklu secim (Enter: klasore gir)");
        println!("  p / c                preset sec / donusumu baslat");
        println!("  o / r / w / M        cikti klasoru / kaynak islemi / uzerine yaz / tasima klasoru");
        println!("  C / g / G / m        ozel arguman / kirpma baslangic / bitis / birlestirme adi");
        println!("  + -                  preset parametresi (CRF, bitrate, kalite...)");
        println!("  Tab                  panel degistir   t: tema   l: dil   s: ayarlar");
        println!("  q                    cikis (onay ister; Ctrl+C de temiz cikar)");
        println!();
        println!("ORTAM DEGISKENLERI:");
        println!("  NO_COLOR=1           renkleri kapatir (vurgular ters-video ile)");
        println!("  XDG_CONFIG_HOME      ayar dosyasi kok klasoru (varsayilan ~/.config)");
        println!();
        println!("KURULUM: ./build.sh  (derler + 'ffstudio' kisayolunu PATH'e kurar)");
        println!("Ayar dosyasi: ~/.config/ffstudio-tui/config.json");
    } else {
        println!("FF Studio TUI — Termux / Linux terminal client");
        println!();
        println!("USAGE:");
        println!("  ffstudio             start the interface");
        println!("  ffstudio --help      show this help");
        println!("  ffstudio --version   version + ffmpeg + config path");
        println!();
        println!("SHORTCUTS (inside the interface):");
        println!("  a / d                pick files / pick folder (recursive scan)");
        println!("  Space                multi-select in picker (Enter: enter folder)");
        println!("  p / c                choose preset / start conversion");
        println!("  o / r / w / M        output folder / source action / overwrite / move folder");
        println!("  C / g / G / m        custom args / trim start / end / merge name");
        println!("  + -                  preset parameter (CRF, bitrate, quality...)");
        println!("  Tab                  switch panel   t: theme   l: language   s: settings");
        println!("  q                    quit (asks; Ctrl+C also exits cleanly)");
        println!();
        println!("ENVIRONMENT:");
        println!("  NO_COLOR=1           disable colors (highlights use reverse video)");
        println!("  XDG_CONFIG_HOME      config root (default ~/.config)");
        println!();
        println!("INSTALL: ./build.sh  (builds + installs the 'ffstudio' shortcut into PATH)");
        println!("Config:  ~/.config/ffstudio-tui/config.json");
    }
}

fn main() {
    // Komut satiri: --help / --version terminale dokunmadan cikar
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(|s| s.as_str()) {
        Some("--help") | Some("-h") | Some("help") => {
            print_help();
            return;
        }
        Some("--version") | Some("-V") | Some("version") => {
            print_version();
            return;
        }
        Some(other) if !other.is_empty() => {
            eprintln!("Bilinmeyen arguman: {other}");
            eprintln!("Kullanim: ffstudio [--help | --version]");
            std::process::exit(2);
        }
        _ => {}
    }

    // Termux: islem surerken telefon uyumasin (pkg install termux-api gerekir;
    // yoksa komut sessizce basarisiz olur, uygulama yine calisir).
    ffstudio_tui::termux("termux-wake-lock");

    let kesildi = ffstudio_tui::run_tui();
    if let Ok(true) = kesildi {
        // terminal geri yuklendi; bilgi satiri shell'e yazilabilir
        let tr = ffstudio_tui::config::Settings::load().lang() == Lang::Tr;
        if tr {
            println!("Ctrl+C: islem durduruldu, terminal geri yuklendi.");
        } else {
            println!("Ctrl+C: interrupted, terminal restored.");
        }
    }
    let code = match kesildi {
        Ok(false) => 0,
        // Ctrl+C: standart SIGINT cikis kodu; terminal geri yuklendi,
        // ffmpeg cocuklari durduruldu.
        Ok(true) => 130,
        Err(e) => {
            eprintln!("FF Studio TUI hatasi: {e}");
            1
        }
    };

    ffstudio_tui::termux("termux-wake-unlock");
    std::process::exit(code);
}
