//! FF Studio TUI — binary girisi.
//!
//! Mantik `ffstudio_tui` kutuphanesinde; burasi yalnizca acilis/kapanis.

fn main() {
    // Termux: islem surerken telefon uyumasin (pkg install termux-api gerekir;
    // yoksa komut sessizce basarisiz olur, uygulama yine calisir).
    ffstudio_tui::termux("termux-wake-lock");

    let code = match ffstudio_tui::run_tui() {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("FF Studio TUI hatasi: {e}");
            1
        }
    };

    ffstudio_tui::termux("termux-wake-unlock");
    std::process::exit(code);
}
