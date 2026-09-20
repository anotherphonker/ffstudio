// FF Studio — giris noktalari
//
// Uygulamanin KODU BOLUMU src/lib.rs'te (ayni kod Android cdylib'ine de
// giriyor). Bu dosya yalnizca isletim-sistemi giris noktasini baglar:
//   - Masaustu: run_desktop()
//   - Android:  .so icindeki `android_main` (lib.rs) — NativeActivity
//               manifest'teki `android.app.lib_name` = "ffstudio" ile girer.

#[cfg(not(target_os = "android"))]
fn main() -> eframe::Result<()> {
    ffstudio::run_desktop()
}

#[cfg(target_os = "android")]
fn main() {
    // Android'de bu binary calistirilmaz; giris noktası libffstudio.so
    // icindeki android_main. Bu stub yalnizca crate'in derlenebilmesi icin.
    std::process::exit(1);
}
