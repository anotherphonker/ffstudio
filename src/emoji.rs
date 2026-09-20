//! Gömülü renkli emoji (Twemoji PNG, 72x72).
//!
//! Emoji dosyaları `include_bytes!` ile .exe'nin İÇİNE gömülür; görünüm
//! Windows sürümünden ve sistem fontlarından tamamen bağımsızdır —
//! Windows 10 / 11 / LTSC fark etmeksizin birebir aynı renkli emoji çizilir.
//!
//! Dosya adlandırması: Unicode kod noktası (hex), bayrak dizileri tire ile
//! birleşik — ör: `1f3ac.png` = U+1F3AC (🎬), `1f1f9-1f1f7.png` = 🇹🇹.
//! Twemoji grafikleri CC-BY 4.0 lisanslıdır (© Twitter/Mozilla).

use std::collections::HashMap;

use eframe::egui::{ColorImage, Context, TextureHandle, TextureOptions};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Emoji {
    Clapperboard, // 🎬 1f3ac — logo
    Folder,       // 📁 1f4c1
    OpenFolder,   // 📂 1f4c2
    Controls,     // 🎛 1f39b — profil
    BarChart,     // 📊 1f4ca — verim
    Thread,       // 🧵 1f9f5 — kuyruk
    Scroll,       // 📜 1f4dc — log
    Palette,      // 🎨 1f3a8 — tema
    Info,         // ℹ️ 2139 — hakkında
    Rocket,       // 🚀 1f680 — dönüştür
    Globe,        // 🌐 1f310 — dil
    Gear,         // ⚙️ 2699 — ayarlar
    FlagTr,       // 🇹 1f1f9-1f1f7
    FlagUs,       // 🇺🇸 1f1fa-1f1f8
}

pub const ALL: [Emoji; 14] = [
    Emoji::Clapperboard,
    Emoji::Folder,
    Emoji::OpenFolder,
    Emoji::Controls,
    Emoji::BarChart,
    Emoji::Thread,
    Emoji::Scroll,
    Emoji::Palette,
    Emoji::Info,
    Emoji::Rocket,
    Emoji::Globe,
    Emoji::Gear,
    Emoji::FlagTr,
    Emoji::FlagUs,
];

impl Emoji {
    pub const fn label(self) -> &'static str {
        match self {
            Emoji::Clapperboard => "emoji_clapperboard",
            Emoji::Folder => "emoji_folder",
            Emoji::OpenFolder => "emoji_openfolder",
            Emoji::Controls => "emoji_controls",
            Emoji::BarChart => "emoji_barchart",
            Emoji::Thread => "emoji_thread",
            Emoji::Scroll => "emoji_scroll",
            Emoji::Palette => "emoji_palette",
            Emoji::Info => "emoji_info",
            Emoji::Rocket => "emoji_rocket",
            Emoji::Globe => "emoji_globe",
            Emoji::Gear => "emoji_gear",
            Emoji::FlagTr => "emoji_flagtr",
            Emoji::FlagUs => "emoji_flagus",
        }
    }

    pub const fn bytes(self) -> &'static [u8] {
        match self {
            Emoji::Clapperboard => include_bytes!("../assets/emoji/1f3ac.png"),
            Emoji::Folder => include_bytes!("../assets/emoji/1f4c1.png"),
            Emoji::OpenFolder => include_bytes!("../assets/emoji/1f4c2.png"),
            Emoji::Controls => include_bytes!("../assets/emoji/1f39b.png"),
            Emoji::BarChart => include_bytes!("../assets/emoji/1f4ca.png"),
            Emoji::Thread => include_bytes!("../assets/emoji/1f9f5.png"),
            Emoji::Scroll => include_bytes!("../assets/emoji/1f4dc.png"),
            Emoji::Palette => include_bytes!("../assets/emoji/1f3a8.png"),
            Emoji::Info => include_bytes!("../assets/emoji/2139.png"),
            Emoji::Rocket => include_bytes!("../assets/emoji/1f680.png"),
            Emoji::Globe => include_bytes!("../assets/emoji/1f310.png"),
            Emoji::Gear => include_bytes!("../assets/emoji/2699.png"),
            Emoji::FlagTr => include_bytes!("../assets/emoji/1f1f9-1f1f7.png"),
            Emoji::FlagUs => include_bytes!("../assets/emoji/1f1fa-1f1f8.png"),
        }
    }
}

/// Emoji dokuları: ilk çalıştırmada PNG baytlarından GPU dokusuna dönüştürülür.
pub struct Emojis {
    map: HashMap<Emoji, TextureHandle>,
}

impl Emojis {
    pub fn new() -> Self {
        Emojis {
            map: HashMap::new(),
        }
    }

    pub fn init(&mut self, ctx: &Context) {
        for e in ALL {
            if !self.map.contains_key(&e) {
                self.load(ctx, e);
            }
        }
    }

    fn load(&mut self, ctx: &Context, e: Emoji) {
        if let Some(img) = decode_png(e.bytes()) {
            let handle = ctx.load_texture(e.label(), img, TextureOptions::LINEAR);
            self.map.insert(e, handle);
        }
    }

    pub fn handle(&self, e: Emoji) -> Option<&TextureHandle> {
        self.map.get(&e)
    }
}

/// PNG (paletli / rgba dahil) → egui dokusu.
pub fn decode_png(bytes: &[u8]) -> Option<ColorImage> {
    let rgba = image::load_from_memory(bytes).ok()?.to_rgba8();
    let (w, h) = rgba.dimensions();
    Some(ColorImage::from_rgba_unmultiplied(
        [w as usize, h as usize],
        rgba.as_raw(),
    ))
}
