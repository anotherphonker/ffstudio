//! Dönüşüm presetleri: ffmpeg argümanları, hedef açıklaması, boyut tahmini.
//! ffmpeg'in yapabildiği temel ses/video/resim dönüşümleri + remux + özel argüman.

use crate::ffmpeg::{Kind, Media};
use anyhow::{bail, Result};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Preset {
    #[default]
    Auto,
    // ses
    Mp3V0,
    Mp3V2,
    Mp3Cbr,
    Aac,
    Opus,
    Vorbis,
    Flac,
    Wav,
    // video
    H264Mp4,
    H265Mp4,
    H265Mkv,
    Av1Mp4,
    Vp9Webm,
    // resim
    Jpeg,
    Webp,
    Avif,
    Png,
    Bmp,
    CoverArt,
    // diger
    Remux,
    Merge,
    // yeni: hedef boyut / kesme / gif / bolme
    TargetSize,
    Clip,
    Gif,
    Split,
}

impl Preset {
    /// Tum preset'ler (TUI secici menusu icin; siraya gore gruplanmis).
    pub const ALL: &'static [Preset] = &[
        Preset::Auto,
        Preset::Mp3V0,
        Preset::Mp3V2,
        Preset::Mp3Cbr,
        Preset::Aac,
        Preset::Opus,
        Preset::Vorbis,
        Preset::Flac,
        Preset::Wav,
        Preset::H264Mp4,
        Preset::H265Mp4,
        Preset::H265Mkv,
        Preset::Av1Mp4,
        Preset::Vp9Webm,
        Preset::Jpeg,
        Preset::Webp,
        Preset::Avif,
        Preset::Png,
        Preset::Bmp,
        Preset::CoverArt,
        Preset::Remux,
        Preset::Merge,
        Preset::TargetSize,
        Preset::Clip,
        Preset::Gif,
        Preset::Split,
    ];

    pub const AUDIO: &'static [Preset] = &[
        Preset::Mp3V0,
        Preset::Mp3V2,
        Preset::Mp3Cbr,
        Preset::Aac,
        Preset::Opus,
        Preset::Vorbis,
        Preset::Flac,
        Preset::Wav,
    ];
    pub const VIDEO: &'static [Preset] = &[
        Preset::H264Mp4,
        Preset::H265Mp4,
        Preset::H265Mkv,
        Preset::Av1Mp4,
        Preset::Vp9Webm,
        Preset::TargetSize,
        Preset::Clip,
        Preset::Gif,
        Preset::Split,
    ];
    pub const IMAGE: &'static [Preset] = &[Preset::Jpeg, Preset::Webp, Preset::Avif, Preset::Png, Preset::Bmp, Preset::CoverArt];

    pub fn label(self, lang: crate::lang::Lang) -> &'static str {
        match (self, lang) {
            (Preset::Auto, crate::lang::Lang::Tr) => "Otomatik - türüne göre (ses: MP3 V0, video: H.264, resim: JPEG)",
            (Preset::Auto, crate::lang::Lang::En) => "Auto - by type (audio: MP3 V0, video: H.264, image: JPEG)",
            (Preset::Mp3V0, crate::lang::Lang::Tr) => "Ses - MP3 (V0, en yüksek kalite)",
            (Preset::Mp3V0, crate::lang::Lang::En) => "Audio - MP3 (V0, highest quality)",
            (Preset::Mp3V2, crate::lang::Lang::Tr) => "Ses - MP3 (V2, çok iyi)",
            (Preset::Mp3V2, crate::lang::Lang::En) => "Audio - MP3 (V2, very good)",
            (Preset::Mp3Cbr, crate::lang::Lang::Tr) => "Ses - MP3 (sabit bitrate: 320/256/192/160/128)",
            (Preset::Mp3Cbr, crate::lang::Lang::En) => "Audio - MP3 (fixed bitrate: 320/256/192/160/128)",
            (Preset::Aac, crate::lang::Lang::Tr) => "Ses - AAC / M4A",
            (Preset::Aac, crate::lang::Lang::En) => "Audio - AAC / M4A",
            (Preset::Opus, crate::lang::Lang::Tr) => "Ses - Opus (modern, çok verimli)",
            (Preset::Opus, crate::lang::Lang::En) => "Audio - Opus (modern, very efficient)",
            (Preset::Vorbis, crate::lang::Lang::Tr) => "Ses - Vorbis / OGG",
            (Preset::Vorbis, crate::lang::Lang::En) => "Audio - Vorbis / OGG",
            (Preset::Flac, crate::lang::Lang::Tr) => "Ses - FLAC (lossless)",
            (Preset::Flac, crate::lang::Lang::En) => "Audio - FLAC (lossless)",
            (Preset::Wav, crate::lang::Lang::Tr) => "Ses - WAV 16-bit (lossless)",
            (Preset::Wav, crate::lang::Lang::En) => "Audio - WAV 16-bit (lossless)",
            (Preset::H264Mp4, crate::lang::Lang::Tr) => "Video - H.264 / MP4 (en uyumlu)",
            (Preset::H264Mp4, crate::lang::Lang::En) => "Video - H.264 / MP4 (most compatible)",
            (Preset::H265Mp4, crate::lang::Lang::Tr) => "Video - H.265 HEVC / MP4 (daha verimli)",
            (Preset::H265Mp4, crate::lang::Lang::En) => "Video - H.265 HEVC / MP4 (more efficient)",
            (Preset::H265Mkv, crate::lang::Lang::Tr) => "Video - H.265 HEVC / MKV",
            (Preset::H265Mkv, crate::lang::Lang::En) => "Video - H.265 HEVC / MKV",
            (Preset::Av1Mp4, crate::lang::Lang::Tr) => "Video - AV1 / MP4 (en verimli, daha yavaş)",
            (Preset::Av1Mp4, crate::lang::Lang::En) => "Video - AV1 / MP4 (most efficient, slower)",
            (Preset::Vp9Webm, crate::lang::Lang::Tr) => "Video - VP9 + Opus / WebM",
            (Preset::Vp9Webm, crate::lang::Lang::En) => "Video - VP9 + Opus / WebM",
            (Preset::Jpeg, crate::lang::Lang::Tr) => "Resim - JPEG",
            (Preset::Jpeg, crate::lang::Lang::En) => "Image - JPEG",
            (Preset::Webp, crate::lang::Lang::Tr) => "Resim - WebP",
            (Preset::Webp, crate::lang::Lang::En) => "Image - WebP",
            (Preset::Avif, crate::lang::Lang::Tr) => "Resim - AVIF (daha verimli)",
            (Preset::Avif, crate::lang::Lang::En) => "Image - AVIF (more efficient)",
            (Preset::Png, crate::lang::Lang::Tr) => "Resim - PNG (lossless)",
            (Preset::Png, crate::lang::Lang::En) => "Image - PNG (lossless)",
            (Preset::Bmp, crate::lang::Lang::Tr) => "Resim - BMP",
            (Preset::Bmp, crate::lang::Lang::En) => "Image - BMP",
            (Preset::CoverArt, crate::lang::Lang::Tr) => "Resim - Kapak görseli çıkar (ses/video içinden)",
            (Preset::CoverArt, crate::lang::Lang::En) => "Image - Extract cover art (from audio/video)",
            (Preset::Remux, crate::lang::Lang::Tr) => "Sadece kapsül değiştir (stream copy, anında biter)",
            (Preset::Remux, crate::lang::Lang::En) => "Change container only (stream copy, instant)",
            (Preset::Merge, crate::lang::Lang::Tr) => "Tüm listeyi TEK dosyada birleştir (concat)",
            (Preset::Merge, crate::lang::Lang::En) => "Merge whole list into ONE file (concat)",
            (Preset::TargetSize, crate::lang::Lang::Tr) => "Video - hedef boyut (MB) sıkıştır",
            (Preset::TargetSize, crate::lang::Lang::En) => "Video - compress to target size (MB)",
            (Preset::Clip, crate::lang::Lang::Tr) => "Video - kısa klip (baştan/sona kes)",
            (Preset::Clip, crate::lang::Lang::En) => "Video - short clip (trim from start/end)",
            (Preset::Gif, crate::lang::Lang::Tr) => "Video - GIF (ara kes, döngü)",
            (Preset::Gif, crate::lang::Lang::En) => "Video - GIF (trim middle, loop)",
            (Preset::Split, crate::lang::Lang::Tr) => "Video - parçalara böl (her parça N saniye)",
            (Preset::Split, crate::lang::Lang::En) => "Video - split into parts (each N seconds)",
        }
    }

    pub fn resolve(self, m: &Media) -> Preset {
        if self == Preset::Auto {
            match m.kind {
                Kind::Audio => Preset::Mp3V0,
                Kind::Video => Preset::H264Mp4,
                Kind::Image => Preset::Jpeg,
            }
        } else {
            self
        }
    }

    pub fn default_crf(self) -> Option<u32> {
        match self {
            Preset::H264Mp4 => Some(23),
            Preset::Clip => Some(23),
            Preset::H265Mp4 | Preset::H265Mkv => Some(28),
            Preset::Av1Mp4 => Some(30),
            Preset::Vp9Webm => Some(32),
            _ => None,
        }
    }

    /// Bu preset hangi medya türleri için anlamlı?
    /// (Arayüz, dosyalara uymayan preset'leri griler.)
    pub fn applies_to(self, kind: Kind) -> bool {
        use Kind::{Image, Video};
        match self {
            // her türe uygulanir
            Preset::Auto | Preset::Remux | Preset::Merge => true,
            // ses preset'leri: ses dosyalari + video (video'dan ses cikarma gecerli)
            Preset::Mp3V0
            | Preset::Mp3V2
            | Preset::Mp3Cbr
            | Preset::Aac
            | Preset::Opus
            | Preset::Vorbis
            | Preset::Flac
            | Preset::Wav => kind != Image,
            // video preset'leri
            Preset::H264Mp4
            | Preset::H265Mp4
            | Preset::H265Mkv
            | Preset::Av1Mp4
            | Preset::Vp9Webm
            | Preset::TargetSize
            | Preset::Clip
            | Preset::Gif
            | Preset::Split => kind == Video,
            // resim preset'leri
            Preset::Jpeg | Preset::Webp | Preset::Avif | Preset::Png | Preset::Bmp => kind == Image,
            // kapak cikarma: ses/video (icinde kapak gorevli olanlardan)
            Preset::CoverArt => kind != Image,
        }
    }

    pub fn is_video(self) -> bool {
        matches!(
            self,
            Preset::H264Mp4
                | Preset::H265Mp4
                | Preset::H265Mkv
                | Preset::Av1Mp4
                | Preset::Vp9Webm
                | Preset::TargetSize
                | Preset::Clip
                | Preset::Gif
                | Preset::Split
        )
    }

    /// "Maks. genişlik" seçeneği sunulan (ölçeklenebilen) preset'ler
    pub fn needs_scale(self) -> bool {
        matches!(
            self,
            Preset::H264Mp4
                | Preset::H265Mp4
                | Preset::H265Mkv
                | Preset::Av1Mp4
                | Preset::Vp9Webm
                | Preset::Clip
                | Preset::TargetSize
                | Preset::Jpeg
                | Preset::Webp
                | Preset::Avif
                | Preset::Png
                | Preset::Bmp
        )
    }

    pub fn is_image(self) -> bool {
        matches!(
            self,
            Preset::Jpeg | Preset::Webp | Preset::Avif | Preset::Png | Preset::Bmp
        )
    }
}

// ---------------------------------------------------------------------------
// Profil (kullaniçinin seçimleri)
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct Profile {
    pub preset: Preset,
    pub mp3_cbr: u32,        // 128..=320
    pub aac_kbps: u32,       // 0 = VBR yüksek
    pub opus_kbps: u32,      // 128..=320
    pub crf: u32,
    pub crf_touched: bool,
    pub x264_preset: u32,    // X264_PRESETS indeksi
    pub video_audio_kbps: u32,
    pub max_width: u32,      // 0 = kapali
    pub img_quality: u32,    // 5..=100
    pub remux_container: u32, // REMUX_CONTAINERS indeksi
    pub custom_args: String,
    // hedef boyut / kesme / gif / bolme
    pub target_mb: u32,        // TargetSize: hedef MB
    pub trim_start: String,    // "ss" | "dd:ss" | "ss:dd:ss" (boş = baştan)
    pub trim_end: String,      // boş = sona kadar
    pub gif_width: u32,        // GIF genişliği px
    pub gif_fps: u32,          // GIF kare hızı
    pub split_secs: u32,       // Split: parça süresi (sn)
    pub merge_name: String,    // Merge: çıktı dosya adı (uzantısız)
    pub merge_reencode: bool,  // Merge: codec'ler farklıysa güvenli olması için yeniden kodla
    pub cover_png: bool,       // CoverArt: PNG (lossless) mi, yoksa JPEG q2 mi
}

pub const X264_PRESETS: &[&str] = &[
    "ultrafast", "superfast", "veryfast", "faster", "fast", "medium", "slow", "slower",
];
pub const REMUX_CONTAINERS: &[(&str, &str)] = &[
    ("MP4 (mp4)", "mp4"),
    ("MKV (mkv)", "mkv"),
    ("WebM (webm)", "webm"),
    ("MOV (mov)", "mov"),
];

impl Default for Profile {
    fn default() -> Self {
        Self {
            preset: Preset::Auto,
            mp3_cbr: 320,
            aac_kbps: 256,
            opus_kbps: 256,
            crf: 23,
            crf_touched: false,
            x264_preset: 5, // medium
            video_audio_kbps: 192,
            max_width: 0,
            img_quality: 85,
            remux_container: 0,
            custom_args: String::new(),
            target_mb: 10,
            trim_start: String::new(),
            trim_end: String::new(),
            gif_width: 480,
            gif_fps: 10,
            split_secs: 60,
            merge_name: "birlesmis".into(),
            merge_reencode: false,
            cover_png: false,
        }
    }
}

/// "ss", "dd:ss", "ss:dd:ss" -> saniye (f64)
pub fn parse_time(s: &str) -> Option<f64> {
    let t = s.trim();
    if t.is_empty() {
        return None;
    }
    let nums: Vec<f64> = t
        .split(':')
        .map(|p| p.trim().parse::<f64>().ok())
        .collect::<Option<Vec<_>>>()?;
    if nums.is_empty() || nums.len() > 3 || nums.iter().any(|x| *x < 0.0) {
        return None;
    }
    Some(match nums.len() {
        1 => nums[0],
        2 => nums[0] * 60.0 + nums[1],
        _ => nums[0] * 3600.0 + nums[1] * 60.0 + nums[2],
    })
}

// ---------------------------------------------------------------------------
// Build + tahmin
// ---------------------------------------------------------------------------

pub struct Built {
    /// `-i` öncesine giden argümanlar (örn: -ss) — normal preset'lerde boş
    pub prefix_args: Vec<String>,
    pub args: Vec<String>,
    pub output_is_pattern: bool,
    pub ext: String,
    pub desc: String,
}

fn push_a(a: &mut Vec<String>, items: &[&str]) {
    for s in items {
        a.push(s.to_string());
    }
}

fn need_audio(m: &Media, lang: crate::lang::Lang) -> Result<()> {
    if m.audio.is_none() {
        bail!("'{}' {}", m.name, crate::lang::tr(lang, crate::lang::Key::NeedAudio))
    }
    Ok(())
}

fn need_video(m: &Media, lang: crate::lang::Lang) -> Result<()> {
    if m.video.is_none() {
        bail!("'{}' {}", m.name, crate::lang::tr(lang, crate::lang::Key::NeedVideo))
    }
    Ok(())
}

fn need_image(m: &Media, lang: crate::lang::Lang) -> Result<()> {
    if m.kind != Kind::Image {
        bail!("'{}' {}", m.name, crate::lang::tr(lang, crate::lang::Key::NeedImage))
    }
    Ok(())
}

fn video_audio_args(p: &Profile, a: &mut Vec<String>, preset: Preset) {
    if preset == Preset::Vp9Webm {
        push_a(a, &["-c:a", "libopus", "-b:a", "160k"]);
    } else {
        push_a(a, &["-c:a", "aac", "-b:a", &format!("{}k", p.video_audio_kbps)[..]]);
    }
}

/// ffmpeg argümanlarını oluşturur. Başarısızsa (uygunsuz preset) hatayı döner.
pub fn build(p: &Profile, m: &Media, lang: crate::lang::Lang) -> Result<Built> {
    let preset = p.preset.resolve(m);
    if preset == Preset::Merge {
        bail!("{}", crate::lang::tr(lang, crate::lang::Key::MergeBuildNote));
    }
    let mut pre: Vec<String> = Vec::new();
    let mut a: Vec<String> = Vec::new();
    let mut pattern = false;
    #[allow(unused_assignments)]
    let mut ext = String::new();

    match preset {
        Preset::Auto => unreachable!("preset çözümlendi"),
        Preset::Mp3V0 => {
            need_audio(m, lang)?;
            push_a(&mut a, &["-c:a", "libmp3lame", "-q:a", "0"]);
            ext = "mp3".into();
        }
        Preset::Mp3V2 => {
            need_audio(m, lang)?;
            push_a(&mut a, &["-c:a", "libmp3lame", "-q:a", "2"]);
            ext = "mp3".into();
        }
        Preset::Mp3Cbr => {
            need_audio(m, lang)?;
            push_a(&mut a, &["-c:a", "libmp3lame", "-b:a", &format!("{}k", p.mp3_cbr)[..]]);
            ext = "mp3".into();
        }
        Preset::Aac => {
            need_audio(m, lang)?;
            push_a(&mut a, &["-c:a", "aac"]);
            if p.aac_kbps == 0 {
                push_a(&mut a, &["-q:a", "0"]);
            } else {
                push_a(&mut a, &["-b:a", &format!("{}k", p.aac_kbps)[..]]);
            }
            ext = "m4a".into();
        }
        Preset::Opus => {
            need_audio(m, lang)?;
            push_a(&mut a, &["-c:a", "libopus", "-b:a", &format!("{}k", p.opus_kbps)[..]]);
            ext = "opus".into();
        }
        Preset::Vorbis => {
            need_audio(m, lang)?;
            push_a(&mut a, &["-c:a", "libvorbis", "-q:a", "6"]);
            ext = "ogg".into();
        }
        Preset::Flac => {
            need_audio(m, lang)?;
            push_a(&mut a, &["-c:a", "flac"]);
            ext = "flac".into();
        }
        Preset::Wav => {
            need_audio(m, lang)?;
            push_a(&mut a, &["-c:a", "pcm_s16le"]);
            ext = "wav".into();
        }
        Preset::H264Mp4 => {
            need_video(m, lang)?;
            let name = X264_PRESETS[(p.x264_preset as usize) % X264_PRESETS.len()];
            push_a(
                &mut a,
                &[
                    "-c:v",
                    "libx264",
                    "-preset",
                    name,
                    "-crf",
                    &p.crf.to_string()[..],
                    "-pix_fmt",
                    "yuv420p",
                ],
            );
            video_audio_args(p, &mut a, preset);
            push_a(&mut a, &["-movflags", "+faststart"]);
            ext = "mp4".into();
        }
        Preset::H265Mp4 => {
            need_video(m, lang)?;
            let name = X264_PRESETS[(p.x264_preset as usize) % X264_PRESETS.len()];
            push_a(
                &mut a,
                &[
                    "-c:v",
                    "libx265",
                    "-preset",
                    name,
                    "-crf",
                    &p.crf.to_string()[..],
                    "-tag:v",
                    "hvc1",
                    "-pix_fmt",
                    "yuv420p",
                ],
            );
            video_audio_args(p, &mut a, preset);
            push_a(&mut a, &["-movflags", "+faststart"]);
            ext = "mp4".into();
        }
        Preset::H265Mkv => {
            need_video(m, lang)?;
            let name = X264_PRESETS[(p.x264_preset as usize) % X264_PRESETS.len()];
            push_a(
                &mut a,
                &[
                    "-c:v",
                    "libx265",
                    "-preset",
                    name,
                    "-crf",
                    &p.crf.to_string()[..],
                    "-pix_fmt",
                    "yuv420p",
                ],
            );
            video_audio_args(p, &mut a, preset);
            ext = "mkv".into();
        }
        Preset::Av1Mp4 => {
            need_video(m, lang)?;
            push_a(
                &mut a,
                &[
                    "-c:v",
                    "libsvtav1",
                    "-preset",
                    "6",
                    "-crf",
                    &p.crf.to_string()[..],
                    "-pix_fmt",
                    "yuv420p",
                ],
            );
            video_audio_args(p, &mut a, preset);
            push_a(&mut a, &["-movflags", "+faststart"]);
            ext = "mp4".into();
        }
        Preset::Vp9Webm => {
            need_video(m, lang)?;
            push_a(
                &mut a,
                &[
                    "-c:v",
                    "libvpx-vp9",
                    "-b:v",
                    "0",
                    "-crf",
                    &p.crf.to_string()[..],
                    "-row-mt",
                    "1",
                ],
            );
            video_audio_args(p, &mut a, preset);
            ext = "webm".into();
        }
        Preset::Jpeg => {
            need_image(m, lang)?;
            let qv = ((31.0 - p.img_quality as f64 * 0.29).round() as u32).clamp(2, 31);
            push_a(&mut a, &["-c:v", "mjpeg", "-q:v", &qv.to_string()[..]]);
            ext = "jpg".into();
        }
        Preset::Webp => {
            need_image(m, lang)?;
            push_a(
                &mut a,
                &["-c:v", "libwebp", "-quality", &p.img_quality.to_string()[..]],
            );
            ext = "webp".into();
        }
        Preset::Avif => {
            need_image(m, lang)?;
            let crf = ((45.0 - p.img_quality as f64 * 0.25).round() as u32).clamp(5, 60);
            push_a(
                &mut a,
                &[
                    "-c:v",
                    "libsvtav1",
                    "-crf",
                    &crf.to_string()[..],
                    "-preset",
                    "6",
                    "-pix_fmt",
                    "yuv420p",
                ],
            );
            ext = "avif".into();
        }
        Preset::Png => {
            need_image(m, lang)?;
            push_a(&mut a, &["-c:v", "png"]);
            ext = "png".into();
        }
        Preset::Bmp => {
            need_image(m, lang)?;
            push_a(&mut a, &["-c:v", "bmp"]);
            ext = "bmp".into();
        }
        Preset::CoverArt => {
            // Kapak: 1. video akisini tek kare olarak al
            // (ses dosyalarinda bu attached_pic / album art akisidir)
            if !m.has_cover && m.kind == Kind::Audio {
                bail!("'{}' {}", m.name, crate::lang::tr(lang, crate::lang::Key::NoCover));
            }
            push_a(&mut a, &["-map", "0:v:0", "-frames:v", "1"]);
            if p.cover_png {
                push_a(&mut a, &["-c:v", "png"]);
                ext = "png".into();
            } else {
                push_a(&mut a, &["-q:v", "2"]);
                ext = "jpg".into();
            }
        }
        Preset::Remux => {
            let (_, c) = REMUX_CONTAINERS[(p.remux_container as usize) % REMUX_CONTAINERS.len()];
            push_a(&mut a, &["-c", "copy"]);
            if c == "mp4" || c == "mov" {
                push_a(&mut a, &["-movflags", "+faststart"]);
            }
            ext = c.into();
        }
        Preset::TargetSize => {
            need_video(m, lang)?;
            if m.duration <= 0.0 {
                bail!("'{}' {}", m.name, crate::lang::tr(lang, crate::lang::Key::TargetSizeNoDur));
            }
            let total_kbps =
                (p.target_mb as f64 * 8.0 * 1024.0 * 1024.0 / m.duration) as u64 / 1000;
            let a_kbps = m
                .audio
                .as_ref()
                .map(|x| x.bitrate.unwrap_or(192_000) / 1000)
                .unwrap_or(192);
            let v = total_kbps.saturating_sub(a_kbps).clamp(100, 12_000);
            let name = X264_PRESETS[(p.x264_preset as usize) % X264_PRESETS.len()];
            push_a(
                &mut a,
                &[
                    "-c:v",
                    "libx264",
                    "-preset",
                    name,
                    "-b:v",
                    &format!("{v}k")[..],
                    "-maxrate",
                    &format!("{}k", v * 3 / 2)[..],
                    "-bufsize",
                    &format!("{}k", v * 3)[..],
                    "-pix_fmt",
                    "yuv420p",
                ],
            );
            video_audio_args(p, &mut a, preset);
            push_a(&mut a, &["-movflags", "+faststart"]);
            ext = "mp4".into();
        }
        Preset::Clip => {
            need_video(m, lang)?;
            let (pre_ss, t_dur) = trim_args(&p.trim_start, &p.trim_end, lang)?;
            if let Some(s) = pre_ss {
                pre.push("-ss".into());
                pre.push(s);
            }
            if let Some(d) = t_dur {
                push_a(&mut a, &["-t", &d]);
            }
            let name = X264_PRESETS[(p.x264_preset as usize) % X264_PRESETS.len()];
            push_a(
                &mut a,
                &[
                    "-c:v",
                    "libx264",
                    "-preset",
                    name,
                    "-crf",
                    &p.crf.to_string()[..],
                    "-pix_fmt",
                    "yuv420p",
                ],
            );
            video_audio_args(p, &mut a, preset);
            push_a(&mut a, &["-movflags", "+faststart"]);
            ext = "mp4".into();
        }
        Preset::Gif => {
            need_video(m, lang)?;
            let (pre_ss, t_dur) = trim_args(&p.trim_start, &p.trim_end, lang)?;
            if let Some(s) = pre_ss {
                pre.push("-ss".into());
                pre.push(s);
            }
            if let Some(d) = t_dur {
                push_a(&mut a, &["-t", &d]);
            }
            let vf = format!(
                "fps={},scale={}:-1:flags=lanczos,split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse",
                p.gif_fps, p.gif_width
            );
            push_a(&mut a, &["-an", "-vf", &vf, "-loop", "0"]);
            ext = "gif".into();
        }
        Preset::Split => {
            need_video(m, lang)?;
            if p.split_secs == 0 {
                bail!("{}", crate::lang::tr(lang, crate::lang::Key::SplitZero));
            }
            push_a(
                &mut a,
                &["-c", "copy", "-f", "segment", "-segment_time", &p.split_secs.to_string()[..], "-reset_timestamps", "1"],
            );
            pattern = true;
            ext = "mp4".into();
        }
        Preset::Merge => unreachable!(),
    }

    // kucultme (video/resim) — GIF'te genişlik zaten -vf'de, Split'te kopya
    if preset.needs_scale() {
        if p.max_width > 0 {
            let w = m.video.as_ref().map(|v| v.width).unwrap_or(0);
            if w == 0 || w > p.max_width {
                // DIKKAT: "-vf" ve deger AYRI arguman olmali. Tek arguman
                // olarak ("-vf scale=...") verilirse ffmpeg
                // "Unrecognized option" hatasi verir.
                a.push("-vf".into());
                a.push(format!("scale={}:-2", p.max_width));
            }
        }
    }

    // kullanıcı özel argümanları (en sonda)
    for tok in p.custom_args.split_whitespace() {
        a.push(tok.to_string());
    }

    Ok(Built {
        prefix_args: pre,
        args: a,
        output_is_pattern: pattern,
        ext,
        desc: desc_of(preset, p, lang),
    })
}

/// (başlangıç ss -> "-ss" değeri, süre -> "-t" değeri)
fn trim_args(start: &str, end: &str, lang: crate::lang::Lang) -> Result<(Option<String>, Option<String>)> {
    let s = parse_time(start);
    let e = parse_time(end);
    match (s, e) {
        (Some(s), Some(e)) if e > s => Ok((Some(format!("{s:.3}")), Some(format!("{:.3}", e - s)))),
        (Some(_s), Some(_e)) => bail!("{}", crate::lang::tr(lang, crate::lang::Key::TrimBadOrder)),
        (Some(s), None) => Ok((Some(format!("{s:.3}")), None)),
        (None, Some(e)) => Ok((None, Some(format!("{e:.3}")))),
        (None, None) => Ok((None, None)),
    }
}

pub fn desc_of(preset: Preset, p: &Profile, lang: crate::lang::Lang) -> String {
    use crate::lang::{tr, Key};
    match preset {
        Preset::Auto => tr(lang, Key::AutoWord).to_string(),
        Preset::Mp3V0 => "MP3 V0 (~220 kbps)".into(),
        Preset::Mp3V2 => "MP3 V2 (~195 kbps)".into(),
        Preset::Mp3Cbr => format!("MP3 {} CBR", p.mp3_cbr),
        Preset::Aac => {
            if p.aac_kbps == 0 {
                format!("AAC {} (~220 kbps) - M4A", tr(lang, Key::VbrHigh))
            } else {
                format!("AAC {} kbps - M4A", p.aac_kbps)
            }
        }
        Preset::Opus => format!("Opus {} kbps", p.opus_kbps),
        Preset::Vorbis => "Vorbis q6 (~160 kbps)".into(),
        Preset::Flac => "FLAC (lossless)".into(),
        Preset::Wav => "WAV 16-bit (lossless)".into(),
        Preset::H264Mp4 => format!(
            "H.264 CRF{} + AAC {}k - MP4",
            p.crf, p.video_audio_kbps
        ),
        Preset::H265Mp4 => format!(
            "H.265 CRF{} + AAC {}k - MP4",
            p.crf, p.video_audio_kbps
        ),
        Preset::H265Mkv => format!(
            "H.265 CRF{} + AAC {}k - MKV",
            p.crf, p.video_audio_kbps
        ),
        Preset::Av1Mp4 => format!(
            "AV1 CRF{} + AAC {}k - MP4",
            p.crf, p.video_audio_kbps
        ),
        Preset::Vp9Webm => format!("VP9 CRF{} + Opus 160k - WebM", p.crf),
        Preset::Jpeg => format!("JPEG {} {}", tr(lang, Key::QualityWord), p.img_quality),
        Preset::Webp => format!("WebP {} {}", tr(lang, Key::QualityWord), p.img_quality),
        Preset::Avif => format!("AVIF {} {}", tr(lang, Key::QualityWord), p.img_quality),
        Preset::Png => "PNG (lossless)".into(),
        Preset::Bmp => "BMP".into(),
        Preset::CoverArt => {
            if p.cover_png {
                "Kapak (PNG)".into()
            } else {
                "Kapak (JPEG q2)".into()
            }
        }
        Preset::Remux => {
            let (name, _) = REMUX_CONTAINERS[(p.remux_container as usize) % REMUX_CONTAINERS.len()];
            format!("Remux {} (stream copy)", name)
        }
        Preset::Merge => tr(lang, Key::MergeDescConcat).to_string(),
        Preset::TargetSize => tr(lang, Key::TargetDesc).replace("{}" , &p.target_mb.to_string()),
        Preset::Clip => {
            let s = if p.trim_start.trim().is_empty() {
                tr(lang, Key::ClipStartWord)
            } else {
                p.trim_start.trim()
            };
            let e = if p.trim_end.trim().is_empty() {
                tr(lang, Key::ClipEndWord)
            } else {
                p.trim_end.trim()
            };
            format!("Klip {s}-{e} (H.264 CRF{})", p.crf)
        }
        Preset::Gif => {
            let mut s = format!("GIF {}px @ {}fps", p.gif_width, p.gif_fps);
            if !p.trim_start.trim().is_empty() {
                s.push_str(&format!(" {}", p.trim_start.trim()));
            }
            if !p.trim_end.trim().is_empty() {
                s.push_str(&format!("-{}", p.trim_end.trim()));
            }
            s
        }
        Preset::Split => tr(lang, Key::SplitDesc).replace("{}", &p.split_secs.to_string()),
    }
}

/// (tahmini çıktı boyutu, hedef açıklaması)
/// - Sabit bitrate ses preset'leri: bitrate x süre (guzel tahmin)
/// - Remux: kaynak boyut (kopya)
/// - CRF video / lossless / resim: belirsiz (None)
pub fn estimate(p: &Profile, m: &Media, lang: crate::lang::Lang) -> (Option<u64>, String) {
    let preset = p.preset.resolve(m);
    let kbps: Option<u64> = match preset {
        Preset::Mp3V0 => Some(220),
        Preset::Mp3V2 => Some(195),
        Preset::Mp3Cbr => Some(p.mp3_cbr as u64),
        Preset::Aac => {
            if p.aac_kbps == 0 {
                Some(220)
            } else {
                Some(p.aac_kbps as u64)
            }
        }
        Preset::Opus => Some(p.opus_kbps as u64),
        Preset::Vorbis => Some(160),
        _ => None,
    };
    let est = match preset {
        Preset::Remux => Some(m.size),
        Preset::TargetSize => Some(p.target_mb as u64 * 1024 * 1024),
        Preset::Split => Some(m.size),
        Preset::Clip | Preset::Gif | Preset::Merge => None,
        _ => kbps
            .filter(|_| m.duration > 0.0)
            .map(|k| (((k * 1000 / 8) as f64) * (m.duration as f64)) as u64),
    };
    (est, desc_of(preset, p, lang))
}

// ---------------------------------------------------------------------------
// Testler
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffmpeg::{AudioInfo, VideoInfo};
    use std::path::PathBuf;

    fn video_media(dur: f64) -> Media {
        Media {
            path: PathBuf::from("/x/clip.mp4"),
            name: "clip.mp4".into(),
            kind: Kind::Video,
            size: 10_000_000,
            duration: dur,
            container: "mov,mp4".into(),
            has_cover: false,
            video: Some(VideoInfo {
                codec: "h264".into(),
                profile: "High".into(),
                width: 1920,
                height: 1080,
                fps: 30.0,
                pix_fmt: "yuv420p".into(),
                bitrate: Some(4_000_000),
            }),
            audio: Some(AudioInfo {
                codec: "aac".into(),
                sample_rate: 44100,
                channels: 2,
                bitrate: Some(192_000),
            }),
            bit_rate: Some(4_192_000),
            extra: vec![],
        }
    }

    #[test]
    fn parse_time_ok() {
        assert_eq!(parse_time("30"), Some(30.0));
        assert_eq!(parse_time("1:05"), Some(65.0));
        assert_eq!(parse_time("01:02:03"), Some(3723.0));
        assert_eq!(parse_time(""), None);
        assert_eq!(parse_time("xx"), None);
    }

    #[test]
    fn target_size_hesap() {
        let mut p = Profile::default();
        p.preset = Preset::TargetSize;
        p.target_mb = 10;
        let m = video_media(100.0); // 100 sn: toplam ~838 kbps - ses 192 = ~646 kbps video
        let b = build(&p, &m, crate::lang::Lang::Tr).unwrap();
        let i = b.args.iter().position(|a| a == "-b:v").unwrap();
        let kb: u32 = b.args[i + 1].trim_end_matches('k').parse().unwrap();
        assert!((620..660).contains(&kb), "video bitrate beklenenden sapti: {kb}");
        assert_eq!(b.ext, "mp4");
        assert!(est_target(&p, &m).is_some());
    }

    fn est_target(p: &Profile, m: &Media) -> Option<u64> {
        estimate(p, m, crate::lang::Lang::Tr).0
    }

    #[test]
    fn clip_sureler() {
        let mut p = Profile::default();
        p.preset = Preset::Clip;
        p.trim_start = "30".into();
        p.trim_end = "1:10".into(); // 70 sn -> 40 sn'lik klip
        let m = video_media(200.0);
        let b = build(&p, &m, crate::lang::Lang::Tr).unwrap();
        assert_eq!(b.prefix_args, vec!["-ss".to_string(), "30.000".to_string()]);
        let i = b.args.iter().position(|a| a == "-t").unwrap();
        assert_eq!(b.args[i + 1], "40.000");
        assert_eq!(b.ext, "mp4");
    }

    #[test]
    fn gif_argumani() {
        let mut p = Profile::default();
        p.preset = Preset::Gif;
        let m = video_media(200.0);
        let b = build(&p, &m, crate::lang::Lang::Tr).unwrap();
        assert_eq!(b.ext, "gif");
        assert!(b.args.iter().any(|a| a.contains("palettegen")));
        assert!(b.args.contains(&"-an".to_string()));
    }

    #[test]
    fn split_pattern() {
        let mut p = Profile::default();
        p.preset = Preset::Split;
        let m = video_media(200.0);
        let b = build(&p, &m, crate::lang::Lang::Tr).unwrap();
        assert!(b.output_is_pattern);
        assert!(b.args.contains(&"segment".to_string()));
        assert!(b.args.contains(&"-c".to_string()));
        assert_eq!(b.ext, "mp4");
    }

    #[test]
    fn merge_build_reddeder() {
        let mut p = Profile::default();
        p.preset = Preset::Merge;
        let m = video_media(200.0);
        assert!(build(&p, &m, crate::lang::Lang::Tr).is_err());
    }

    #[test]
    fn preset_tur_uygulanabilirligi() {
        use Kind::{Audio, Image, Video};
        // ses presetleri: ses + video (ses cikarma), resim DEGIL
        assert!(Preset::Mp3V0.applies_to(Audio));
        assert!(Preset::Opus.applies_to(Video));
        assert!(!Preset::Mp3V0.applies_to(Image));
        // video presetleri: sadece video
        assert!(Preset::H264Mp4.applies_to(Video));
        assert!(Preset::Gif.applies_to(Video));
        assert!(!Preset::Gif.applies_to(Audio));
        assert!(!Preset::TargetSize.applies_to(Audio));
        // resim presetleri: sadece resim
        assert!(Preset::Jpeg.applies_to(Image));
        assert!(!Preset::Jpeg.applies_to(Audio));
        assert!(!Preset::Png.applies_to(Video));
        // evrensel
        assert!(Preset::Auto.applies_to(Audio));
        assert!(Preset::Merge.applies_to(Video));
        assert!(Preset::Remux.applies_to(Image));
    }
}