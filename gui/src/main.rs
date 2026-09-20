// FF Studio — giris noktasi (masaustu)
//
// Uygulamanin KODU src/lib.rs'te (UI + is mantigi + testler).
// Bu dosya yalnizca isletim sisteminin giris noktasini baglar: run_desktop().
//
// `windows_subsystem = "windows"` ONEMLI:
//   Bu satir olmazsa exe KONSOL alt sistemiyle derlenir ve Windows, GUI
//   penceresinin YANINDA bir de konsol (siyah cmd) penceresi acar —
//   "cift pencere" problemi tam olarak buydu.
#![windows_subsystem = "windows"]

fn main() -> eframe::Result<()> {
    ffstudio::run_desktop()
}
