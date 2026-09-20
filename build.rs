use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rustc-check-cfg=cfg(embed_ffmpeg)");
    println!("cargo:rerun-if-changed=build.rs");

    // -----------------------------------------------------------------
    // 1) ffmpeg.zip gomme (all-in-one)
    //    Proje kokunde "ffmpeg.zip" (gyan.dev essentials zip'i) varsa
    //    binary icine gomulur -> tek exe, ffmpeg kurulumu GEREKMEZ.
    // -----------------------------------------------------------------
    let p = Path::new("ffmpeg.zip");
    if p.is_file() {
        println!("cargo:rustc-cfg=embed_ffmpeg");
        println!("cargo:warning=ffmpeg.zip bulundu: binary icine gomulecek (all-in-one).");
    } else {
        println!("cargo:warning=ffmpeg.zip yok: binary, PATH'teki veya exe yanindaki ffmpeg'i kullanacak.");
    }
    println!("cargo:rerun-if-changed=ffmpeg.zip");

    // -----------------------------------------------------------------
    // 2) Windows exe ikonu (Gezgin/taskbar) + surum bilgisi
    //
    //    Pencere ikonu lib.rs'te logo.png'den geliyor; ama Windows'un
    //    DOSYA ikonu (Gezgin'de, taskbar'da, kisayolda) exe'nin icindeki
    //    kaynak bolumunden okunur. Burada assets/icon.ico'yu gomuyoruz.
    //
    //    Onemli: rc.exe / windres bulunamazsa BUILD KIRILMAZ, sadece
    //    uyari verir (uygulama yine derlenir, sadece ikon gomulmez).
    // -----------------------------------------------------------------
    embed_windows_icon();
}

fn embed_windows_icon() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return; // Windows hedefi degilse kaynak gomme yok
    }
    let ico = PathBuf::from("assets/icon.ico");
    println!("cargo:rerun-if-changed=assets/icon.ico");
    if !ico.is_file() {
        println!("cargo:warning=assets/icon.ico yok: exe ikonu gomulemedi.");
        return;
    }

    // winresource: MSVC icin rc.exe, GNU icin windres kullanir.
    // Panik ederse (arac yoksa) yakalayip uyariya ceviriyoruz.
    let res = std::panic::catch_unwind(move || {
        let mut w = winresource::WindowsResource::new();
        w.set_icon("assets/icon.ico");
        w.set("ProductName", "FF Studio");
        w.set("FileDescription", "FF Studio - all-in-one ffmpeg GUI");
        w.set("CompanyName", "Resul Celik");
        w.set("LegalCopyright", "Resul Celik");
        w.set("OriginalFilename", "ffstudio.exe");
        w.set("FileVersion", "0.1.0.0");
        w.set("ProductVersion", "0.1.0.0");
        w.compile()
    });

    match res {
        Ok(Ok(())) => println!("cargo:warning=Logo exe'ye gomuldu: pencere + Gezgin/taskbar ikonu."),
        Ok(Err(e)) => println!("cargo:warning=exe ikonu gomulemedi: {e}"),
        Err(_) => println!(
            "cargo:warning=exe ikonu gomulemedi (rc.exe/windres bulunamadi). Windows SDK kuruluysa tekrar dene."
        ),
    }
}
