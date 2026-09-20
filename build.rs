use std::path::Path;

fn main() {
    println!("cargo:rustc-check-cfg=cfg(embed_ffmpeg)");
    // Embed YALNIZCA masaustu icin: ffmpeg.zip icindeki binary WINDOWS
    // x64. Android hedefinde ffmpeg gozulemez (Faz 3'te NDK ile derlenen
    // ARM ffmpeg + Rust FFI gelecek).
    let is_android = std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("android");
    if is_android {
        println!("cargo:warning=Android hedefi: ffmpeg.zip gozulmez (Faz 3: NDK ffmpeg).");
        return;
    }
    // Proje kokunde "ffmpeg.zip" (gyan.dev essentials zip'i) varsa,
    // binary icine gomulur -> all-in-one. Yoksa sistem ffmpeg kullanilir.
    let p = Path::new("ffmpeg.zip");
    if p.is_file() {
        println!("cargo:rustc-cfg=embed_ffmpeg");
        println!("cargo:warning=ffmpeg.zip bulundu: binary icine gomulecek (all-in-one).");
    } else {
        println!("cargo:warning=ffmpeg.zip yok: binary, PATH'teki veya exe yanindaki ffmpeg'i kullanacak.");
    }
    println!("cargo:rerun-if-changed=ffmpeg.zip");
}
