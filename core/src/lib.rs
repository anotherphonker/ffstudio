//! FF Studio cekirdegi: platformdan BAGIMSIZ is mantigi.
//!
//! Iki frontend ayni kodu kullanir:
//!   - `gui`  : masaustu (Windows) egui/eframe uygulamasi
//!   - `tui`  : Termux (Android/Linux) ratatui istemcisi
//!
//! Burada UI kodu YOKTUR; sadece ffmpeg cagrilari, preset/arguman uretimi,
//! CPU/paralellik plani, ceviriler ve dosya yardimcilari bulunur.

pub mod cpu;
pub mod ffmpeg;
pub mod lang;
pub mod profiles;
pub mod util;
