use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../ffmpeg.zip");

    // "embed-ffmpeg" feature'i ACIK MI? (build script'lere CARGO_FEATURE_<AD>
    // seklinde gelir; tire -> alt cizgi.)
    let embed = std::env::var_os("CARGO_FEATURE_EMBED_FFMPEG").is_some();

    // ffmpeg.zip WORKSPACE KOKUNDE durur. Bu script crate koku (core/) icinden
    // calisir -> "../ffmpeg.zip"; core/src/ffmpeg.rs icindeki
    // include_bytes!("../../ffmpeg.zip") de AYNI dosyayi gosterir.
    let zip_var = Path::new("../ffmpeg.zip").is_file();

    match (embed, zip_var) {
        (true, true) => println!(
            "cargo:warning=ffmpeg.zip bulundu: ffmpeg binary icine gomulecek (all-in-one GUI)."
        ),
        (true, false) => println!(
            "cargo:warning=embed-ffmpeg acik ama ffmpeg.zip yok! ffmpeg.zip'i repo KOKUNE koy \
             (aksi halde include_bytes! derlemesi hata verir). Sadece TUI derliyorsan sorun yok: \
             TUI bu feature'i acmaz."
        ),
        (false, true) => println!(
            "cargo:warning=ffmpeg.zip var ama 'embed-ffmpeg' kapali: gomulmeyecek \
             (TUI her zaman boyle: ffmpeg'i PATH'ten bulur)."
        ),
        (false, false) => println!(
            "cargo:warning=ffmpeg.zip yok, gomme yok: ffmpeg PATH'ten / exe yanindan bulunacak."
        ),
    }
}
