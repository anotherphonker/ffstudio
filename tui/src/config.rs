//! Kalici ayarlar: `~/.config/ffstudio-tui/config.json`
//!
//! GUI'nin ayar dosyasindan (eframe storage) BAGIMSIZDIR; TUI kendi
//! dosyasini kullanir ama alanlar benzerdir (dil, tema, paralellik...).

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use ffstudio_core::lang::Lang;

/// Terminal renk semasi (RGB slider yerine birkac sabit tema).
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Default,
    Gruvbox,
    Mono,
    Solarized,
}

impl Theme {
    pub const ALL: [Theme; 4] = [Theme::Default, Theme::Gruvbox, Theme::Mono, Theme::Solarized];

    pub fn name(self, lang: Lang) -> &'static str {
        match (self, lang) {
            (Theme::Default, Lang::Tr) => "Varsayilan (mavi)",
            (Theme::Default, Lang::En) => "Default (blue)",
            (Theme::Gruvbox, _) => "Gruvbox",
            (Theme::Mono, Lang::Tr) => "Tek renk",
            (Theme::Mono, Lang::En) => "Monochrome",
            (Theme::Solarized, _) => "Solarized",
        }
    }
}

/// Kaynak dosyalara donusum basarili olunca ne yapilsin?
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum SrcAction {
    Keep,
    Delete,
    Move,
}

impl SrcAction {
    pub fn next(self) -> Self {
        match self {
            SrcAction::Keep => SrcAction::Delete,
            SrcAction::Delete => SrcAction::Move,
            SrcAction::Move => SrcAction::Keep,
        }
    }
    pub fn label(self, lang: Lang) -> &'static str {
        use ffstudio_core::lang::Key;
        use ffstudio_core::lang::tr;
        match self {
            SrcAction::Keep => tr(lang, Key::SrcKeep),
            SrcAction::Delete => tr(lang, Key::SrcDelete),
            SrcAction::Move => tr(lang, Key::SrcMove),
        }
    }
}

/// Cikti nereye yazilsin?
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum OutMode {
    Source,
    Dir(PathBuf),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Settings {
    /// "tr" | "en" — core'daki Lang enum'u serde turetmedigi icin
    /// dosyada metin tutulur, kullanimda Lang'e cevrilir.
    #[serde(default = "def_lang")]
    pub lang: String,
    #[serde(default = "def_theme")]
    pub theme: Theme,
    /// 0 = otomatik (CPU'ya gore), 1..=16 = manuel paralel is sayisi
    #[serde(default)]
    pub workers: u32,
    /// Mevcut cikti varsa uzerine yaz (varsayilan: atla)
    #[serde(default)]
    pub overwrite: bool,
    #[serde(default = "def_out_mode")]
    pub out_mode: OutMode,
    #[serde(default = "def_src_action")]
    pub src_action: SrcAction,
    #[serde(default)]
    pub src_move_dir: Option<PathBuf>,
}

fn def_lang() -> String {
    "tr".to_string()
}
fn def_theme() -> Theme {
    Theme::Default
}
fn def_out_mode() -> OutMode {
    OutMode::Source
}
fn def_src_action() -> SrcAction {
    SrcAction::Keep
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            lang: "tr".to_string(),
            theme: Theme::Default,
            workers: 0,
            overwrite: false,
            out_mode: OutMode::Source,
            src_action: SrcAction::Keep,
            src_move_dir: None,
        }
    }
}

/// Config dosyasinin yolu.
///
/// Sirayla: `$XDG_CONFIG_HOME` -> `$HOME/.config` -> `dirs::config_dir()`.
/// Yani Termux ve genel Linux'ta `~/.config/ffstudio-tui/config.json`,
/// XDG_CONFIG_HOME ayarliysa onun altinda.
pub fn config_path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| dirs::home_dir().map(|h| h.join(".config")))
        .or_else(dirs::config_dir);
    base.map(|b| b.join("ffstudio-tui").join("config.json"))
}

impl Settings {
    /// Ayarlardaki dil kodu -> Lang
    pub fn lang(&self) -> Lang {
        Lang::parse(&self.lang)
    }

    /// Lang -> ayar (dosyaya yazilacak kod)
    pub fn set_lang(&mut self, l: Lang) {
        self.lang = l.as_str().to_string();
    }

    /// Diskten yukle; dosya yoksa/bozuksa varsayilanlari dondurur.
    pub fn load() -> Self {
        let Some(p) = config_path() else {
            return Self::default();
        };
        match std::fs::read_to_string(&p) {
            Ok(s) => serde_json::from_str::<Settings>(&s).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Diske yaz (klasoru gerekirse olusturur). Hata sessizce yutulur:
    /// ayar kaydedememek uygulamayi durdurmamali.
    pub fn save(&self) {
        let Some(p) = config_path() else {
            return;
        };
        if let Some(dir) = p.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(s) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(&p, s);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ayarlar_json_gidis_donusu() {
        let s = Settings {
            lang: "en".to_string(),
            theme: Theme::Gruvbox,
            workers: 3,
            overwrite: true,
            out_mode: OutMode::Dir(PathBuf::from("/tmp/x")),
            src_action: SrcAction::Move,
            src_move_dir: Some(PathBuf::from("/tmp/y")),
        };
        let j = serde_json::to_string(&s).unwrap();
        let back: Settings = serde_json::from_str(&j).unwrap();
        assert!(matches!(back.lang(), Lang::En));
        assert_eq!(back.theme, Theme::Gruvbox);
        assert_eq!(back.workers, 3);
        assert!(back.overwrite);
        assert_eq!(back.out_mode, OutMode::Dir(PathBuf::from("/tmp/x")));
        assert_eq!(back.src_action, SrcAction::Move);
    }

    /// XDG_CONFIG_HOME verilirse config yolu onun altinda olmali.
    #[test]
    fn xdg_config_home_oncelikli() {
        let eski = std::env::var_os("XDG_CONFIG_HOME");
        let tmp = std::env::temp_dir().join(format!("ffstudio_xdg_{}", std::process::id()));
        std::env::set_var("XDG_CONFIG_HOME", &tmp);
        let p = config_path().expect("config yolu");
        assert!(
            p.starts_with(&tmp),
            "XDG_CONFIG_HOME kullanilmali: {}",
            p.display()
        );
        assert!(p.ends_with("ffstudio-tui/config.json"));
        match eski {
            Some(v) => std::env::set_var("XDG_CONFIG_HOME", v),
            None => std::env::remove_var("XDG_CONFIG_HOME"),
        }
    }

    #[test]
    fn bozuk_json_varsayilana_doner() {
        let bad = "{ bu json degil ";
        let s: Result<Settings, _> = serde_json::from_str(bad);
        assert!(s.is_err());
        // load() bu durumda Default doner
        let d = Settings::default();
        assert_eq!(d.workers, 0);
        assert!(!d.overwrite);
    }
}
