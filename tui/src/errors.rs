//! Ham isletim sistemi hatalarini kullanici dostu metne cevirir.
//!
//! Kural: kullaniciya ASLA "Permission denied", "os error 13",
//! "No such file or directory" gibi ciplak OS metni gosterilmez;
//! hepsi Turkce (ya da secili dile gore) net bir cumleye cevrilir.

use ffstudio_core::lang::Lang;

/// Bilinen OS hata kalibi var mi? Varsa dile uygun mesaji dondurur.
fn match_pattern(raw: &str, lang: Lang) -> Option<&'static str> {
    let low = raw.to_lowercase();
    let has = |p: &str| low.contains(p);
    match lang {
        Lang::Tr => {
            if has("permission denied") || has("os error 13") || has("eacces") {
                Some("Erişim izni yok (izin verilmemiş bir klasör olabilir)")
            } else if has("no such file or directory") || has("os error 2") {
                Some("Dosya ya da klasör bulunamadı")
            } else if has("is a directory") || has("os error 21") {
                Some("Bu bir klasör, dosya değil")
            } else if has("no space left") || has("os error 28") {
                Some("Cihazda yer kalmadı")
            } else if has("read-only file system") || has("os error 30") {
                Some("Dosya sistemi salt okunur")
            } else if has("text file busy") {
                Some("Dosya şu an başka bir işlem tarafından kullanılıyor")
            } else {
                None
            }
        }
        Lang::En => {
            if has("permission denied") || has("os error 13") || has("eacces") {
                Some("Permission denied (the folder may not be accessible)")
            } else if has("no such file or directory") || has("os error 2") {
                Some("File or folder not found")
            } else if has("is a directory") || has("os error 21") {
                Some("That is a folder, not a file")
            } else if has("no space left") || has("os error 28") {
                Some("No space left on device")
            } else if has("read-only file system") || has("os error 30") {
                Some("File system is read-only")
            } else if has("text file busy") {
                Some("File is currently in use by another process")
            } else {
                None
            }
        }
    }
}

/// Serbest metindeki ham OS hatalarini dostu metne cevir.
///
/// Ornek: `"okuma hatası: Permission denied (os error 13)"` ->
/// `"okuma hatası: Erişim izni yok (izin verilmemiş bir klasör olabilir)"`.
pub fn map_text(raw: &str, lang: Lang) -> String {
    let mut out = raw.to_string();
    for kalip in [
        "Permission denied (os error 13)",
        "Permission denied",
        "os error 13",
        "No such file or directory (os error 2)",
        "No such file or directory",
        "os error 2",
        "Is a directory (os error 21)",
        "os error 21",
        "No space left on device (os error 28)",
        "os error 28",
        "Read-only file system (os error 30)",
        "os error 30",
    ] {
        if out.contains(kalip) {
            if let Some(ceviri) = match_pattern(kalip, lang) {
                out = out.replace(kalip, ceviri);
            }
        }
    }
    out
}

/// `std::io::Error` -> dile uygun net mesaj.
pub fn io_error(e: &std::io::Error, lang: Lang) -> String {
    let raw = e.to_string();
    if let Some(m) = match_pattern(&raw, lang) {
        return m.to_string();
    }
    // Bilinmeyen hata: ham metni gostermek yerine tur bazli genel mesaj
    use std::io::ErrorKind as K;
    match e.kind() {
        K::NotFound => match lang {
            Lang::Tr => "Dosya ya da klasör bulunamadı",
            Lang::En => "File or folder not found",
        },
        K::PermissionDenied => match lang {
            Lang::Tr => "Erişim izni yok",
            Lang::En => "Permission denied",
        },
        K::AlreadyExists => match lang {
            Lang::Tr => "Zaten var",
            Lang::En => "Already exists",
        },
        K::InvalidInput | K::InvalidData => match lang {
            Lang::Tr => "Geçersiz veri",
            Lang::En => "Invalid data",
        },
        _ => match lang {
            Lang::Tr => "Dosya işlemi başarısız",
            Lang::En => "File operation failed",
        },
    }
    .to_string()
}

/// Bir klasor okunamadiginda gezicide gosterilecek metin (ham OS metni YOK).
pub fn dir_error(path: &std::path::Path, e: &std::io::Error, lang: Lang) -> String {
    let why = io_error(e, lang);
    match lang {
        Lang::Tr => format!("{} açılamadı: {why}", path.display()),
        Lang::En => format!("{} could not be opened: {why}", path.display()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ham_os_metni_cevrilir() {
        let t = map_text("okuma hatası: Permission denied (os error 13)", Lang::Tr);
        assert!(!t.contains("Permission denied"), "ham metin kalmamali: {t}");
        assert!(!t.contains("os error"), "os error kalmamali: {t}");
        assert!(t.contains("Erişim izni yok"), "{t}");

        let t2 = map_text("No such file or directory (os error 2)", Lang::Tr);
        assert!(t2.contains("bulunamadı"), "{t2}");

        let t3 = map_text("Permission denied", Lang::En);
        assert!(t3.starts_with("Permission denied"), "EN'de Ingilizce mesaj: {t3}");
        assert!(!t3.contains("os error"), "{t3}");
    }

    #[test]
    fn bilinmeyen_hata_tur_bazli_genel_mesaj() {
        let e = std::io::Error::new(std::io::ErrorKind::NotFound, "weird internal detail 0x99");
        let m = io_error(&e, Lang::Tr);
        assert!(!m.contains("0x99"), "ic detay gosterilmemeli: {m}");
        assert!(m.contains("bulunamadı"), "{m}");
    }

    #[test]
    fn klasor_hatasi_yol_icerir() {
        let e = std::io::Error::from_raw_os_error(13);
        let m = dir_error(std::path::Path::new("/sdcard/Yasak"), &e, Lang::Tr);
        assert!(m.contains("/sdcard/Yasak"), "{m}");
        assert!(!m.to_lowercase().contains("permission denied"), "{m}");
    }
}
