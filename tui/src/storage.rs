//! Termux depolama sagligi.
//!
//! Termux'ta `/sdcard` (paylasilan depolama) ancak `termux-setup-storage`
//! calistirilip izin verildikten sonra erisilebilir olur; o zamana kadar
//! `~/storage/shared` YOKTUR. Bu modul:
//!
//!   - Termux ortaminda miyiz (masaustu Linux'ta bu kontrol YAPILMAZ)
//!   - `~/storage/shared` var mi (depolama izni verilmis mi)
//!   - gezicinin varsayilan baslangic klasoru (Music -> shared -> home)
//!   - izin yoksa kullaniciya gosterilecek NET Turkce mesaj
//!
//! Ham OS hatalari ("Permission denied", "os error 13") kullaniciya asla
//! ciplak gosterilmez; cevirisi `crate::errors` icindedir.

use std::path::PathBuf;

use ffstudio_core::lang::Lang;

/// Termux (Android) ortaminda mi calisiyoruz?
///
/// Masaustu Linux/Windows'ta bu modulun tum kontrolleri devre disi kalir.
pub fn is_termux() -> bool {
    if std::env::var_os("TERMUX_VERSION").is_some() {
        return true;
    }
    if let Some(p) = std::env::var_os("PREFIX") {
        if std::path::Path::new(&p).join("bin").is_dir() {
            return true;
        }
    }
    std::path::Path::new("/data/data/com.termux/files/usr").is_dir()
}

/// `$HOME/storage/shared` — paylasilan depolama koku (/storage/emulated/0).
pub fn shared_dir() -> Option<PathBuf> {
    let h = dirs::home_dir()?;
    let p = h.join("storage").join("shared");
    if p.is_dir() {
        Some(p)
    } else {
        None
    }
}

/// Gezici icin en iyi baslangic klasoru.
///
/// Termux'ta: `~/storage/shared/Music` -> `~/storage/shared` -> ev dizini.
/// Masaustunde: indirilenler -> ev dizini -> "/".
/// Kullanici /storage/emulated/0 yolunu elle yazmak zorunda kalmaz.
pub fn default_start() -> PathBuf {
    if let Some(shared) = shared_dir() {
        let music = shared.join("Music");
        if music.is_dir() {
            return music;
        }
        return shared;
    }
    if let Some(d) = dirs::download_dir() {
        if d.is_dir() {
            return d;
        }
    }
    dirs::home_dir()
        .filter(|p| p.is_dir())
        .unwrap_or_else(|| PathBuf::from("/"))
}

/// Depolama izni kontrolu: sorun varsa kullaniciya gosterilecek NET mesaj.
///
/// `None` = sorun yok (ya da Termux degiliz). Metin asla ham OS hatasi
/// icermez; ne yapilacagini adim adim soyler.
pub fn health(lang: Lang) -> Option<String> {
    if !is_termux() {
        return None; // masaustu: /sdcard diye bir sey yok, kontrol de yok
    }
    if shared_dir().is_some() {
        return None;
    }
    Some(match lang {
        Lang::Tr => {
            "Depolama izni verilmemiş.\n\
             \n\
             Terminale çık, şunu çalıştır:\n\
             \n    termux-setup-storage\n\
             \n\
             Açılan Android izin penceresinde 'İzin ver'e bas,\n\
             sonra tekrar 'ffstudio' yaz."
                .to_string()
        }
        Lang::En => {
            "Storage permission is not granted.\n\
             \n\
             Leave the app, run:\n\
             \n    termux-setup-storage\n\
             \n\
             Tap 'Allow' in the Android dialog,\n\
             then run 'ffstudio' again."
                .to_string()
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Masaustu Linux'ta (Termux degilken) saglik kontrolu HIC devreye
    /// girmemeli: yoksa her kullaniciya hatali ekran gosterilirdi.
    #[test]
    fn masaustunde_kontrol_yok() {
        if is_termux() {
            return; // Termux'ta calisiyorsa bu testin anlami yok
        }
        assert!(health(Lang::Tr).is_none(), "masaustunde depolama hatasi cikmamali");
    }

    /// Termux taklidi: TERMUX_VERSION + HOME'da storage/shared YOK -> mesaj gelmeli.
    #[test]
    fn termuxta_izin_yoksa_mesaj_verir() {
        let eski_home = std::env::var_os("HOME");
        let eski_tv = std::env::var_os("TERMUX_VERSION");
        let home = std::env::temp_dir().join(format!("ffstudio_storage_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&home);
        std::fs::create_dir_all(&home).unwrap();
        std::env::set_var("HOME", &home);
        std::env::set_var("TERMUX_VERSION", "0.118");

        // storage/shared yok -> net mesaj
        let m = health(Lang::Tr).expect("mesaj beklenirdi");
        assert!(m.contains("termux-setup-storage"), "mesaj ne yapilacagini soylemeli: {m}");
        assert!(
            !m.to_lowercase().contains("permission denied") && !m.contains("os error"),
            "ham OS hatasi gosterilmemeli: {m}"
        );

        // storage/shared olusturulunca susmali
        std::fs::create_dir_all(home.join("storage/shared")).unwrap();
        assert!(health(Lang::Tr).is_none(), "izin verilince mesaj kalkmali");

        // varsayilan baslangic: shared/Music varsa orasi
        std::fs::create_dir_all(home.join("storage/shared/Music")).unwrap();
        assert_eq!(default_start(), home.join("storage/shared/Music"));
        std::fs::remove_dir_all(home.join("storage/shared/Music")).unwrap();
        assert_eq!(default_start(), home.join("storage/shared"));

        match eski_home {
            Some(v) => std::env::set_var("HOME", v),
            None => std::env::remove_var("HOME"),
        }
        match eski_tv {
            Some(v) => std::env::set_var("TERMUX_VERSION", v),
            None => std::env::remove_var("TERMUX_VERSION"),
        }
        let _ = std::fs::remove_dir_all(&home);
    }
}
