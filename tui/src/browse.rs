//! Dizin gezici: klavyeyle gezinme + coklu secim.
//!
//! Gercek drag-drop yok (Termux TUI); yerine bu gezici kullanilir.
//! Klasor secilirse alt klasorler dahil TUM medya dosyalari taranir.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use ffstudio_core::util;

/// Gezicinin hangi amacla acildigi.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PickMode {
    /// Medya dosyalari sec (coklu, Space ile)
    Files,
    /// Tek klasor sec (recursive tarama icin)
    Folder,
    /// Cikti klasoru sec
    OutDir,
    /// Kaynak dosyalarin tasinacagi klasor
    MoveDir,
}

pub struct Entry {
    pub path: PathBuf,
    pub is_dir: bool,
}

pub struct Browse {
    pub mode: PickMode,
    /// Arayuz dili (hata mesajlari icin)
    pub lang: ffstudio_core::lang::Lang,
    pub cur: PathBuf,
    pub entries: Vec<Entry>,
    pub sel: usize,
    pub marked: HashSet<PathBuf>,
    pub error: Option<String>,
}

impl Browse {
    /// Geziciyi baslat: baslangic klasoru verilmezse ev dizini (yoksa "/").
    pub fn new(mode: PickMode, start: Option<PathBuf>) -> Self {
        // Varsayilan: Termux'ta ~/storage/shared/Music -> ~/storage/shared -> home
        // (masaustunde indirilenler -> home). Kullaniciya /storage/emulated/0
        // yolunu elle yazdirmayiz.
        let cur = start
            .filter(|p| p.is_dir())
            .unwrap_or_else(crate::storage::default_start);
        let mut b = Browse {
            mode,
            lang: ffstudio_core::lang::Lang::Tr,
            cur: cur.clone(),
            entries: Vec::new(),
            sel: 0,
            marked: HashSet::new(),
            error: None,
        };
        b.reload();
        b
    }

    /// Bulundugumuz klasoru yeniden oku: once klasorler, sonra dosyalar.
    pub fn reload(&mut self) {
        self.error = None;
        let rd = match std::fs::read_dir(&self.cur) {
            Ok(rd) => rd,
            Err(e) => {
                // ham OS metni ("Permission denied (os error 13)") gosterilmez
                self.error = Some(crate::errors::dir_error(&self.cur, &e, self.lang));
                self.entries.clear();
                self.sel = 0;
                return;
            }
        };
        let mut dirs: Vec<PathBuf> = Vec::new();
        let mut files: Vec<PathBuf> = Vec::new();
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                dirs.push(p);
            } else {
                files.push(p);
            }
        }
        dirs.sort();
        files.sort();
        self.entries = dirs
            .into_iter()
            .map(|path| Entry { path, is_dir: true })
            .chain(files.into_iter().map(|path| Entry { path, is_dir: false }))
            .collect();
        if self.sel >= self.entries.len() {
            self.sel = self.entries.len().saturating_sub(1);
        }
    }

    pub fn up(&mut self) {
        if self.sel > 0 {
            self.sel -= 1;
        }
    }

    pub fn down(&mut self) {
        if self.sel + 1 < self.entries.len() {
            self.sel += 1;
        }
    }

    pub fn current(&self) -> Option<&Entry> {
        self.entries.get(self.sel)
    }

    /// Enter: klasore gir; dosyaysa medya ise secime ekle.
    pub fn enter(&mut self) {
        let Some(e) = self.current() else {
            return;
        };
        let path = e.path.clone();
        let is_dir = e.is_dir;
        if is_dir {
            self.cur = path;
            self.sel = 0;
            self.reload();
        } else if util::is_media_ext(
            path.extension()
                .and_then(|x| x.to_str())
                .unwrap_or_default(),
        ) {
            self.toggle_mark();
        }
    }

    /// Ust klasore cik.
    pub fn go_up(&mut self) {
        if let Some(parent) = self.cur.parent() {
            let parent = parent.to_path_buf();
            self.cur = parent;
            self.sel = 0;
            self.reload();
        }
    }

    /// Space: imlecin uzerindeki ogeyi isaretle/kaldir.
    pub fn toggle_mark(&mut self) {
        let Some(e) = self.current() else {
            return;
        };
        let p = e.path.clone();
        if self.marked.contains(&p) {
            self.marked.remove(&p);
        } else {
            self.marked.insert(p);
        }
        self.down();
    }

    /// Su anki klasordeki tum dosyalari isaretle (klasorler haric).
    pub fn mark_all_files(&mut self) {
        let all: Vec<PathBuf> = self
            .entries
            .iter()
            .filter(|e| !e.is_dir)
            .map(|e| e.path.clone())
            .collect();
        let every = all.iter().all(|p| self.marked.contains(p));
        for p in all {
            if every {
                self.marked.remove(&p);
            } else {
                self.marked.insert(p);
            }
        }
    }

    /// Secimi temizle.
    pub fn clear_marks(&mut self) {
        self.marked.clear();
    }

    /// Enter/Home tusuyla secim onaylandiginda ne yapilacak:
    /// - Files: isaretli dosyalar (bos ise imlecin uzerindeki medya dosyasi)
    /// - Folder/OutDir/MoveDir: o anki klasor
    pub fn confirm(&self) -> PickResult {
        match self.mode {
            PickMode::Files => {
                let mut out: Vec<PathBuf> = self.marked.iter().cloned().collect();
                out.sort();
                if out.is_empty() {
                    if let Some(e) = self.current() {
                        if !e.is_dir {
                            out.push(e.path.clone());
                        } else {
                            // klasor uzerinde Enter'a basildi: klasoru komple ekle
                            return PickResult::Paths(vec![e.path.clone()]);
                        }
                    }
                }
                PickResult::Paths(out)
            }
            PickMode::Folder => PickResult::Paths(vec![self.cur.clone()]),
            PickMode::OutDir => PickResult::Dir(self.cur.clone()),
            PickMode::MoveDir => PickResult::Dir(self.cur.clone()),
        }
    }
}

pub enum PickResult {
    /// Eklenecek dosya/klasor yollari
    Paths(Vec<PathBuf>),
    /// Secilen klasor (cikti/tasima hedefi)
    Dir(PathBuf),
}

/// Gezici icin makul baslangic klasorleri (Termux + masaustu).
pub fn quick_starts() -> Vec<(&'static str, PathBuf)> {
    let mut v: Vec<(&'static str, PathBuf)> = Vec::new();
    // Termux: ~/storage/shared (+ Music) once gelir
    if let Some(shared) = crate::storage::shared_dir() {
        v.push(("Depolama", shared.clone()));
        let music = shared.join("Music");
        if music.is_dir() {
            v.push(("Music", music));
        }
        let movies = shared.join("Movies");
        if movies.is_dir() {
            v.push(("Movies", movies));
        }
    }
    if let Some(h) = dirs::home_dir() {
        v.push(("~", h.clone()));
    }
    for (label, p) in [
        ("/sdcard", PathBuf::from("/sdcard")),
        (
            "/sdcard/Download",
            PathBuf::from("/sdcard/Download"),
        ),
        (
            "/storage/emulated/0",
            PathBuf::from("/storage/emulated/0"),
        ),
    ] {
        if p.is_dir() {
            v.push((label, p));
        }
    }
    if let Some(d) = dirs::download_dir() {
        if d.is_dir() && !v.iter().any(|(_, p)| *p == d) {
            v.push(("Downloads", d));
        }
    }
    if !v.iter().any(|(_, p)| p == Path::new("/")) {
        v.push(("/", PathBuf::from("/")));
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmpdir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("ffstudio_tui_browse_{}_{}", std::process::id(), tag));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn klasorler_once_dosyalar_sonra() {
        let d = tmpdir("sira");
        std::fs::create_dir_all(d.join("b_klasor")).unwrap();
        std::fs::create_dir_all(d.join("a_klasor")).unwrap();
        std::fs::write(d.join("z.mp3"), b"x").unwrap();
        std::fs::write(d.join("a.mp3"), b"x").unwrap();

        let b = Browse::new(PickMode::Files, Some(d.clone()));
        let names: Vec<String> = b
            .entries
            .iter()
            .map(|e| e.path.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        assert_eq!(names, vec!["a_klasor", "b_klasor", "a.mp3", "z.mp3"]);
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn space_ile_coklu_secim_ve_onay() {
        let d = tmpdir("secim");
        std::fs::write(d.join("a.flac"), b"x").unwrap();
        std::fs::write(d.join("b.flac"), b"x").unwrap();
        std::fs::write(d.join("not.txt"), b"x").unwrap();

        let mut b = Browse::new(PickMode::Files, Some(d.clone()));
        // ilk medya dosyasina in (dosyalar klasorden sonra gelir)
        b.toggle_mark(); // a.flac
        b.toggle_mark(); // b.flac
        match b.confirm() {
            PickResult::Paths(v) => {
                assert_eq!(v.len(), 2, "isaretli iki dosya donmeli: {v:?}");
                assert!(v.iter().all(|p| p.extension().unwrap() == "flac"));
            }
            _ => panic!("Paths bekleniyordu"),
        }
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn medya_olmayan_isaretlenmez_enter_ile() {
        let d = tmpdir("medya");
        std::fs::write(d.join("not.txt"), b"x").unwrap();
        let mut b = Browse::new(PickMode::Files, Some(d.clone()));
        b.enter(); // not.txt uzerinde: medya degil -> isaretlenmemeli
        assert!(b.marked.is_empty(), "txt dosyasi isaretlenmemeli");
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn klasore_gir_ve_cik() {
        let d = tmpdir("gir_cik");
        let sub = d.join("alt");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::write(sub.join("x.mp3"), b"x").unwrap();

        let mut b = Browse::new(PickMode::Files, Some(d.clone()));
        b.enter(); // ilk girdi klasor -> icine gir
        assert_eq!(b.cur, sub);
        assert_eq!(b.entries.len(), 1);
        b.go_up();
        assert_eq!(b.cur, d);
        let _ = std::fs::remove_dir_all(&d);
    }
}
