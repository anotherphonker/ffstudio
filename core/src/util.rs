//! Platformdan bagimsiz yardimcilar (GUI + TUI ortak).
//!
//! Buradaki mantik tek yerde durur; hem masaustu GUI'si hem Termux TUI
//! istemcisi ayni fonksiyonlari cagirir.

use crate::ffmpeg::Media;
use std::path::{Path, PathBuf};

const INPUT_EXTS: &[&str] = &[
    // ses
    "mp3", "flac", "wav", "ogg", "oga", "opus", "m4a", "aac", "wma", "ape",
    // video
    "mp4", "mkv", "avi", "mov", "webm", "wmv", "flv", "m4v", "ts", "m2ts", "mpg", "mpeg", "3gp", "vob",
    // resim
    "jpg", "jpeg", "png", "webp", "bmp", "gif", "tif", "tiff", "avif",
];

/// Desteklenen medya uzantisi mi?
pub fn is_media_ext(ext: &str) -> bool {
    INPUT_EXTS.contains(&ext.to_lowercase().as_str())
}

pub fn walk(p: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(p) else {
        return;
    };
    let mut entries: Vec<PathBuf> = rd.flatten().map(|e| e.path()).collect();
    entries.sort();
    for e in entries {
        if e.is_dir() {
            walk(&e, out);
        } else if let Some(ext) = e.extension().and_then(|x| x.to_str()) {
            if INPUT_EXTS.contains(&ext.to_lowercase().as_str()) {
                out.push(e);
            }
        }
    }
}

pub fn fmt_size(b: u64) -> String {
    let units: &[(&str, u64)] = &[("GB", 1u64 << 30), ("MB", 1u64 << 20), ("KB", 1u64 << 10), ("B", 1)];
    for (s, v) in units {
        if b >= *v {
            return format!("{:.1} {}", b as f64 / *v as f64, s);
        }
    }
    "0 B".to_string()
}

pub fn fmt_dur(s: f64) -> String {
    if s <= 0.0 {
        return "-".into();
    }
    let t = s as u64;
    let h = t / 3600;
    let m = (t % 3600) / 60;
    let sec = t % 60;
    if h > 0 {
        format!("{h}:{m:02}:{sec:02}")
    } else {
        format!("{m}:{sec:02}")
    }
}

pub fn truncate(s: &str, max: usize) -> String {
    let count = s.chars().count();
    if count <= max {
        s.to_string()
    } else {
        s.chars().take(max.saturating_sub(3)).collect::<String>() + "..."
    }
}

/// Cikti dosyasi gercekten olusmus mu? (pattern ise ilk parca dosyasi)
pub fn output_ok(output: &Path, pattern: bool) -> bool {
    if pattern {
        let name = output
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let first = name.replace("%03d", "001");
        let dir = output.parent().unwrap_or(Path::new("."));
        dir.join(first).is_file()
    } else {
        output.metadata().map(|m| m.is_file() && m.len() > 0).unwrap_or(false)
    }
}

/// Kaynagi klasore tasi; isim cakisiyorsa "_1", "_2" ekle;
/// farkli diskler arasiysa kopyala + sil.
pub fn move_to_dir(src: &Path, dir: &Path) -> std::io::Result<PathBuf> {
    std::fs::create_dir_all(dir)?;
    let fname = src
        .file_name()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "geçersiz yol"))?;
    let mut dst = dir.join(fname);
    let mut n = 1u32;
    while dst.exists() {
        let stem = src
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        match src.extension().map(|s| s.to_string_lossy().to_string()) {
            Some(e) => dst = dir.join(format!("{stem}_{n}.{e}")),
            None => dst = dir.join(format!("{stem}_{n}")),
        }
        n += 1;
    }
    if std::fs::rename(src, &dst).is_err() {
        std::fs::copy(src, &dst)?;
        std::fs::remove_file(src)?;
    }
    Ok(dst)
}

pub fn src_kbps(m: &Media) -> Option<u64> {
    if let Some(b) = m.bit_rate {
        return Some(b / 1000);
    }
    if m.duration > 0.0 {
        return Some(((m.size as f64 * 8.0) / m.duration / 1000.0) as u64);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_to_dir_cakisma() {
        let dir = std::env::temp_dir().join(format!("ffstudio_t{}_1", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let src = dir.join("a b.mp3");
        std::fs::write(&src, b"x").unwrap();
        let dst1 = dir.join("hedef");
        std::fs::create_dir_all(&dst1).unwrap();
        let d1 = move_to_dir(&src, &dst1).unwrap();
        assert!(d1.is_file());
        std::fs::write(&src, b"y").unwrap();
        let d2 = move_to_dir(&src, &dst1).unwrap();
        assert_ne!(d1, d2, "isim cakismasinda farkli isim verilmeli");
        assert!(d2.is_file());
        assert!(!src.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn output_ok_kontrol() {
        let dir = std::env::temp_dir().join(format!("ffstudio_t{}_2", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        // pattern
        let p = dir.join("clip_parca_%03d.mp4");
        assert!(!output_ok(&p, true));
        std::fs::write(dir.join("clip_parca_001.mp4"), b"x").unwrap();
        assert!(output_ok(&p, true));
        // normal
        let q = dir.join("s.mp3");
        assert!(!output_ok(&q, false));
        std::fs::write(&q, b"x").unwrap();
        assert!(output_ok(&q, false));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
