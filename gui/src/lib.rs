// FF Studio - all-in-one ffmpeg GUI (Rust + egui)
// Koyu mod, sürükle-bırak, sıralı dönüşüm kuyruğu, bitrate/verim tablosu,
// detay paneli, komut önizlemesi, ffmpeg'i gömülü taşıma.

mod emoji;

use eframe::egui;
use emoji::{Emoji, Emojis};
// Paylasilan mantik cekirdekten gelir (TUI ile AYNI kod).
use ffstudio_core::cpu::{self, CpuInfo};
use ffstudio_core::ffmpeg::{self, Ffmpeg, JobMsg, JobSpec, Kind, Media};
use ffstudio_core::lang::{self, tr, Key, Lang};
use ffstudio_core::profiles::{self, Preset, Profile, REMUX_CONTAINERS, X264_PRESETS};
use ffstudio_core::util::{fmt_dur, fmt_size, move_to_dir, output_ok, src_kbps, truncate, walk};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

#[derive(Clone, PartialEq)]
enum OutMode {
    Source,
    Dir(PathBuf),
}

/// Dönüşüm BAŞARILI bitince kaynak dosyaya ne yapılsın?
#[derive(Clone, Copy, PartialEq, Eq)]
enum SrcAction {
    Keep,
    Delete,
    Move,
}

struct Job {
    name: String,
    target: String,
    src_size: u64,
    input: PathBuf,
    output: PathBuf,
    output_is_pattern: bool,
    merge: bool,
    frac: f32,
    speed: Option<String>,
    state: JobState,
}

enum JobState {
    Queued,
    Running,
    Done { secs: f64, new_size: Option<u64> },
    Skipped { reason: String },
    Failed { msg: String },
}

const DEFAULT_ACCENT: u32 = 0x5B9DFF;

const SWATCHES: &[(&str, u32)] = &[
    ("Mavi (varsayılan)", 0x5B9DFF),
    ("Lacivert", 0x4C6FE0),
    ("Turkuaz", 0x2EC4B6),
    ("Yeşil", 0x56C271),
    ("Mor", 0xA472E8),
    ("Kırmızı", 0xE05B6E),
    ("Turuncu", 0xE89A4A),
    ("Gri", 0x9AA0B4),
];

struct App {
    dark: bool,
    accent: u32,
    hex_buf: String,
    lang: Lang,
    zoom: f32,
    about_open: bool,
    settings_open: bool,
    /// Paralel is sayisi: 0 = otomatik (CPU'ya gore), 1..=16 = manuel
    workers: u32,
    cpu: CpuInfo,
    emojis: Emojis,
    /// App logosu dokusu (embed PNG): ust panel + Hakkinda + pencere ikonu
    logo_tex: Option<egui::TextureHandle>,
    /// Zoom sliyeri suruklenirken bekletilir, birakilincaya kadar uygulanmaz
    /// (her tikte uygulamak tum UI'yi yeniden olceklendirip ekrani flaslatti)
    zoom_dirty: bool,
    ff: Result<Ffmpeg, String>,
    ff_version: String,
    files: Vec<Media>,
    selected: Option<usize>,
    pending: usize,
    pending_paths: Vec<PathBuf>,
    probe_tx: Option<mpsc::Sender<PathBuf>>,
    probe_rx: Option<mpsc::Receiver<(PathBuf, Result<Media, String>)>>,
    profile: Profile,
    out_mode: OutMode,
    overwrite: bool,
    src_action: SrcAction,
    src_move_dir: Option<PathBuf>,
    jobs: Vec<Job>,
    running: bool,
    rx: Option<mpsc::Receiver<JobMsg>>,
    log: Vec<(String, bool)>,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let (accent, dark, lang, zoom, workers) = match cc.storage {
            Some(s) => (
                s.get_string("theme.accent")
                    .and_then(|h| u32::from_str_radix(h.trim().trim_start_matches('#'), 16).ok())
                    .unwrap_or(DEFAULT_ACCENT),
                s.get_string("theme.dark").and_then(|v| v.parse().ok()).unwrap_or(true),
                s.get_string("ui.lang").map(|v| Lang::parse(&v)).unwrap_or(Lang::Tr),
                s.get_string("ui.zoom")
                    .and_then(|v| v.parse().ok())
                    .filter(|z| (0.75..=2.5).contains(z))
                    .unwrap_or(1.25),
                s.get_string("ui.workers")
                    .and_then(|v| v.parse().ok())
                    .filter(|w| (0..=16).contains(w))
                    .unwrap_or(0),
            ),
            None => (DEFAULT_ACCENT, true, Lang::Tr, 1.25, 0),
        };
        cc.egui_ctx.set_zoom_factor(zoom);
        let ff = Ffmpeg::locate(lang).map_err(|e| e.to_string());
        let ff_version = ff.as_ref().ok().map(|f| f.version_line()).unwrap_or_default();
        let cpu = CpuInfo::detect();
        let mut emojis = Emojis::new();
        emojis.init(&cc.egui_ctx);
        let logo_tex = logo_rgba().map(|(rgba, w, h)| {
            cc.egui_ctx.load_texture(
                "app_logo",
                eframe::egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &rgba),
                eframe::egui::TextureOptions::LINEAR,
            )
        });
        let mut app = App {
            dark,
            accent,
            hex_buf: format!("{:06x}", accent),
            lang,
            zoom,
            about_open: false,
            settings_open: false,
            workers,
            cpu,
            emojis,
            logo_tex,
            zoom_dirty: false,
            ff,
            ff_version,
            files: Vec::new(),
            selected: None,
            pending: 0,
            pending_paths: Vec::new(),
            probe_tx: None,
            probe_rx: None,
            profile: Profile::default(),
            out_mode: OutMode::Source,
            overwrite: false,
            src_action: SrcAction::Keep,
            src_move_dir: None,
            jobs: Vec::new(),
            running: false,
            rx: None,
            log: Vec::new(),
        };
        match &app.ff {
            Ok(f) => app.push_log(
                format!("{} ffmpeg bulundu: {}", tr(lang, Key::LogOk), f.source),
                false,
            ),
            Err(e) => app.push_log(format!("[Hata] {e}"), true),
        }
        if let Ok(f) = &app.ff {
            app.spawn_probe_worker(f.ffprobe.clone(), lang);
        }
        app.apply_theme(&cc.egui_ctx);
        app
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.draw(ctx);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        storage.set_string("theme.accent", format!("{:06x}", self.accent));
        storage.set_string("theme.dark", self.dark.to_string());
        storage.set_string("ui.lang", self.lang.as_str().to_string());
        storage.set_string("ui.zoom", self.zoom.to_string());
        storage.set_string("ui.workers", self.workers.to_string());
    }
}

impl App {
    /// Tum UI'yi cizer (eframe update'inden ayrildi ki headless testte de calissin)
    fn draw(&mut self, ctx: &egui::Context) {
        // sürükle-bırak
        let dropped = ctx.input(|i| i.raw.dropped_files.clone());
        for d in dropped {
            if let Some(p) = d.path {
                self.push_log(
                    format!("{} {}", tr(self.lang, Key::LogDropped), p.display()),
                    false,
                );
                self.add_path(&p);
            }
        }
        self.poll_probes(ctx);
        self.poll_jobs(ctx);
        if self.running || self.pending > 0 {
            ctx.request_repaint_after(Duration::from_millis(150));
        }

        self.top_panel(ctx);
        self.status_bar(ctx);
        self.left_panel(ctx);
        self.central(ctx);
        self.settings_window(ctx);
        self.about_window(ctx);
    }

    /// Headless test icin: ffmpeg'siz, dolu icerikli baslangic
    #[cfg(test)]
    fn test_new(lang: Lang, dark: bool) -> App {
        let mut app = App {
            dark,
            accent: DEFAULT_ACCENT,
            hex_buf: format!("{:06x}", DEFAULT_ACCENT),
            lang,
            zoom: 1.25,
            about_open: true,
            settings_open: true,
            workers: 0,
            // sabit CPU: testler makineden bagimsiz olsun (8T = i5-4590 benzeri)
            cpu: CpuInfo {
                model: "Test CPU".into(),
                physical: Some(4),
                logical: 8,
            },
            emojis: Emojis::new(),
            logo_tex: None,
            zoom_dirty: false,
            ff: Err("test modu - ffmpeg yok".into()),
            ff_version: String::new(),
            files: Vec::new(),
            selected: Some(0),
            pending: 0,
            pending_paths: Vec::new(),
            probe_tx: None,
            probe_rx: None,
            profile: Profile::default(),
            out_mode: OutMode::Source,
            overwrite: false,
            src_action: SrcAction::Move,
            src_move_dir: Some(std::env::temp_dir()),
            jobs: Vec::new(),
            running: false,
            rx: None,
            log: Vec::new(),
        };
        // dosyalar: ses (kapakli) + video + resim
        app.files.push(ffmpeg::testutil::audio_media("sarki.flac", 44.0, true));
        app.files.push(ffmpeg::testutil::video_media("film.mkv", 120.0));
        app.files.push(ffmpeg::testutil::image_media("foto.png"));
        // kuyruk durumlari
        app.jobs.push(Job {
            name: "a.mp3".into(),
            target: "MP3 V0".into(),
            src_size: 1_000_000,
            input: PathBuf::from("/x/a.flac"),
            output: PathBuf::from("/x/a.mp3"),
            output_is_pattern: false,
            merge: false,
            frac: 0.4,
            speed: Some("x2.1".into()),
            state: JobState::Running,
        });
        app.jobs.push(Job {
            name: "b.mp4".into(),
            target: "H.264".into(),
            src_size: 2_000_000,
            input: PathBuf::from("/x/b.mkv"),
            output: PathBuf::from("/x/b.mp4"),
            output_is_pattern: false,
            merge: false,
            frac: 1.0,
            speed: None,
            state: JobState::Done { secs: 3.5, new_size: Some(900_000) },
        });
        app.jobs.push(Job {
            name: "c.jpg".into(),
            target: "JPEG".into(),
            src_size: 3_000_000,
            input: PathBuf::from("/x/c.png"),
            output: PathBuf::from("/x/c.jpg"),
            output_is_pattern: false,
            merge: false,
            frac: 0.0,
            speed: None,
            state: JobState::Skipped { reason: "çıktı var".into() },
        });
        app.jobs.push(Job {
            name: "d.gif".into(),
            target: "GIF".into(),
            src_size: 4_000_000,
            input: PathBuf::from("/x/d.mp4"),
            output: PathBuf::from("/x/d.gif"),
            output_is_pattern: false,
            merge: false,
            frac: 0.0,
            speed: None,
            state: JobState::Failed { msg: "ffmpeg hata".into() },
        });
        app
    }

    // ------------------------------------------------------------------
    // Arka plan işlemleri
    // ------------------------------------------------------------------

    fn spawn_probe_worker(&mut self, ffprobe: PathBuf, lang: Lang) {
        let (in_tx, in_rx) = mpsc::channel::<PathBuf>();
        let (out_tx, out_rx) = mpsc::channel::<(PathBuf, Result<Media, String>)>();
        self.probe_tx = Some(in_tx);
        self.probe_rx = Some(out_rx);
        std::thread::spawn(move || {
            while let Ok(p) = in_rx.recv() {
                let res = Media::probe(&ffprobe, &p, lang).map_err(|e| e.to_string());
                let _ = out_tx.send((p, res));
            }
        });
    }

    fn add_path(&mut self, p: &Path) {
        // gorece yollar CWD'ye gore cozmek icin mutlak yola cevir (drift onlenu)
        let p = std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
        if p.is_dir() {
            let mut found = Vec::new();
            walk(&p, &mut found);
            self.push_log(
                format!(
                    "{} {}: {} {}",
                    tr(self.lang, Key::LogFolder),
                    p.display(),
                    found.len(),
                    tr(self.lang, Key::MediaFound)
                ),
                false,
            );
            for f in found {
                self.enqueue_probe(f);
            }
        } else if p.is_file() {
            self.enqueue_probe(p);
        }
    }

    fn enqueue_probe(&mut self, p: PathBuf) {
        if self.files.iter().any(|f| f.path == p) {
            return;
        }
        if self.pending_paths.iter().any(|x| *x == p) {
            return;
        }
        if let Some(tx) = &self.probe_tx {
            if tx.send(p.clone()).is_ok() {
                self.pending_paths.push(p);
                self.pending += 1;
            }
        } else {
            self.push_log(
                format!("{} {}", tr(self.lang, Key::LogError), tr(self.lang, Key::FfmpegMissingProbe)),
                true,
            );
        }
    }

    fn poll_probes(&mut self, ctx: &egui::Context) {
        // once: mesajlari al (borcu bitir), sonra state'i guncelle
        let msgs: Vec<(PathBuf, Result<Media, String>)> = self
            .probe_rx
            .as_ref()
            .map(|rx| {
                let mut v = Vec::new();
                while let Ok(m) = rx.try_recv() {
                    v.push(m);
                }
                v
            })
            .unwrap_or_default();
        if msgs.is_empty() {
            return;
        }
        for (p, res) in msgs {
            self.pending = self.pending.saturating_sub(1);
            self.pending_paths.retain(|x| *x != p);
            match res {
                Ok(m) => {
                    if !self.files.iter().any(|f| f.path == p) {
                        self.push_log(
                            format!("{} {}", tr(self.lang, Key::LogAdded), m.name),
                            false,
                        );
                        // suspeli dosya uyarisi: ses dosyasi ama ornekleme hizi okunamamis
                        if m.kind == Kind::Audio {
                            if let Some(a) = &m.audio {
                                if a.sample_rate == 0 {
                                    self.push_log(
                                        format!(
                                            "{} {}: {}",
                                            tr(self.lang, Key::LogWarning),
                                            m.name,
                                            tr(self.lang, Key::SuspiciousFile)
                                        ),
                                        true,
                                    );
                                }
                            }
                        }
                        self.files.push(m);
                    }
                }
                Err(e) => self.push_log(
                    format!("{} {}: {e}", tr(self.lang, Key::LogSkippedFile), p.display()),
                    true,
                ),
            }
        }
        ctx.request_repaint();
    }

    fn pick_files(&mut self) {
        let paths = rfd::FileDialog::new()
            .set_title(tr(self.lang, Key::PickFilesTitle))
            .pick_files()
            .unwrap_or_default();
        for p in &paths {
            self.add_path(p);
        }
    }

    fn pick_folder(&mut self) {
        if let Some(d) = rfd::FileDialog::new()
            .set_title(tr(self.lang, Key::PickFolderTitle))
            .pick_folder()
        {
            self.add_path(&d);
        }
    }

    fn start_conversion(&mut self) {
        if self.running {
            return;
        }
        let lang = self.lang;
        if self.ff.is_err() {
            self.push_log(
                format!("{} {}", tr(lang, Key::LogError), tr(lang, Key::NoFfmpegStart)),
                true,
            );
            return;
        }
        let ffmpeg_path = self.ff.as_ref().unwrap().ffmpeg.clone();
        if self.profile.preset == Preset::Merge {
            self.start_merge(ffmpeg_path);
            return;
        }
        let mut specs = Vec::new();
        let mut errors: Vec<String> = Vec::new();
        for (i, m) in self.files.iter().enumerate() {
            let out_dir = match &self.out_mode {
                OutMode::Source => m.path.parent().map(|p| p.to_path_buf()).unwrap_or_default(),
                OutMode::Dir(d) => d.clone(),
            };
            match profiles::build(&self.profile, m, lang) {
                Ok(b) => {
                    let stem = m
                        .path
                        .file_stem()
                        .map(|s| s.to_string_lossy().into_owned())
                        .unwrap_or_else(|| m.name.clone());
                    let out = if b.output_is_pattern {
                        out_dir.join(format!("{stem}_parca_%03d.{}", b.ext))
                    } else {
                        out_dir.join(format!("{stem}.{}", b.ext))
                    };
                    // GUVENLIK: çıktı, kaynak dosyanın ta kendisiyse asla çalıştırma
                    // (hata durumunda orijinal dosya silinir!)
                    let skip_reason = if out == m.path {
                        Some(tr(lang, Key::SkipInPlace).to_string())
                    } else if out.exists() && !self.overwrite {
                        Some(tr(lang, Key::SkipExists).to_string())
                    } else {
                        None
                    };
                    if let Some(r) = &skip_reason {
                        errors.push(format!("[{}] {}: {r}", tr(lang, Key::QueueSkipped), m.name));
                    }
                    specs.push(JobSpec {
                        job_index: i,
                        input_prefix_args: b.prefix_args.clone(),
                        input: m.path.clone(),
                        input_name: m.name.clone(),
                        output: out,
                        args: b.args.clone(),
                        duration: m.duration,
                        target_desc: b.desc.clone(),
                        skip: skip_reason.is_some(),
                        skip_reason,
                        output_is_pattern: b.output_is_pattern,
                        cleanup: None,
                    });
                }
                Err(e) => errors.push(format!("[{}] {}: {e}", tr(lang, Key::QueueSkipped), m.name)),
            }
        }
        for e in errors {
            self.push_log(e, true);
        }
        if specs.is_empty() {
            self.push_log(tr(lang, Key::NoFilesToConvert).to_string(), true);
            return;
        }
        let sizes: Vec<u64> = self.files.iter().map(|m| m.size).collect();
        self.jobs = specs
            .iter()
            .map(|s| Job {
                name: s.input_name.clone(),
                target: s.target_desc.clone(),
                src_size: sizes.get(s.job_index).copied().unwrap_or(0),
                input: s.input.clone(),
                output: s.output.clone(),
                output_is_pattern: s.output_is_pattern,
                merge: false,
                frac: 0.0,
                speed: None,
                state: s
                    .skip_reason
                    .clone()
                    .map(|r| JobState::Skipped { reason: r })
                    .unwrap_or(JobState::Queued),
            })
            .collect();
        let n_run = specs.iter().filter(|s| !s.skip).count();
        if n_run == 0 {
            // tumu atlandi: calisma moduna girmesin (AllDone bekleyip
            // "Durdur" butonunda takilmasin)
            self.push_log(tr(lang, Key::NoFilesToConvert).to_string(), true);
            return;
        }
        let (workers, threads) = self.parallel_plan(n_run);
        let runnable: Vec<JobSpec> = specs
            .iter()
            .filter(|s| !s.skip)
            .cloned()
            .map(|mut s| {
                // her isin kodlayicisina otomatik thread; kullanici kendi
                // -threads argumanini verdiyse O kazanir (son -threads onemli)
                s.args.insert(0, "-threads".into());
                s.args.insert(1, threads.to_string());
                s
            })
            .collect();
        let skipped = specs.iter().filter(|s| s.skip).count();
        self.push_log(
            format!(
                "[{}] {} {} ({} × {}t), {} {}",
                tr(lang, Key::LogQueue),
                runnable.len(),
                tr(lang, Key::QueueWillRun),
                workers,
                threads,
                skipped,
                tr(lang, Key::QueueSkipped)
            ),
            false,
        );
        // Her isin gercek komutunu logla (hata ayiklama icin)
        for s in &runnable {
            let pre = if s.input_prefix_args.is_empty() {
                String::new()
            } else {
                format!("{} ", s.input_prefix_args.join(" "))
            };
            let cmd = format!(
                "ffmpeg -nostdin -loglevel error -y {pre}-i \"{}\"  {}  \"{}\"",
                s.input.display(),
                s.args.join(" "),
                s.output.display()
            );
            self.push_log(
                format!("{} {}: {}", tr(lang, Key::LogCommand), s.input_name, cmd),
                false,
            );
        }
        self.running = true;
        let (tx, rx) = mpsc::channel();
        self.rx = Some(rx);
        ffmpeg::start_jobs(ffmpeg_path, runnable, tx, workers);
    }

    /// Birleştirme: listedeki tüm dosyaları concat demuxer ile tek dosyada birleştirir.
    fn start_merge(&mut self, ffmpeg_path: PathBuf) {
        let lang = self.lang;
        let files: Vec<Media> = self.files.clone();
        if files.len() < 2 {
            self.push_log(
                format!("{} {}", tr(lang, Key::LogError), tr(lang, Key::MergeNeedTwo)),
                true,
            );
            return;
        }
        let out_dir = match &self.out_mode {
            OutMode::Source => files[0]
                .path
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_default(),
            OutMode::Dir(d) => d.clone(),
        };
        let name = self.profile.merge_name.trim();
        let mut stem = if name.is_empty() { "birlesmis".to_string() } else { name.to_string() };
        let mut out = out_dir.join(format!("{stem}.mp4"));
        // GUVENLIK: çıktı bir kaynak dosyanın üzerine yazmasin
        if files.iter().any(|m| m.path == out) {
            stem = format!("{stem}_birlesik");
            out = out_dir.join(format!("{stem}.mp4"));
        }
        let skip = out.exists() && !self.overwrite;
        // geçici concat listesi (ffmpeg istediği biçimde: file 'yol')
        let list = std::env::temp_dir().join(format!("ffstudio_concat_{}.txt", std::process::id()));
        let mut txt = String::new();
        for m in &files {
            let p = m.path.to_string_lossy().replace('\\', "/");
            let q = p.replace('\'', r"'\''");
            txt.push_str(&format!("file '{q}'\n"));
        }
        if let Err(e) = std::fs::write(&list, txt) {
            self.push_log(
                tr(lang, Key::ConcatWriteFail).replace("{e}", &e.to_string()),
                true,
            );
            return;
        }
        let (mut args, desc) = if self.profile.merge_reencode {
            (
                vec![
                    "-c:v".into(), "libx264".into(), "-preset".into(), "medium".into(),
                    "-crf".into(), "23".into(), "-pix_fmt".into(), "yuv420p".into(),
                    "-c:a".into(), "aac".into(), "-b:a".into(), "192k".into(),
                    "-movflags".into(), "+faststart".into(),
                ],
                tr(lang, Key::MergeDescRe).to_string(),
            )
        } else {
            (vec!["-c".into(), "copy".into()], tr(lang, Key::MergeDescCopy).to_string())
        };
        // tek is: CPU'nun tum thread'i bu ise verilir
        args.insert(0, "-threads".into());
        args.insert(1, self.cpu.logical.max(1).to_string());
        let total: f64 = files.iter().map(|m| m.duration).sum();
        let total_size: u64 = files.iter().map(|m| m.size).sum();
        let job_input = list.clone();
        let spec = ffmpeg::JobSpec {
            job_index: 0,
            input_prefix_args: vec!["-f".into(), "concat".into(), "-safe".into(), "0".into()],
            input: list.clone(),
            input_name: format!("{} {}", files.len(), tr(lang, Key::UnitFile)),
            output: out.clone(),
            args,
            duration: total,
            target_desc: desc.clone(),
            skip,
            skip_reason: if skip { Some(tr(lang, Key::SkipExists).to_string()) } else { None },
            output_is_pattern: false,
            cleanup: Some(list),
        };
        self.jobs = vec![Job {
            name: format!("{} {} - {stem}.mp4", files.len(), tr(lang, Key::UnitFile)),
            target: desc,
            src_size: total_size,
            input: job_input,
            output: out.clone(),
            output_is_pattern: false,
            merge: true,
            frac: 0.0,
            speed: None,
            state: if skip {
                JobState::Skipped {
                    reason: tr(lang, Key::SkipExists).to_string(),
                }
            } else {
                JobState::Queued
            },
        }];
        self.push_log(
            format!(
                "[{}] {} {} {}: {}",
                tr(lang, Key::LogQueue),
                files.len(),
                tr(lang, Key::UnitFile),
                tr(lang, Key::MergeWillRun),
                out.display()
            ),
            false,
        );
        self.running = true;
        let (tx, rx) = mpsc::channel();
        self.rx = Some(rx);
        ffmpeg::start_jobs(ffmpeg_path, vec![spec], tx, 1);
    }

    fn poll_jobs(&mut self, ctx: &egui::Context) {
        // once: mesajlari al (borcu bitir), sonra state'i guncelle
        let msgs: Vec<JobMsg> = self
            .rx
            .as_ref()
            .map(|rx| {
                let mut v = Vec::new();
                while let Ok(m) = rx.try_recv() {
                    v.push(m);
                }
                v
            })
            .unwrap_or_default();
        if msgs.is_empty() {
            return;
        }
        for msg in msgs {
            match msg {
                JobMsg::Progress { idx, frac, speed } => {
                    if let Some(j) = self.jobs.get_mut(idx) {
                        if matches!(j.state, JobState::Queued) {
                            j.state = JobState::Running;
                        }
                        j.frac = frac;
                        if speed.is_some() {
                            j.speed = speed;
                        }
                    }
                }
                JobMsg::Finished { idx, ok, msg, out_size, secs } => {
                    let name = self.jobs.get(idx).map(|j| j.name.clone());
                    let lang = self.lang;
                    if let Some(j) = self.jobs.get_mut(idx) {
                        if ok {
                            let size_s = out_size.map(fmt_size).unwrap_or_default();
                            j.state = JobState::Done { secs, new_size: out_size };
                            if let Some(n) = name {
                                self.push_log(
                                    format!(
                                        "{} {n} - {size_s} ({secs:.1} {})",
                                        tr(lang, Key::LogDone),
                                        tr(lang, Key::UnitSecond)
                                    ),
                                    false,
                                );
                            }
                        } else {
                            j.state = JobState::Failed { msg: msg.clone() };
                            if let Some(n) = name {
                                self.push_log(
                                    format!("{} {n}: {msg}", tr(lang, Key::LogError)),
                                    true,
                                );
                            }
                        }
                    }
                    // basarili bittiyse kaynak dosya islemi (sil/tasi)
                    if ok {
                        self.apply_source_action(idx);
                    }
                }
                JobMsg::AllDone => {
                    self.running = false;
                    let lang = self.lang;
                    let done = self
                        .jobs
                        .iter()
                        .filter(|j| matches!(j.state, JobState::Done { .. }))
                        .count();
                    let fail = self
                        .jobs
                        .iter()
                        .filter(|j| matches!(j.state, JobState::Failed { .. }))
                        .count();
                    self.push_log(
                        format!(
                            "[{}] {}: {} {}, {} {}",
                            tr(lang, Key::LogQueue),
                            tr(lang, Key::QueueFinished),
                            done,
                            tr(lang, Key::UnitFinished),
                            fail,
                            tr(lang, Key::UnitFailed)
                        ),
                        fail > 0,
                    );
                }
            }
        }
        ctx.request_repaint();
    }

    fn push_log(&mut self, line: String, is_err: bool) {
        self.log.push((line, is_err));
        if self.log.len() > 600 {
            let drain = self.log.len() - 600;
            self.log.drain(..drain);
        }
    }

    fn out_dir_path(&self) -> PathBuf {
        match &self.out_mode {
            OutMode::Dir(d) => d.clone(),
            _ => PathBuf::new(),
        }
    }

    // ------------------------------------------------------------------
    // UI
    // ------------------------------------------------------------------

    fn top_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // logo: embed PNG (doku yoksa emoji yedegi) + mavi metin
                ui.spacing_mut().item_spacing.x = 6.0;
                let logo = self.logo_tex.as_ref().map(|t| (t.id(), t.size_vec2()));
                if let Some((id, tsz)) = logo {
                    ui.add(
                        egui::Image::from_texture((id, tsz))
                            .fit_to_exact_size(egui::vec2(22.0, 22.0)),
                    );
                } else {
                    self.emoji_at(ui, 20.0, Emoji::Clapperboard);
                }
                ui.label(
                    egui::RichText::new("FF Studio").heading().color(hex_to_color(0x60A5FA)),
                );
                ui.label(egui::RichText::new(tr(self.lang, Key::Subtitle)).weak());
                ui.separator();
                match &self.ff {
                    Ok(f) => {
                        ui.label(egui::RichText::new(format!("ffmpeg: {}", f.source)).weak());
                    }
                    Err(_) => {
                        ui.label(
                            egui::RichText::new(tr(self.lang, Key::FfmpegMissing))
                                .color(egui::Color32::from_rgb(240, 110, 110))
                                .strong(),
                        );
                    }
                }
                // sag ust yikilim (sagdan sola):
                // [ℹ️ Hakkında] (en sağ) | [🌐 🇹🇷 🇺🇸] | [⚙️ Ayarlar]
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self
                        .emoji_button(
                            ui,
                            15.0,
                            Emoji::Info,
                            tr(self.lang, Key::AboutBtn),
                            egui::TextStyle::Body,
                            None,
                            None,
                        )
                        .clicked()
                    {
                        self.about_open = true;
                    }
                    ui.separator();
                    self.emoji_at(ui, 14.0, Emoji::Globe);
                    let l = self.lang;
                    if self
                        .flag_chip(ui, 16.0, Emoji::FlagTr, FLAG_TR, l == Lang::Tr, "Türkçe")
                        .clicked()
                    {
                        self.lang = Lang::Tr;
                    }
                    if self
                        .flag_chip(ui, 16.0, Emoji::FlagUs, FLAG_US, l == Lang::En, "English")
                        .clicked()
                    {
                        self.lang = Lang::En;
                    }
                    ui.separator();
                    if self
                        .emoji_button(
                            ui,
                            15.0,
                            Emoji::Gear,
                            tr(self.lang, Key::SettingsBtn),
                            egui::TextStyle::Body,
                            None,
                            None,
                        )
                        .clicked()
                    {
                        self.settings_open = true;
                    }
                });
            });
        });
    }

    fn about_window(&mut self, ctx: &egui::Context) {
        if !self.about_open {
            return;
        }
        let lang = self.lang;
        let title = tr(lang, Key::AboutTitle).to_string();
        let ver = env!("CARGO_PKG_VERSION").to_string();
        let ffv = truncate(&self.ff_version, 48).to_string();
        let ffsrc = self
            .ff
            .as_ref()
            .map(|f| f.source.clone())
            .unwrap_or_else(|_| "-".into());
        // .open() self'i mutably borrows; closure'da kullanmak icin dokuyu simdiden cozmek
        let logo_tex = self.logo_tex.as_ref().map(|t| (t.id(), t.size_vec2()));
        egui::Window::new(&title)
            .collapsible(false)
            .open(&mut self.about_open)
            .default_size([430.0, 250.0])
            .min_size([340.0, 190.0])
            .resizable(true)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 10.0;
                    if let Some((id, tsz)) = logo_tex {
                        ui.add(
                            egui::Image::from_texture((id, tsz))
                                .fit_to_exact_size(egui::vec2(48.0, 48.0)),
                        );
                    }
                    ui.vertical_centered(|ui| {
                        ui.label(egui::RichText::new("FF Studio").heading());
                        ui.label(
                            egui::RichText::new(tr(lang, Key::Subtitle)).weak().size(11.0),
                        );
                    });
                });
                ui.add_space(4.0);
                ui.add(
                    egui::Label::new(egui::RichText::new(tr(lang, Key::AboutDesc)).weak()).wrap(),
                );
                ui.separator();
                ui.label(format!("{}: {}", tr(lang, Key::AboutBy), "Resul Çelik"));
                ui.label(format!("{}: v{ver}", tr(lang, Key::AboutVersion)));
                ui.label(format!("ffmpeg: {} ({})", ffv, ffsrc));
                ui.label(egui::RichText::new(tr(lang, Key::AboutBuilt)).weak());
                ui.add_space(4.0);
                ui.hyperlink_to(
                    "github.com/anotherphonker/ffstudio",
                    "https://github.com/anotherphonker/ffstudio",
                );
            });
    }

    fn status_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let st = if self.running {
                    tr(self.lang, Key::StRunning)
                } else if self.pending > 0 {
                    tr(self.lang, Key::StScanning)
                } else {
                    tr(self.lang, Key::StIdle)
                };
                ui.label(
                    egui::RichText::new(format!(
                        "{} {}   |   {} {}   |   {}",
                        self.files.len(),
                        tr(self.lang, Key::UnitFile),
                        self.jobs.len(),
                        tr(self.lang, Key::UnitQueue),
                        st
                    ))
                    .weak(),
                );
                if self.pending > 0 {
                    ui.spinner();
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(truncate(&self.ff_version, 80)).weak().size(10.5));
                });
            });
        });
    }

    fn left_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("files")
            .default_width(380.0)
            .min_width(220.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if self
                        .emoji_button(
                            ui,
                            15.0,
                            Emoji::Folder,
                            tr(self.lang, Key::AddFile),
                            egui::TextStyle::Button,
                            None,
                            None,
                        )
                        .clicked()
                    {
                        self.pick_files();
                    }
                    if self
                        .emoji_button(
                            ui,
                            15.0,
                            Emoji::OpenFolder,
                            tr(self.lang, Key::AddFolder),
                            egui::TextStyle::Button,
                            None,
                            None,
                        )
                        .clicked()
                    {
                        self.pick_folder();
                    }
                    // min_size(0,24) = emoji butonlariyla AYNI yukselik, hizali satir
                    if ui
                        .add_enabled(
                            !self.running,
                            egui::Button::new(tr(self.lang, Key::Clear))
                                .min_size(egui::vec2(0.0, 24.0)),
                        )
                        .clicked()
                    {
                        self.files.clear();
                        self.selected = None;
                    }
                });
                ui.add(
                    egui::Label::new(egui::RichText::new(tr(self.lang, Key::DropHint))
                        .weak()
                        .size(10.5))
                    .wrap(),
                );
                ui.separator();
                egui::ScrollArea::vertical()
                    .id_source("filelist")
                    .show(ui, |ui| {
                        let count = self.files.len();
                        for i in 0..count {
                            self.file_row(ui, i);
                        }
                        if self.files.is_empty() && self.pending == 0 {
                            ui.add_space(24.0);
                            ui.horizontal_centered(|ui| {
                                ui.label(egui::RichText::new(tr(self.lang, Key::NoFiles)).weak());
                            });
                        }
                    });
            });
    }

    fn file_row(&mut self, ui: &mut egui::Ui, i: usize) {
        let Some(m) = self.files.get(i).cloned() else {
            return;
        };
        let is_sel = self.selected == Some(i);
        let (badge, color) = match m.kind {
            Kind::Audio => ("SES", egui::Color32::from_rgb(96, 165, 250)),
            Kind::Video => ("VID", egui::Color32::from_rgb(110, 205, 130)),
            Kind::Image => ("IMG", egui::Color32::from_rgb(240, 175, 95)),
        };
        // pano genisligine gore kes: monospace 12pt bir karakter ~7.2px
        let avail = ui.available_width();
        let max_chars = ((avail / 7.2).floor() as usize).max(8);
        let name = truncate(&m.name, max_chars);
        let resp = ui
            .selectable_label(is_sel, egui::RichText::new(name.clone()).monospace().size(12.0))
            .on_hover_text(&m.name);
        if resp.clicked() {
            self.selected = if is_sel { None } else { Some(i) };
        }
        ui.horizontal(|ui| {
            ui.add_sized(
                [30.0, 13.0],
                egui::Label::new(
                    egui::RichText::new(badge)
                        .monospace()
                        .size(9.0)
                        .strong()
                        .color(color),
                ),
            );
            ui.label(
                egui::RichText::new(format!("{}   {}", m.summary(), fmt_size(m.size)))
                    .weak()
                    .size(10.5),
            );
        });
        if is_sel {
            self.details_block(ui, &m);
        }
        ui.add_space(3.0);
    }

    fn details_block(&self, ui: &mut egui::Ui, m: &Media) {
        egui::Frame::group(ui.style())
            .inner_margin(6.0)
            .show(ui, |ui| {
                    egui::Grid::new(format!("det_{}", m.path.to_string_lossy()))
                        .show(ui, |ui| {
                            let l = self.lang;
                            let mut row = |k: &str, v: String| {
                                ui.weak(k);
                                ui.label(v);
                                ui.end_row();
                            };
                            row(tr(l, Key::DContainer), m.container.clone());
                            if m.duration > 0.0 {
                                row(tr(l, Key::DDuration), fmt_dur(m.duration));
                            }
                            if let Some(k) = src_kbps(m) {
                                row(tr(l, Key::DTotalBitrate), format!("{k} kbps"));
                            }
                            if let Some(v) = &m.video {
                                let prof = if v.profile.is_empty() {
                                    String::new()
                                } else {
                                    format!(" / {}", v.profile)
                                };
                                row(
                                    tr(l, Key::DVideo),
                                    format!(
                                        "{}{}  {}x{} @ {:.0} fps  ({})",
                                        v.codec, prof, v.width, v.height, v.fps, v.pix_fmt
                                    ),
                                );
                                if let Some(b) = v.bitrate {
                                    row(tr(l, Key::DVideoBitrate), format!("{:.0} kbps", b as f64 / 1000.0));
                                }
                            }
                            if let Some(a) = &m.audio {
                                let sr = if a.sample_rate > 0 {
                                    format!("{} Hz", a.sample_rate)
                                } else {
                                    tr(l, Key::HzUnknown).to_string()
                                };
                                row(
                                    tr(l, Key::DAudio),
                                    format!(
                                        "{}  {}, {} {}",
                                        a.codec, sr, a.channels, tr(l, Key::UnitChannel)
                                    ),
                                );
                                if let Some(b) = a.bitrate {
                                    row(tr(l, Key::DAudioBitrate), format!("{:.0} kbps", b as f64 / 1000.0));
                                }
                            }
                            for e in &m.extra {
                                row("", e.clone());
                            }
                        });
            });
    }

    fn central(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                if let Err(e) = &self.ff {
                    egui::Frame::group(ui.style())
                        .fill(egui::Color32::from_rgb(64, 32, 32))
                        .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new(format!("{}: {e}", tr(self.lang, Key::FfmpegMissing)))
                                        .color(egui::Color32::from_rgb(250, 160, 160)),
                                );
                        });
                    ui.separator();
                }

                ui.group(|ui| {
                    self.emoji_heading(ui, 19.0, Emoji::Controls, tr(self.lang, Key::SecProfile));
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label(tr(self.lang, Key::LabelPreset));
                        self.preset_combo(ui);
                    });
                    ui.add_space(4.0);
                    self.profile_controls(ui);
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        let cur = self.out_dir_path();
                        ui.label(tr(self.lang, Key::LabelOutput));
                        let _ = ui.selectable_value(&mut self.out_mode, OutMode::Source, tr(self.lang, Key::OutSource));
                        if ui
                            .selectable_value(&mut self.out_mode, OutMode::Dir(cur.clone()), tr(self.lang, Key::OutOther))
                            .clicked()
                        {
                            if let Some(d) = rfd::FileDialog::new()
                                .set_title(tr(self.lang, Key::PickOutFolder))
                                .pick_folder()
                            {
                                self.out_mode = OutMode::Dir(d);
                            }
                        }
                        if let OutMode::Dir(d) = &self.out_mode {
                            ui.label(egui::RichText::new(d.display().to_string()).monospace().size(11.0));
                        }
                    });
                    // KAYNAK KLASORE modunda gecerli cikti klasorunu goruntule
                    if matches!(self.out_mode, OutMode::Source) {
                        if let Some(m) = self.files.first() {
                            if let Some(d) = m.path.parent() {
                                ui.horizontal(|ui| {
                                    ui.add_space(14.0);
                                    ui.label(
                                        egui::RichText::new(d.display().to_string())
                                            .monospace()
                                            .size(11.0)
                                            .weak(),
                                    );
                                });
                            }
                        }
                    }
                    ui.checkbox(&mut self.overwrite, tr(self.lang, Key::Overwrite));
                    self.source_action_row(ui);
                    ui.add_space(6.0);
                    if self.profile.preset == Preset::Merge {
                        if !self.files.is_empty() {
                            let mode = if self.profile.merge_reencode {
                                "-c:v libx264 -crf 23 -c:a aac -b:a 192k -movflags +faststart"
                            } else {
                                "-c copy"
                            };
                            let name = if self.profile.merge_name.trim().is_empty() {
                                "birlesmis"
                            } else {
                                self.profile.merge_name.trim()
                            };
                            let cmd = format!(
                                "ffmpeg -nostdin -loglevel error -y -f concat -safe 0 -i \"list.txt ({} {})\"  {}  -threads {}  \"{}.mp4\"",
                                self.files.len(),
                                tr(self.lang, Key::UnitFile),
                                mode,
                                self.cpu.logical.max(1),
                                name
                            );
                            ui.label(egui::RichText::new(tr(self.lang, Key::CmdPreviewMerge)).weak());
                            ui.add(egui::Label::new(egui::RichText::new(cmd).monospace().size(11.0)).wrap());
                        }
                    } else if let Some(m) = self.files.first() {
                        if let Ok(b) = profiles::build(&self.profile, m, self.lang) {
                            let stem = m
                                .path
                                .file_stem()
                                .map(|s| s.to_string_lossy().into_owned())
                                .unwrap_or_else(|| m.name.clone());
                            let pre = if b.prefix_args.is_empty() {
                                String::new()
                            } else {
                                format!("{} ", b.prefix_args.join(" "))
                            };
                            let out_name = if b.output_is_pattern {
                                format!("{stem}_parca_%03d.{}", b.ext)
                            } else {
                                format!("{stem}.{}", b.ext)
                            };
                            let (_w, t) = self.parallel_plan(self.files.len().max(1));
                            let mut pargs = b.args.clone();
                            pargs.insert(0, "-threads".into());
                            pargs.insert(1, t.to_string());
                            let cmd = format!(
                                "ffmpeg -nostdin -loglevel error -y {pre}-i \"{}\"  {}  \"{}\"",
                                m.name,
                                pargs.join(" "),
                                out_name
                            );
                            ui.label(egui::RichText::new(tr(self.lang, Key::CmdPreview)).weak());
                            ui.add(egui::Label::new(egui::RichText::new(cmd).monospace().size(11.0)).wrap());
                        }
                    }
                });
                ui.add_space(8.0);
                self.efficiency_table(ui);
                ui.add_space(8.0);
                self.queue_section(ui);
                ui.add_space(8.0);
                self.log_section(ui);
            });
        });
    }

    /// Listede en az bir dosya bu preset'i kullanabiliyormu?
    fn preset_available(&self, p: Preset) -> bool {
        if self.files.is_empty() {
            return true;
        }
        self.files.iter().any(|m| p.applies_to(m.kind))
    }

    fn preset_item(&mut self, ui: &mut egui::Ui, p: Preset) {
        let label = p.label(self.lang);
        if self.preset_available(p) {
            ui.selectable_value(&mut self.profile.preset, p, label);
        } else {
            // uygun degil: gri, tiklanamayan
            ui.add_enabled(false, egui::SelectableLabel::new(false, label));
        }
    }

    fn preset_combo(&mut self, ui: &mut egui::Ui) {
        let prev = self.profile.preset;
        egui::ComboBox::from_id_source("preset")
            .selected_text(self.profile.preset.label(self.lang))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.profile.preset, Preset::Auto, Preset::Auto.label(self.lang));
                ui.separator();
                ui.strong(tr(self.lang, Key::AudioHdr));
                ui.add_space(2.0);
                for &p in Preset::AUDIO {
                    self.preset_item(ui, p);
                }
                ui.separator();
                ui.strong(tr(self.lang, Key::VideoHdr));
                ui.add_space(2.0);
                for &p in Preset::VIDEO {
                    self.preset_item(ui, p);
                }
                ui.separator();
                ui.strong(tr(self.lang, Key::ImageHdr));
                ui.add_space(2.0);
                for &p in Preset::IMAGE {
                    self.preset_item(ui, p);
                }
                ui.separator();
                ui.strong(tr(self.lang, Key::OtherHdr));
                ui.add_space(2.0);
                self.preset_item(ui, Preset::Remux);
                self.preset_item(ui, Preset::Merge);
            });
        // secili preset, mevcut dosyalara uymuyorsa Otomatik'e don
        if !self.files.is_empty() && !self.preset_available(self.profile.preset) {
            self.profile.preset = Preset::Auto;
            self.push_log(tr(self.lang, Key::InfoAutoSwitched).to_string(), false);
        }
        if self.profile.preset != prev {
            if let Some(c) = self.profile.preset.default_crf() {
                if !self.profile.crf_touched {
                    self.profile.crf = c;
                }
            }
        }
    }

    fn profile_controls(&mut self, ui: &mut egui::Ui) {
        let preset = self.profile.preset;
        let lang = self.lang;
        let kbps = |v: u32| format!("{v} kbps");
        match preset {
            Preset::Mp3Cbr => {
                ui.horizontal(|ui| {
                    ui.label(tr(lang, Key::Bitrate));
                    combo_u32(
                        ui,
                        "mp3_cbr",
                        &mut self.profile.mp3_cbr,
                        &[
                            (320, format!("{} {}", kbps(320), tr(lang, Key::PillMax))),
                            (256, kbps(256)),
                            (192, kbps(192)),
                            (160, kbps(160)),
                            (128, kbps(128)),
                        ],
                    );
                });
            }
            Preset::Aac => {
                ui.horizontal(|ui| {
                    ui.label(tr(lang, Key::Bitrate));
                    combo_u32(
                        ui,
                        "aac_kbps",
                        &mut self.profile.aac_kbps,
                        &[
                            (0, format!("{} {}", tr(lang, Key::VbrHigh), tr(lang, Key::PillRecommended))),
                            (256, kbps(256)),
                            (192, kbps(192)),
                            (160, kbps(160)),
                            (128, kbps(128)),
                        ],
                    );
                });
            }
            Preset::Opus => {
                ui.horizontal(|ui| {
                    ui.label(tr(lang, Key::Bitrate));
                    combo_u32(
                        ui,
                        "opus_kbps",
                        &mut self.profile.opus_kbps,
                        &[
                            (320, kbps(320)),
                            (256, kbps(256)),
                            (160, kbps(160)),
                            (128, kbps(128)),
                        ],
                    );
                });
            }
            Preset::TargetSize | Preset::Clip => {
                if preset == Preset::TargetSize {
                    ui.horizontal(|ui| {
                        ui.label(tr(lang, Key::TargetSize));
                        combo_u32(
                            ui,
                            "target_mb",
                            &mut self.profile.target_mb,
                            &[(5, "5 MB".into()), (10, "10 MB".into()), (25, "25 MB".into()), (50, "50 MB".into()), (100, "100 MB".into())],
                        );
                    });
                    ui.label(
                        egui::RichText::new(tr(lang, Key::TargetNote)).weak().size(10.5),
                    );
                } else {
                    self.trim_rows(ui);
                }
                if preset == Preset::Clip {
                    ui.horizontal(|ui| {
                        ui.label(tr(lang, Key::Crf));
                        let r = ui.add(
                            egui::Slider::new(&mut self.profile.crf, 18..=36)
                                .text(tr(lang, Key::CrfText)),
                        );
                        if r.dragged() {
                            self.profile.crf_touched = true;
                        }
                    });
                }
                ui.horizontal(|ui| {
                    ui.label(tr(lang, Key::EncodeSpeed));
                    for (idx, name) in X264_PRESETS.iter().enumerate() {
                        ui.selectable_value(&mut self.profile.x264_preset, idx as u32, name.to_string());
                    }
                });
                ui.horizontal(|ui| {
                    ui.label(tr(lang, Key::AudioInVideo));
                    combo_u32(
                        ui,
                        "video_audio_kbps",
                        &mut self.profile.video_audio_kbps,
                        &[
                            (256, "AAC 256 kbps".into()),
                            (192, "AAC 192 kbps".into()),
                            (160, "AAC 160 kbps".into()),
                            (128, "AAC 128 kbps".into()),
                        ],
                    );
                });
            }
            Preset::Gif => {
                self.trim_rows(ui);
                ui.horizontal(|ui| {
                    ui.label(tr(lang, Key::Width));
                    combo_u32(
                        ui,
                        "gif_width",
                        &mut self.profile.gif_width,
                        &[(320, "320 px".into()), (480, "480 px".into()), (640, "640 px".into()), (854, "854 px".into())],
                    );
                    ui.label(tr(lang, Key::Fps));
                    combo_u32(
                        ui,
                        "gif_fps",
                        &mut self.profile.gif_fps,
                        &[(5, "5".into()), (10, "10".into()), (15, "15".into())],
                    );
                });
                ui.label(
                    egui::RichText::new(tr(lang, Key::GifNote)).weak().size(10.5),
                );
            }
            Preset::Split => {
                ui.horizontal(|ui| {
                    ui.label(tr(lang, Key::SegmentLen));
                    combo_u32(
                        ui,
                        "split_secs",
                        &mut self.profile.split_secs,
                        &[
                            (15, format!("{} {}", 15, tr(lang, Key::UnitSecond))),
                            (30, format!("{} {}", 30, tr(lang, Key::UnitSecond))),
                            (60, format!("{} {}", 1, tr(lang, Key::UnitMin))),
                            (120, format!("{} {}", 2, tr(lang, Key::UnitMin))),
                            (300, format!("{} {}", 5, tr(lang, Key::UnitMin))),
                        ],
                    );
                });
                ui.label(
                    egui::RichText::new(tr(lang, Key::SplitNote)).weak().size(10.5),
                );
            }
            Preset::Merge => {
                ui.horizontal(|ui| {
                    ui.label(tr(lang, Key::OutputName));
                    ui.add_sized([140.0, 20.0], egui::TextEdit::singleline(&mut self.profile.merge_name));
                    ui.label(".mp4");
                    ui.checkbox(&mut self.profile.merge_reencode, tr(lang, Key::Reencode));
                });
                ui.label(
                    egui::RichText::new(tr(lang, Key::MergeNote)).weak().size(10.5),
                );
            }
            p if p.is_video() => {
                ui.horizontal(|ui| {
                    ui.label(tr(lang, Key::Crf));
                    let range: std::ops::RangeInclusive<u32> = match preset {
                        Preset::Av1Mp4 => 15..=50,
                        Preset::Vp9Webm => 0..=63,
                        _ => 18..=36,
                    };
                    let r = ui.add(
                        egui::Slider::new(&mut self.profile.crf, range)
                            .text(tr(lang, Key::CrfText)),
                    );
                    if r.dragged() {
                        self.profile.crf_touched = true;
                    }
                });
                if preset == Preset::H264Mp4 || preset == Preset::H265Mp4 {
                    ui.horizontal(|ui| {
                        ui.label(tr(lang, Key::EncodeSpeed));
                        for (idx, name) in X264_PRESETS.iter().enumerate() {
                            ui.selectable_value(&mut self.profile.x264_preset, idx as u32, name.to_string());
                        }
                    });
                    ui.label(
                        egui::RichText::new(tr(lang, Key::SpeedNote)).weak().size(10.5),
                    );
                }
                ui.horizontal(|ui| {
                    ui.label(tr(lang, Key::AudioInVideo));
                    if preset == Preset::Vp9Webm {
                        ui.label(egui::RichText::new(tr(lang, Key::OpusFixed)).weak());
                    } else {
                        combo_u32(
                            ui,
                            "video_audio_kbps",
                            &mut self.profile.video_audio_kbps,
                            &[
                                (256, "AAC 256 kbps".into()),
                                (192, "AAC 192 kbps".into()),
                                (160, "AAC 160 kbps".into()),
                                (128, "AAC 128 kbps".into()),
                            ],
                        );
                    }
                });
            }
            Preset::CoverArt => {
                ui.checkbox(&mut self.profile.cover_png, tr(lang, Key::CoverPng));
                ui.label(
                    egui::RichText::new(tr(lang, Key::CoverNote)).weak().size(10.5),
                );
            }
            p if p.is_image() => {
                ui.horizontal(|ui| {
                    ui.label(tr(lang, Key::Quality));
                    ui.add(egui::Slider::new(&mut self.profile.img_quality, 5..=100).text("%"));
                });
            }
            Preset::Remux => {
                ui.horizontal(|ui| {
                    ui.label(tr(lang, Key::DContainer));
                    for (idx, (name, _)) in REMUX_CONTAINERS.iter().enumerate() {
                        ui.selectable_value(&mut self.profile.remux_container, idx as u32, name.to_string());
                    }
                    ui.label(egui::RichText::new(tr(lang, Key::RemuxNote)).weak().size(10.5));
                });
            }
            _ => {}
        }
        if preset.needs_scale() {
            ui.horizontal(|ui| {
                ui.label(tr(lang, Key::MaxWidth));
                combo_u32(
                    ui,
                    "max_width",
                    &mut self.profile.max_width,
                    &[
                        (0, tr(lang, Key::Original).to_string()),
                        (3840, "3840 (4K)".into()),
                        (1920, "1920 (Full HD)".into()),
                        (1280, "1280 (720p)".into()),
                        (854, "854 (480p)".into()),
                        (640, "640".into()),
                    ],
                );
            });
        }
        ui.horizontal(|ui| {
            ui.label(tr(lang, Key::CustomArgs));
            ui.text_edit_singleline(&mut self.profile.custom_args);
            ui.label(egui::RichText::new(tr(lang, Key::CustomArgsHint)).weak().size(10.5));
        });
    }

    /// "Kaynak dosyalar: Aynen kalsin / Cirirince sil / Tasi" satiri
    fn source_action_row(&mut self, ui: &mut egui::Ui) {
        let lang = self.lang;
        ui.horizontal(|ui| {
            ui.label(tr(lang, Key::LabelSrcAction));
            let _ = ui.selectable_value(&mut self.src_action, SrcAction::Keep, tr(lang, Key::SrcKeep));
            let _ = ui.selectable_value(&mut self.src_action, SrcAction::Delete, tr(lang, Key::SrcDelete));
            let _ = ui.selectable_value(&mut self.src_action, SrcAction::Move, tr(lang, Key::SrcMove));
            if self.src_action == SrcAction::Move {
                if let Some(d) = &self.src_move_dir {
                    ui.label(egui::RichText::new(d.display().to_string()).monospace().size(11.0));
                }
                if ui.small_button(tr(lang, Key::OutOther)).clicked() {
                    if let Some(d) = rfd::FileDialog::new()
                        .set_title(tr(lang, Key::PickSrcFolder))
                        .pick_folder()
                    {
                        self.src_move_dir = Some(d);
                    }
                }
            }
        });
        if self.src_action != SrcAction::Keep {
            ui.label(
                egui::RichText::new(tr(lang, Key::SrcNote)).weak().size(10.5),
            );
        }
    }

    /// Is basarili bitince kaynak dosyaya secilen islemi uygula
    fn apply_source_action(&mut self, idx: usize) {
        let (input, output, pattern, merge, name) = match self.jobs.get(idx) {
            Some(j) => (
                j.input.clone(),
                j.output.clone(),
                j.output_is_pattern,
                j.merge,
                j.name.clone(),
            ),
            None => return,
        };
        if merge {
            return; // birlesme: kaynaklara dokunma (tek is)
        }
        let lang = self.lang;
        match self.src_action {
            SrcAction::Keep => {}
            SrcAction::Delete => {
                if output_ok(&output, pattern) {
                    match std::fs::remove_file(&input) {
                        Ok(()) => self.push_log(
                            format!("{} {}: {name}", tr(lang, Key::LogSource), tr(lang, Key::SrcDeleted)),
                            false,
                        ),
                        Err(e) => self.push_log(
                            tr(lang, Key::SrcDelFail)
                                .replace("{name}", &name)
                                .replace("{e}", &e.to_string()),
                            true,
                        ),
                    }
                } else {
                    self.push_log(
                        format!("{} {}: {}", tr(lang, Key::LogWarning), name, tr(lang, Key::OutNotVerifiedDel)),
                        true,
                    );
                }
            }
            SrcAction::Move => {
                let dir = match &self.src_move_dir {
                    Some(d) => d.clone(),
                    None => {
                        self.push_log(
                            format!("{} {}", tr(lang, Key::LogWarning), tr(lang, Key::MoveNoDir)),
                            true,
                        );
                        return;
                    }
                };
                if output_ok(&output, pattern) {
                    match move_to_dir(&input, &dir) {
                        Ok(dst) => self.push_log(
                            format!(
                                "{} {}: {} - {}",
                                tr(lang, Key::LogSource),
                                tr(lang, Key::SrcMoved),
                                name,
                                dst.display()
                            ),
                            false,
                        ),
                        Err(e) => self.push_log(
                            tr(lang, Key::SrcMoveFail)
                                .replace("{name}", &name)
                                .replace("{e}", &e.to_string()),
                            true,
                        ),
                    }
                } else {
                    self.push_log(
                        format!("{} {}: {}", tr(lang, Key::LogWarning), name, tr(lang, Key::OutNotVerifiedMove)),
                        true,
                    );
                }
            }
        }
    }

    fn trim_rows(&mut self, ui: &mut egui::Ui) {
        let lang = self.lang;
        ui.horizontal(|ui| {
            ui.label(tr(lang, Key::TrimStart));
            ui.add_sized(
                [70.0, 20.0],
                egui::TextEdit::singleline(&mut self.profile.trim_start).hint_text("mm:ss"),
            );
            ui.label(tr(lang, Key::TrimEnd));
            ui.add_sized(
                [70.0, 20.0],
                egui::TextEdit::singleline(&mut self.profile.trim_end).hint_text(tr(lang, Key::TrimEndHint)),
            );
        });
        ui.label(
            egui::RichText::new(tr(lang, Key::TimeFormatHint)).weak().size(10.5),
        );
    }

    fn efficiency_table(&mut self, ui: &mut egui::Ui) {
        let lang = self.lang;
        ui.group(|ui| {
            self.emoji_heading(ui, 19.0, Emoji::BarChart, tr(lang, Key::SecEff));
            ui.add_space(4.0);
            if self.files.is_empty() {
                ui.label(
                    egui::RichText::new(tr(lang, Key::EffEmpty)).weak(),
                );
                return;
            }
            egui::Grid::new("effgrid")
                .striped(true)
                .min_col_width(40.0)
                .show(ui, |ui| {
                    ui.strong(tr(lang, Key::ColFile));
                    ui.strong(tr(lang, Key::ColType));
                    ui.strong(tr(lang, Key::ColSize));
                    ui.strong(tr(lang, Key::ColDur));
                    ui.strong(tr(lang, Key::ColSrcKbps));
                    ui.strong(tr(lang, Key::ColTarget));
                    ui.strong(tr(lang, Key::ColEst));
                    ui.strong(tr(lang, Key::ColDiff));
                    ui.end_row();
                    for m in &self.files {
                        let (est, desc) = profiles::estimate(&self.profile, m, lang);
                        let kbps = src_kbps(m).map(|k| k.to_string()).unwrap_or_else(|| "-".into());
                        let est_s = match est {
                            Some(e) => fmt_size(e),
                            None => {
                                if m.kind == Kind::Video {
                                    "CRF?".into()
                                } else {
                                    "-".into()
                                }
                            }
                        };
                        let diff = match est {
                            Some(e) if m.size > 0 && e != m.size => {
                                format!("{:+.0}%", (e as f64 / m.size as f64 - 1.0) * 100.0)
                            }
                            Some(_) => tr(lang, Key::CopyLabel).to_string(),
                            None => "-".to_string(),
                        };
                        let diff_color = if diff.starts_with('-') {
                            egui::Color32::from_rgb(110, 205, 130)
                        } else if diff.starts_with('+') {
                            egui::Color32::from_rgb(240, 175, 95)
                        } else {
                            egui::Color32::from_rgb(140, 140, 140)
                        };
                        let kind_s = match m.kind {
                            Kind::Audio => tr(lang, Key::TypeAudio),
                            Kind::Video => tr(lang, Key::TypeVideo),
                            Kind::Image => tr(lang, Key::TypeImage),
                        };
                        ui.label(truncate(&m.name, 20));
                        ui.label(egui::RichText::new(kind_s).weak());
                        ui.label(fmt_size(m.size));
                        ui.label(if m.duration > 0.0 { fmt_dur(m.duration) } else { "-".into() });
                        ui.label(kbps);
                        ui.label(egui::RichText::new(truncate(&desc, 26)).weak());
                        ui.label(est_s);
                        ui.label(egui::RichText::new(diff).color(diff_color));
                        ui.end_row();
                    }
                });
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(tr(lang, Key::CrfNote)).weak().size(10.5),
            );
        });
    }

    fn queue_section(&mut self, ui: &mut egui::Ui) {
        let lang = self.lang;
        ui.group(|ui| {
            ui.horizontal(|ui| {
                self.emoji_heading(ui, 19.0, Emoji::Thread, tr(lang, Key::SecQueue));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let enabled = !self.running && !self.files.is_empty() && self.ff.is_ok();
                    let text = if self.running {
                        tr(lang, Key::Running).to_string()
                    } else {
                        tr(lang, Key::ConvertAll).to_string()
                    };
                    let resp = self.emoji_button(
                        ui,
                        18.0,
                        Emoji::Rocket,
                        &text,
                        egui::TextStyle::Button,
                        if enabled { Some(hex_to_color(self.accent)) } else { None },
                        if enabled { Some(egui::Color32::WHITE) } else { None },
                    );
                    if enabled && resp.clicked() {
                        self.start_conversion();
                    }
                });
            });
            ui.add_space(4.0);
            if self.jobs.is_empty() {
                ui.label(
                    egui::RichText::new(tr(lang, Key::NoJobs)).weak(),
                );
                return;
            }
            // CANLI KUYRUK DURUMU: "N calisiyor / N sirada / N bitti / N hata"
            // (kuyruk devamlidir: bir sarki bitince siradakinden biri hemen
            // baslar, tum partinin bitmesi beklenmez)
            let (mut run, mut que, mut don, mut bad) = (0usize, 0, 0, 0);
            for j in &self.jobs {
                match &j.state {
                    JobState::Running => run += 1,
                    JobState::Queued => que += 1,
                    JobState::Done { .. } => don += 1,
                    JobState::Failed { .. } => bad += 1,
                    JobState::Skipped { .. } => {}
                }
            }
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                ui.label(
                    egui::RichText::new(format!("{run} {}", tr(lang, Key::WordRunning)))
                        .color(egui::Color32::from_rgb(110, 205, 130))
                        .size(11.0),
                );
                ui.label(
                    egui::RichText::new(format!("{que} {}", tr(lang, Key::Queued)))
                        .weak()
                        .size(11.0),
                );
                ui.label(
                    egui::RichText::new(format!("{don} {}", tr(lang, Key::Done)))
                        .color(egui::Color32::from_rgb(110, 205, 130))
                        .size(11.0),
                );
                if bad > 0 {
                    ui.label(
                        egui::RichText::new(format!("{bad} {}", tr(lang, Key::WordError)))
                            .color(egui::Color32::from_rgb(240, 110, 110))
                            .size(11.0),
                    );
                }
            });
            ui.add_space(3.0);
            for (i, j) in self.jobs.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(format!("#{:2} ", i + 1));
                    ui.label(egui::RichText::new(truncate(&j.name, 26)).monospace().size(11.5));
                    ui.label(egui::RichText::new(j.target.clone()).weak().size(10.5));
                    match &j.state {
                        JobState::Queued => {
                            ui.label(egui::RichText::new(tr(lang, Key::Queued)).weak());
                        }
                        JobState::Running => {
                            ui.add_sized(
                                [150.0, 14.0],
                                egui::ProgressBar::new(j.frac).text(format!("{:.0}%", j.frac * 100.0)),
                            );
                            if let Some(s) = &j.speed {
                                ui.label(egui::RichText::new(s.clone()).weak().size(10.5));
                            }
                        }
                        JobState::Done { secs, new_size } => {
                            ui.label(
                                egui::RichText::new(tr(lang, Key::Done))
                                    .color(egui::Color32::from_rgb(110, 205, 130)),
                            );
                            if let Some(n) = new_size {
                                ui.label(
                                    egui::RichText::new(format!(
                                        "{} -> {} ({secs:.1} {})",
                                        fmt_size(j.src_size),
                                        fmt_size(*n),
                                        tr(lang, Key::UnitSecond)
                                    ))
                                    .weak()
                                    .size(10.5),
                                );
                            }
                        }
                        JobState::Skipped { reason } => {
                            ui.label(
                                egui::RichText::new(format!("{}: {reason}", tr(lang, Key::SkippedPrefix)))
                                    .weak()
                                    .size(10.5),
                            );
                        }
                        JobState::Failed { msg } => {
                            ui.label(
                                egui::RichText::new(format!("{} {}", tr(lang, Key::ErrorPrefix), truncate(msg, 60)))
                                    .color(egui::Color32::from_rgb(240, 110, 110))
                                    .size(11.0),
                            );
                        }
                    }
                });
            }
        });
    }

    fn log_section(&mut self, ui: &mut egui::Ui) {
        let lang = self.lang;
        ui.group(|ui| {
            ui.horizontal(|ui| {
                self.emoji_heading(ui, 19.0, Emoji::Scroll, tr(lang, Key::LogHdr));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button(tr(lang, Key::Clear)).clicked() {
                        self.log.clear();
                    }
                });
            });
            egui::ScrollArea::vertical()
                .id_source("log")
                .max_height(170.0)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for (line, is_err) in &self.log {
                        ui.label(if *is_err {
                            egui::RichText::new(line.clone())
                                .monospace()
                                .size(11.0)
                                .color(egui::Color32::from_rgb(240, 110, 110))
                        } else {
                            egui::RichText::new(line.clone()).monospace().size(11.0).weak()
                        });
                    }
                });
        });
    }

    fn settings_window(&mut self, ctx: &egui::Context) {
        if !self.settings_open {
            return;
        }
        // Windows guzu gibi: solda kisiltma oku YOK (.collapsible(false)),
        // sag uste CARPISI var (.open). .open() self alanini borrows ettigi
        // icin acilis durumu lokal degiskende tutulur; pencere kapanirsa
        // flag geri yazilir.
        let mut open = true;
        egui::Window::new(tr(self.lang, Key::SettingsTitle))
            .collapsible(false)
            .open(&mut open)
            .default_size([440.0, 330.0])
            .min_size([380.0, 260.0])
            .resizable(true)
            .show(ctx, |ui| {
                self.settings_controls(ui, ctx);
            });
        if !open {
            self.settings_open = false;
        }
    }

    /// Ayarlar: islemci/paralellik + dil + tema (eski Tema penceresi buraya alindi)
    fn settings_controls(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let lang = self.lang;
        ui.scope(|ui| {
            ui.add_space(4.0);
            // --- islemci / paralellik ---
            ui.strong(tr(lang, Key::SecCpu));
            let n = self.cpu.logical.to_string();
            let cores = match self.cpu.physical {
                Some(p) => tr(lang, Key::CpuCores)
                    .replace("{p}", &p.to_string())
                    .replace("{n}", &n),
                None => tr(lang, Key::CpuThreadsOnly).replace("{n}", &n),
            };
            // Model okunamadiysa bos satir kalmasin: yalnizca thread bilgisi
            let cpu_line = if self.cpu.model.is_empty() {
                cores
            } else {
                format!("{} — {}", self.cpu.model, cores)
            };
            ui.add(
                egui::Label::new(format!("{}: {}", tr(lang, Key::CpuWord), cpu_line)).wrap(),
            );
            let auto_n = self.cpu.auto_workers();
            ui.horizontal_wrapped(|ui| {
                ui.label(tr(lang, Key::CpuParallel));
                for (v, s) in [
                    (0u32, format!("{} ({})", tr(lang, Key::AutoWord), auto_n)),
                    (1, "1".to_string()),
                    (2, "2".to_string()),
                    (3, "3".to_string()),
                    (4, "4".to_string()),
                    (6, "6".to_string()),
                    (8, "8".to_string()),
                    (12, "12".to_string()),
                    (16, "16".to_string()),
                ] {
                    ui.selectable_value(&mut self.workers, v, s);
                }
            });
            let (w, t) = self.parallel_plan(self.files.len().max(1));
            ui.label(
                egui::RichText::new(
                    tr(lang, Key::CpuPlan)
                        .replace("{w}", &w.to_string())
                        .replace("{t}", &t.to_string()),
                )
                .weak()
                .size(10.5),
            );
            ui.separator();
            // --- dil ---
            ui.horizontal(|ui| {
                ui.label(tr(lang, Key::Language));
                let _ = ui.selectable_value(&mut self.lang, Lang::Tr, "Türkçe");
                let _ = ui.selectable_value(&mut self.lang, Lang::En, "English");
            });
            ui.separator();
            // --- tema ---
            self.theme_controls(ui, ctx);
        });
    }

    fn theme_controls(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let lang = self.lang;
        ui.scope(|ui| {
            ui.add_space(4.0);
            let prev_dark = self.dark;
            ui.horizontal(|ui| {
                ui.label(tr(lang, Key::Appearance));
                let _ = ui.selectable_value(&mut self.dark, true, tr(lang, Key::Dark));
                let _ = ui.selectable_value(&mut self.dark, false, tr(lang, Key::Light));
            });
            if self.dark != prev_dark {
                self.apply_theme(ctx);
            }
            // yazi boyutu / dpi
            // ESKI BUG (1): her surukleme tikinde set_zoom_factor cagrilirdi;
            // zoom factor TUM uygulamanin pixels-per-point'ini degistirdigi
            // icin her tikte ekran yeniden olceklendi/flasladi. Simdi yalnizca
            // sliyer BIRAKILINCA bir kez uygulanir.
            // ESKI BUG (2): .show_value varsayilan true'dur — ham deger (1.25)
            // ve .text() etiketi (125%) yan yana gosterilirdi. show_value(false).
            ui.horizontal(|ui| {
                ui.label(tr(lang, Key::ZoomLabel));
                let ztext = format!("{:.0}%", self.zoom * 100.0);
                let resp = ui.add(
                    egui::Slider::new(&mut self.zoom, 0.75..=2.0)
                        .step_by(0.05)
                        .show_value(false)
                        .text(ztext),
                );
                if resp.changed() {
                    if resp.dragged() {
                        self.zoom_dirty = true; // surukluyor: birakincaya kadar bekle
                    } else {
                        ctx.set_zoom_factor(self.zoom); // tik: aninda
                    }
                }
                if self.zoom_dirty && !resp.dragged() {
                    // onceki kare surukluyordu, bu kare bitti -> TEK kez uygula
                    ctx.set_zoom_factor(self.zoom);
                    self.zoom_dirty = false;
                }
            });
            // Vurgu rengi: 1. satir = swatch'lar (dar alanda wrap olur),
            // 2. satir = RGB sliyerleri + hex girisi. Eski tek cizgi 440px
            // eninde siziyordu / kirma-sikisigi "karmasik" gorunum.
            ui.horizontal_wrapped(|ui| {
                ui.label(tr(lang, Key::Accent));
                let prev_accent = self.accent;
                for (i, (_name, hex)) in SWATCHES.iter().enumerate() {
                    let col = hex_to_color(*hex);
                    let stroke = if self.accent == *hex {
                        egui::Stroke::new(2.0_f32, egui::Color32::WHITE)
                    } else {
                        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(90, 95, 110))
                    };
                    let resp = ui
                        .add(
                            egui::Button::new("").fill(col).min_size(egui::vec2(22.0, 22.0)).stroke(stroke),
                        )
                        .on_hover_text(lang::swatch_name(self.lang, i));
                    if resp.clicked() {
                        self.accent = *hex;
                    }
                }
                if self.accent != prev_accent {
                    self.hex_buf = format!("{:06x}", self.accent);
                    self.apply_theme(ctx);
                }
            });
            ui.horizontal(|ui| {
                let c0 = hex_to_color(self.accent);
                let mut r = c0.r();
                let mut g = c0.g();
                let mut b = c0.b();
                let mut changed = false;
                for (label, ch) in [("R", &mut r), ("G", &mut g), ("B", &mut b)] {
                    if ui.add(egui::Slider::new(ch, 0..=255).text(label)).dragged() {
                        changed = true;
                    }
                }
                if changed {
                    self.accent = (r as u32) << 16 | (g as u32) << 8 | b as u32;
                    self.hex_buf = format!("{:06x}", self.accent);
                    self.apply_theme(ctx);
                }
                ui.add_sized([64.0, 20.0], egui::TextEdit::singleline(&mut self.hex_buf));
                if ui.button("OK").clicked() {
                    if let Ok(h) = u32::from_str_radix(self.hex_buf.trim().trim_start_matches('#'), 16) {
                        self.accent = h;
                        self.hex_buf = format!("{:06x}", h);
                        self.apply_theme(ctx);
                    }
                }
            });
            if ui.small_button(tr(lang, Key::ResetColor)).clicked() {
                self.accent = DEFAULT_ACCENT;
                self.hex_buf = format!("{:06x}", DEFAULT_ACCENT);
                self.apply_theme(ctx);
            }
        });
    }

    fn apply_theme(&self, ctx: &egui::Context) {
        ctx.set_visuals(build_visuals(self.dark, hex_to_color(self.accent)));
    }

    /// Paralel calisma plani: (worker sayisi, is basina thread sayisi).
    /// - worker: otomatik ise CPU'nun mantiksal thread'inin yarisi (2..=8),
    ///   manuel ise secilen deger; ikisi de calisacak is sayisiyla sinirlanir
    /// - thread: mantiksal thread'ler worker'lara esit dagitilir
    /// Tek is varsa CPU'nun tumu o ise verilir.
    pub fn parallel_plan(&self, n_jobs: usize) -> (usize, u32) {
        // Mantik core'da: TUI istemcisi de ayni fonksiyonu cagirir.
        cpu::plan(&self.cpu, self.workers, n_jobs)
    }
}

fn hex_to_color(h: u32) -> egui::Color32 {
    egui::Color32::from_rgb(((h >> 16) & 255) as u8, ((h >> 8) & 255) as u8, (h & 255) as u8)
}

// ---------------------------------------------------------------------------
// Renkli emoji (Twemoji PNG, exe'ye gomulu) — cizim yardimcilari
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// App logosu (embed PNG): pencere/taskbar ikonu + ust panel + Hakkinda
// ---------------------------------------------------------------------------
const LOGO_PNG: &[u8] = include_bytes!("../assets/logo.png");

/// Logoyu RGBA'ya coz — TAM boyut, doku icin (her zoom'da keskin).
fn logo_rgba() -> Option<(Vec<u8>, u32, u32)> {
    let img = image::load_from_memory(LOGO_PNG).ok()?.to_rgba8();
    Some((img.as_raw().to_vec(), img.width(), img.height()))
}

/// Pencere/taskbar ikonu: 256x256'ya kucult. (IconData boyutlarin
/// 4'un katisi olmasini ister; orijinal 1254 degil.)
fn logo_icon() -> Option<egui::IconData> {
    let img = image::load_from_memory(LOGO_PNG).ok()?.to_rgba8();
    let small = image::imageops::resize(&img, 256, 256, image::imageops::FilterType::Lanczos3);
    Some(egui::IconData {
        rgba: small.as_raw().to_vec(),
        width: small.width(),
        height: small.height(),
    })
}

const FLAG_TR: u32 = 0xE30A17; // Turkice cipli halka rengi
const FLAG_US: u32 = 0x3C3B6E; // Ingilizce cipli halka rengi

impl App {
    /// Emoji dokusunu cizer (doku yoksa yalnizca alani ayirir)
    fn emoji_at(&self, ui: &mut egui::Ui, size: f32, e: Emoji) {
        if let Some(h) = self.emojis.handle(e) {
            let img = egui::Image::from_texture((h.id(), h.size_vec2()))
                .fit_to_exact_size(egui::vec2(size, size));
            ui.add(img);
        } else {
            ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
        }
    }

    /// Flat "emoji + metin" butonu.
    /// egui 0.28'in buton etiketi tekil metin/gorsel tasidigi icin gorsel ve
    /// metin elle yerlestirilir; dolgular tema visuals'inden alinir.
    /// bg/fg override verilirse (orn. vurgu renkli donusturme butonu) onlar kazanir.
    fn emoji_button(
        &self,
        ui: &mut egui::Ui,
        size: f32,
        e: Emoji,
        text: &str,
        style: egui::TextStyle,
        bg_override: Option<egui::Color32>,
        fg_override: Option<egui::Color32>,
    ) -> egui::Response {
        let font_id = style.resolve(ui.style());
        let galley = ui
            .ctx()
            .fonts(|f| f.layout_no_wrap(text.to_string(), font_id, egui::Color32::WHITE));
        // Yukselik, egui'nin normal (kucuk olmayan) butonuyla AYNI olsun ki
        // ayni satirdaki native buttonlarla hizasizlik ("buggy gorunum") olmasin:
        // 14-15pt metin / 15px ikon + 3px dikey marj *2 = 24px.
        let margin_x = 8.0;
        let margin_y = 3.0;
        let gap = 5.0;
        let min_h = 18.0;
        let total = egui::vec2(
            galley.size().x + size + gap + 2.0 * margin_x,
            galley.size().y.max(size).max(min_h) + 2.0 * margin_y,
        );
        let (rect, resp) = ui.allocate_exact_size(total, egui::Sense::click());
        let hovered = resp.hovered();
        let w = &ui.visuals().widgets;
        let (bg, stroke, fg) = if let (Some(b), Some(f)) = (bg_override, fg_override) {
            (
                b,
                egui::Stroke::new(if hovered { 1.5_f32 } else { 1.0_f32 }, b),
                f,
            )
        } else if hovered {
            (w.hovered.bg_fill, w.hovered.bg_stroke, w.hovered.fg_stroke.color)
        } else {
            (w.inactive.bg_fill, w.inactive.bg_stroke, w.inactive.fg_stroke.color)
        };
        ui.painter().rect_filled(rect, 4.0, bg);
        ui.painter().rect_stroke(rect, 4.0, stroke);
        let cy = rect.center().y;
        let mut x = rect.left() + margin_x;
        if let Some(h) = self.emojis.handle(e) {
            // painter ile ciz: ui.put() layout imlecini gorsel kenarina tasir
            // ve sonraki buton bir oncekini orterdi!
            let img_rect = egui::Rect::from_center_size(
                egui::pos2(x + size / 2.0, cy),
                egui::vec2(size, size),
            );
            ui.painter().image(
                h.id(),
                img_rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
        }
        x += size + gap;
        ui.painter().galley_with_override_text_color(
            egui::pos2(x, cy - galley.size().y / 2.0),
            galley,
            fg,
        );
        resp
    }

    /// Baslik: renkli emoji + baslik metni
    fn emoji_heading(&self, ui: &mut egui::Ui, size: f32, e: Emoji, text: &str) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            self.emoji_at(ui, size, e);
            ui.label(egui::RichText::new(text).heading());
        });
    }

    /// Ulke bayragi cipli: gozumle bayrak + seciliyken dolgu ve rengi halka
    fn flag_chip(
        &self,
        ui: &mut egui::Ui,
        size: f32,
        e: Emoji,
        ring: u32,
        selected: bool,
        tip: &str,
    ) -> egui::Response {
        let m = 3.0;
        let (rect, resp) =
            ui.allocate_exact_size(egui::vec2(size + 2.0 * m, size + 2.0 * m), egui::Sense::click());
        if selected {
            ui.painter().rect_filled(rect, 4.0, ui.visuals().widgets.hovered.bg_fill);
        }
        let stroke = if selected {
            egui::Stroke::new(1.5_f32, hex_to_color(ring))
        } else {
            egui::Stroke::new(1.0_f32, ui.visuals().widgets.inactive.bg_stroke.color)
        };
        ui.painter().rect_stroke(rect, 4.0, stroke);
        if let Some(h) = self.emojis.handle(e) {
            // painter ile ciz (ui.put imleci tasirir — ayni sebep, bak emoji_button)
            ui.painter().image(
                h.id(),
                egui::Rect::from_center_size(rect.center(), egui::vec2(size, size)),
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
        }
        let resp = resp.on_hover_text(tip);
        resp
    }
}

fn build_visuals(dark: bool, accent: egui::Color32) -> egui::Visuals {
    let a = [accent.r() as f32, accent.g() as f32, accent.b() as f32];
    let mut v = if dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    let tint = |t: f32, base: [f32; 3]| -> egui::Color32 {
        egui::Color32::from_rgb(
            (base[0] * (1.0 - t) + a[0] * t) as u8,
            (base[1] * (1.0 - t) + a[1] * t) as u8,
            (base[2] * (1.0 - t) + a[2] * t) as u8,
        )
    };
    v.hyperlink_color = if dark { accent } else { tint(0.15, [30.0, 35.0, 60.0]) };
    if dark {
        let black = [0.0f32; 3];
        v.panel_fill = tint(0.13, black);
        v.window_fill = tint(0.20, black);
        v.extreme_bg_color = tint(0.08, black);
        v.faint_bg_color = tint(0.10, black);
        v.code_bg_color = tint(0.08, black);
        // NOT: egui 0.28'de Button/SelectableLabel dolgusunu weak_bg_fill'den
        // alir — ikisini birlikte boyamazsan butonlar gri kalir.
        v.widgets.noninteractive.bg_fill = tint(0.06, black);
        v.widgets.noninteractive.weak_bg_fill = tint(0.06, black);
        v.widgets.inactive.bg_fill = tint(0.18, black);
        v.widgets.inactive.weak_bg_fill = tint(0.18, black);
        v.widgets.inactive.bg_stroke = egui::Stroke::new(1.0_f32, tint(0.35, black));
        v.widgets.inactive.fg_stroke.color = tint(0.30, [135.0, 138.0, 148.0]);
        v.widgets.hovered.bg_fill = tint(0.30, black);
        v.widgets.hovered.weak_bg_fill = tint(0.30, black);
        v.widgets.hovered.bg_stroke = egui::Stroke::new(1.5_f32, tint(0.55, black));
        v.widgets.active.bg_fill = tint(0.42, black);
        v.widgets.active.weak_bg_fill = tint(0.42, black);
        v.widgets.active.bg_stroke = egui::Stroke::new(1.5_f32, accent);
        v.selection.bg_fill = tint(0.32, black);
        v.selection.stroke = egui::Stroke::new(1.0_f32, accent);
    } else {
        let white = [255.0f32; 3];
        v.panel_fill = tint(0.05, white);
        v.window_fill = tint(0.08, white);
        v.extreme_bg_color = tint(0.12, white);
        v.faint_bg_color = tint(0.09, white);
        v.code_bg_color = tint(0.10, white);
        v.widgets.noninteractive.bg_fill = tint(0.16, white);
        v.widgets.noninteractive.weak_bg_fill = tint(0.16, white);
        v.widgets.inactive.bg_fill = tint(0.14, white);
        v.widgets.inactive.weak_bg_fill = tint(0.14, white);
        v.widgets.inactive.bg_stroke = egui::Stroke::new(1.0_f32, tint(0.25, white));
        v.widgets.hovered.bg_fill = tint(0.25, white);
        v.widgets.hovered.weak_bg_fill = tint(0.25, white);
        v.widgets.hovered.bg_stroke = egui::Stroke::new(1.5_f32, tint(0.50, white));
        v.widgets.active.bg_fill = tint(0.35, white);
        v.widgets.active.weak_bg_fill = tint(0.35, white);
        v.widgets.active.bg_stroke = egui::Stroke::new(1.5_f32, accent);
        v.selection.bg_fill = tint(0.30, white);
        v.selection.stroke = egui::Stroke::new(1.0_f32, accent);
    }
    v
}

// ---------------------------------------------------------------------------
// Yardımcılar
// ---------------------------------------------------------------------------




    // Küçük seçenek listeleri: dropdown yerine "hapi" (pill) dizisi.
    // Popup'lı ComboBox çift tıkla/kaydırmada anında kapanabiliyordu;
    // pill'lerde böyle bir sorun yok.
    fn combo_u32(ui: &mut egui::Ui, _id: &str, val: &mut u32, opts: &[(u32, String)]) {
        for (v, s) in opts {
            ui.selectable_value(val, *v, s.clone());
        }
    }

// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Embed logo PNG'i decode edilebilmeli; doku tam boyut, ikon 256x256
    /// (IconData boyutlarin 4'un katisi olmasini ister).
    #[test]
    fn logo_rgba_decode() {
        let (rgba, w, h) = logo_rgba().expect("logo.png decode edilemedi");
        assert!(!rgba.is_empty());
        assert_eq!(rgba.len(), (w * h * 4) as usize, "RGBA boyutu tutmali");
        assert!(w >= 64 && h >= 64, "doku cok kucuk olmamali");
        let icon = logo_icon().expect("ikon hazirlanamadi");
        assert_eq!((icon.width, icon.height), (256, 256), "ikon 256x256 olmali");
        assert_eq!(icon.rgba.len(), 256 * 256 * 4, "ikon RGBA boyutu tutmali");
    }

    /// Zoom sliyeri suruklenirken (zoom_dirty) UI yine de sorunsuz cerceve uretmeli.
    #[test]
    fn ui_smoke_zoom_dirty_cizim() {
        let ctx = egui::Context::default();
        let mut app = App::test_new(Lang::Tr, true);
        app.emojis.init(&ctx);
        app.apply_theme(&ctx);
        ctx.set_zoom_factor(app.zoom);
        app.zoom_dirty = true; // surukleme ortasindaymis gibi
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            app.draw(ctx);
        });
        // cerceve uretildi; zoom_dirty yeni alanda panik yok
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            app.draw(ctx);
        });
    }

    /// Headless smoke test: tum UI'yi her dil x tema kombinasyonunda cerceve cerceve cizer.
    /// Emoji dokulari (Twemoji PNG), panel/pencere yapisı, dokulu buton/chipler
    /// calisma zamaninda patlarsa bu test duser.
    #[test]
    fn ui_smoke_diller_ve_temalar() {
        let presets = [
            Preset::Auto,
            Preset::Mp3Cbr,
            Preset::Aac,
            Preset::Opus,
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
        for lang in [Lang::Tr, Lang::En] {
            for dark in [true, false] {
                let ctx = egui::Context::default();
                let mut app = App::test_new(lang, dark);
                app.emojis.init(&ctx);
                app.apply_theme(&ctx);
                ctx.set_zoom_factor(app.zoom);
                for (i, preset) in presets.iter().enumerate() {
                    app.profile.preset = *preset;
                    app.profile.cover_png = i % 2 == 0;
                    app.selected = Some(i % 3); // detay paneli her dosya turu ile
                    let out = ctx.run(egui::RawInput::default(), |ctx| {
                        app.draw(ctx);
                    });
                    // cerceve uretildi; UI hicbir widget'ta panik yapmadi
                    let _ = out;
                }
                // Ayarlar'da manuel worker secili iken de cizilmeli
                app.workers = 4;
                let _ = ctx.run(egui::RawInput::default(), |ctx| {
                    app.draw(ctx);
                });
                app.workers = 0;
                // dil gecisi calisma sirasinda da cizilmeli
                app.lang = if lang == Lang::Tr { Lang::En } else { Lang::Tr };
                let _ = ctx.run(egui::RawInput::default(), |ctx| {
                    app.draw(ctx);
                });
            }
        }
    }




    #[test]
    fn parallel_plan_otomatik_ve_manuel() {
        // test_new: sabit 8T CPU
        let mut app = App::test_new(Lang::Tr, true);
        // otomatik: 8/2 = 4 worker, thread = 8/4 = 2
        assert_eq!(app.parallel_plan(10), (4, 2));
        // 2 is: 2 worker
        assert_eq!(app.parallel_plan(2), (2, 4));
        // tek is: 1 worker, CPU'nun tumu
        assert_eq!(app.parallel_plan(1), (1, 8));
        // manuel 2
        app.workers = 2;
        assert_eq!(app.parallel_plan(10), (2, 4));
        // manuel 16 ama 3 is var: worker 3'e sinirlanir
        app.workers = 16;
        assert_eq!(app.parallel_plan(3), (3, 2));
    }

    #[test]
    fn cover_art_build() {
        let mut p = Profile::default();
        p.preset = Preset::CoverArt;
        // ses + kapak: JPEG varsayilan
        let m = ffmpeg::testutil::audio_media("a.flac", 44.0, true);
        let b = profiles::build(&p, &m, Lang::Tr).unwrap();
        assert!(b.args.contains(&"-map".to_string()));
        assert!(b.args.contains(&"0:v:0".to_string()));
        assert_eq!(b.ext, "jpg");
        // png secilince
        p.cover_png = true;
        let b = profiles::build(&p, &m, Lang::Tr).unwrap();
        assert_eq!(b.ext, "png");
        assert!(b.args.contains(&"png".to_string()));
        // kapak OLMAZSA ses: hata
        p.cover_png = false;
        let m2 = ffmpeg::testutil::audio_media("b.mp3", 44.0, false);
        assert!(profiles::build(&p, &m2, Lang::Tr).is_err());
        // video: kapse gerek yok, ilk kare alinir
        let m3 = ffmpeg::testutil::video_media("c.mkv", 10.0);
        let b = profiles::build(&p, &m3, Lang::Tr).unwrap();
        assert_eq!(b.ext, "jpg");
    }


}

pub fn run_desktop() -> eframe::Result<()> {
    // Taskbar/Alt-Tab ikonu = ayni embed logo
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1360.0, 840.0])
        .with_min_inner_size([1080.0, 640.0])
        .with_title("FF Studio — all-in-one ffmpeg");
    if let Some(icon) = logo_icon() {
        viewport = viewport.with_icon(icon);
    }
    let opts = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        "FF Studio",
        opts,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(App::new(cc)))
        }),
    )
}

