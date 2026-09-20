use std::path::Path;

fn main() {
    println!("cargo:rustc-check-cfg=cfg(embed_ffmpeg)");
    println!("cargo:rerun-if-changed=build.rs");

    // ffmpeg.zip WORKSPACE KOKUNDE durur (core/ degil): core/src/ffmpeg.rs
    // icindeki include_bytes!("../ffmpeg.zip") oraya bakar.
    let p = Path::new("../ffmpeg.zip");
    if p.is_file() {
        println!("cargo:rustc-cfg=embed_ffmpeg");
        println!("cargo:warning=ffmpeg.zip bulundu: binary icine gomulecek (all-in-one).");
    } else {
        println!("cargo:warning=ffmpeg.zip yok: binary, PATH'teki veya exe yanindaki ffmpeg'i kullanacak.");
    }
    println!("cargo:rerun-if-changed=../ffmpeg.zip");
}
