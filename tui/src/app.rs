//! TUI uygulama durumu ve eylemleri.
//!
//! Tum agir isler `ffstudio_core`'dan gelir (ffmpeg cagrilari, preset/arguman
//! uretimi, CPU/paralellik plani, ceviriler). Burada yalnizca TUI'ye ozgu
//! durum ve akis vardir.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};

use ffstudio_core::cpu::{self, CpuInfo};
use ffstudio_core::ffmpeg::{self, Ffmpeg, JobMsg, JobSpec, Media};
use ffstudio_core::lang::{tr, Key, Lang};
use ffstudio_core::profiles::{self, Profile};
use ffstudio_core::util;

use crate::browse::{Browse, PickMode, PickResult};
use crate::config::{OutMode, Settings, SrcAction};

pub enum Mode {
    Main,
    Browse,
    Preset,
    Settings,
    Help,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Files,
    Profile,
    Details,
    Queue,
    Log,
}

impl Focus {
    pub fn next(self) -> Self {
        match self {
            Focus::Files => Focus::Profile,
            Focus::Profile => Focus::Details,
            Focus::Details => Focus::Queue,
            Focus::Queue => Focus::Log,
            Focus::Log => Focus::Files,
        }
    }
    pub fn prev(self) -> Self {
        match self {
            Focus::Files => Focus::Log,
            Focus::Profile => Focus::Files,
            Focus::Details => Focus::Profile,
            Focus::Queue => Focus::Details,
            Focus::Log => Focus::Queue,
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum JobState {
    Queued,
    Running,
    Done,
    Skipped(String),
    Failed(String),
}

pub struct JobRow {
    pub name: String,
    pub target: String,
    pub src_size: u64,
    pub frac: f32,
    pub speed: Option<String>,
    pub state: JobState,
}

/// Metin girisi gerektiren alanlar.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum InputTarget {
    CustomArgs,
    TrimStart,
    TrimEnd,
    MergeName,
}

pub struct InputState {
    pub target: InputTarget,
    pub buf: String,
}

pub struct App {
    pub lang: Lang,
    pub settings: Settings,
    pub cpu: CpuInfo,
    pub ff: Result<Ffmpeg, String>,
    pub ff_version: String,

    pub files: Vec<Media>,
    pub sel: usize,
    /// Siralı tarama kuyrugu: her turda bir dosya ffprobe ile cozulur
    pub pending_probe: VecDeque<PathBuf>,
    pub probed_total: usize,

    pub profile: Profile,

    pub jobs: Vec<JobRow>,
    pub rx: Option<Receiver<JobMsg>>,
    pub running: bool,
    pub log: Vec<(String, bool)>,

    /// NO_COLOR ortam degiskeni doluysa renkler kapanir (standart konvansiyon).
    pub no_color: bool,
    /// Termux depolama izni yoksa net Turkce mesaj; varsa `None`.
    /// Bu alan doluyken arayuz yerine hata ekrani cizilir.
    pub storage_error: Option<String>,
    pub mode: Mode,
    pub focus: Focus,
    pub browse: Option<Browse>,
    pub preset_sel: usize,
    pub settings_sel: usize,
    pub input: Option<InputState>,
    pub quit_confirm: bool,
    pub theme: crate::config::Theme,
    /// Kuyruk baslatildiginda kullanilan spec'ler (kaynak dosya islemleri icin)
    pub started: Vec<JobSpec>,
    /// Terminal, TAM arayuzun sigdigi en kucuk boyuttan (FIT_COLS x FIT_ROWS)
    /// kucukse "zoom out yap" uyarisi gosterilir; kullanici 'u' ile yok
    /// saydiysa bu alan true olur (sikisik da olsa paneller gizlenmez).
    /// Her resize/pinch-zoom'da sifirlanir -> uyari yeniden gorunur.
    pub fit_ack: bool,
}

impl App {
    pub fn new(settings: Settings) -> Self {
        let lang = settings.lang();
        let ff = Ffmpeg::locate(lang).map_err(|e| e.to_string());
        let ff_version = ff
            .as_ref()
            .ok()
            .map(|f| f.version_line())
            .unwrap_or_default();
        let theme = settings.theme;
        let mut app = App {
            lang,
            settings,
            cpu: CpuInfo::detect(),
            ff,
            ff_version,
            files: Vec::new(),
            sel: 0,
            pending_probe: VecDeque::new(),
            probed_total: 0,
            profile: Profile::default(),
            jobs: Vec::new(),
            rx: None,
            running: false,
            log: Vec::new(),
            no_color: std::env::var_os("NO_COLOR")
                .map(|v| !v.is_empty())
                .unwrap_or(false),
            storage_error: crate::storage::health(lang),
            mode: Mode::Main,
            focus: Focus::Files,
            browse: None,
            preset_sel: 0,
            settings_sel: 0,
            input: None,
            quit_confirm: false,
            theme,
            started: Vec::new(),
            fit_ack: false,
        };
        match &app.ff {
            Ok(f) => {
                let src = f.source.clone();
                app.push_log(
                    format!("{} ffmpeg bulundu: {}", tr(lang, Key::LogOk), src),
                    false,
                );
            }
            Err(e) => {
                let e = e.clone();
                app.push_log(format!("[Hata] {e}"), true);
            }
        }
        app
    }

    // ------------------------------------------------------------------
    // Log
    // ------------------------------------------------------------------

    pub fn push_log(&mut self, line: String, is_err: bool) {
        self.log.push((line, is_err));
        if self.log.len() > 600 {
            let n = self.log.len() - 600;
            self.log.drain(..n);
        }
    }

    // ------------------------------------------------------------------
    // Dosya ekleme (gezici -> tarama kuyrugu -> ffprobe)
    // ------------------------------------------------------------------

    /// Yol ekle: klasorse alt klasorler dahil medya dosyalarini kuyruga alir,
    /// dosyaysa dogrudan kuyruga alir.
    pub fn add_path(&mut self, p: &Path) {
        if p.is_dir() {
            // okunamayan klasor: ham OS hatasi degil, dostu mesaj
            if let Err(e) = std::fs::read_dir(p) {
                let m = crate::errors::dir_error(p, &e, self.lang);
                self.push_log(m, true);
                return;
            }
            let mut found = Vec::new();
            util::walk(p, &mut found);
            self.push_log(
                format!(
                    "{} {} ({} {})",
                    tr(self.lang, Key::LogFolder),
                    p.display(),
                    found.len(),
                    tr(self.lang, Key::UnitFile)
                ),
                false,
            );
            for f in found {
                self.enqueue_probe(f);
            }
        } else if p.is_file() {
            self.enqueue_probe(p.to_path_buf());
        }
    }

    pub fn enqueue_probe(&mut self, p: PathBuf) {
        if self.files.iter().any(|f| f.path == p) {
            return;
        }
        if self.pending_probe.contains(&p) {
            return;
        }
        self.pending_probe.push_back(p);
    }

    /// Her turda bir dosya: ffprobe ile coz (UI donmaz, sira gosterilir).
    pub fn tick_probe(&mut self) {
        let Some(p) = self.pending_probe.pop_front() else {
            return;
        };
        let Ok(ff) = self.ff.as_ref() else {
            return;
        };
        match Media::probe(&ff.ffprobe, &p, self.lang) {
            Ok(m) => {
                self.probed_total += 1;
                self.files.push(m);
            }
            Err(e) => {
                let name = p
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| p.display().to_string());
                self.push_log(
                    format!(
                        "{} {name}: {}",
                        tr(self.lang, Key::LogSkippedFile),
                        crate::errors::map_text(&e.to_string(), self.lang)
                    ),
                    true,
                );
            }
        }
    }

    pub fn probing(&self) -> bool {
        !self.pending_probe.is_empty()
    }

    // ------------------------------------------------------------------
    // Gezici
    // ------------------------------------------------------------------

    pub fn open_browse(&mut self, mode: PickMode) {
        // Termux'ta varsayilan: ~/storage/shared/Music -> ~/storage/shared -> home
        let start = match mode {
            PickMode::OutDir => match &self.settings.out_mode {
                OutMode::Dir(d) => Some(d.clone()),
                OutMode::Source => None,
            },
            PickMode::MoveDir => self.settings.src_move_dir.clone(),
            _ => self.files.first().and_then(|m| m.path.parent().map(|p| p.to_path_buf())),
        };
        let mut b = Browse::new(mode, start);
        b.lang = self.lang; // hata mesajlari secili dilde
        self.browse = Some(b);
        self.mode = Mode::Browse;
    }

    /// Gezici secimini uygula.
    pub fn apply_pick(&mut self, r: PickResult) {
        match r {
            PickResult::Paths(paths) => {
                for p in paths {
                    self.add_path(&p);
                }
            }
            PickResult::Dir(d) => {
                let mode = self.browse.as_ref().map(|b| b.mode);
                match mode {
                    Some(PickMode::OutDir) => {
                        self.settings.out_mode = OutMode::Dir(d.clone());
                        self.push_log(
                            format!(
                                "{}: {}",
                                tr(self.lang, Key::LabelOutput),
                                d.display()
                            ),
                            false,
                        );
                    }
                    Some(PickMode::MoveDir) => {
                        self.settings.src_move_dir = Some(d.clone());
                        self.push_log(
                            format!(
                                "{}: {}",
                                tr(self.lang, Key::LabelSrcAction),
                                d.display()
                            ),
                            false,
                        );
                    }
                    _ => {
                        self.add_path(&d);
                    }
                }
                self.settings.save();
            }
        }
        self.mode = Mode::Main;
        self.browse = None;
    }

    pub fn cancel_browse(&mut self) {
        self.browse = None;
        self.mode = Mode::Main;
    }

    // ------------------------------------------------------------------
    // Komut uretimi: ONIZLEME ve CALISTIRMA ayni kaynaktan
    // ------------------------------------------------------------------

    /// Depolama iznini yeniden kontrol et ('r' tusu): izin sonradan
    /// verildiyse hata ekrani kalkar, gezici /sdcard'a baglanir.
    pub fn recheck_storage(&mut self) {
        let onceki = self.storage_error.is_some();
        self.storage_error = crate::storage::health(self.lang);
        match (&self.storage_error, onceki) {
            (None, true) => {
                let l = self.lang;
                self.push_log(tr(l, Key::StorageOk).to_string(), false);
            }
            (Some(_), _) => {}
            (None, false) => {}
        }
    }

    /// Bu dosya icin is spesifikasyonunu uret (skip dahil).
    /// Onizleme komutu DA bu fonksiyondan turer; boylece ekranda gorunen
    /// komut ile gercekte calisan komut birebir aynidir.
    pub fn spec_for(&self, m: &Media) -> Result<JobSpec, String> {
        let lang = self.lang;
        let b = profiles::build(&self.profile, m, lang).map_err(|e| e.to_string())?;
        let out_dir = self.out_dir_for(m);
        let stem = m
            .path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| m.name.clone());
        let mut out = if b.output_is_pattern {
            out_dir.join(format!("{stem}_parca_%03d.{}", b.ext))
        } else {
            out_dir.join(format!("{stem}.{}", b.ext))
        };
        // GUVENLIK: cikti yolu girdinin kendisi olamaz (or. mp4 -> mp4 ayni
        // klasore). Boyle bir durumda "stem_donusen.ext" kullanilir; kaynak
        // dosya hicbir kosulda ezilmez.
        if out == m.path {
            out = out_dir.join(format!("{stem}_donusen.{}", b.ext));
        }
        // cikti zaten varsa ve "uzerine yaz" kapaliysa atlanir
        let skip = out.exists() && !self.settings.overwrite;
        let skip_reason = if skip {
            Some(tr(lang, Key::SkipExists).to_string())
        } else {
            None
        };
        Ok(JobSpec {
            job_index: 0,
            input_prefix_args: b.prefix_args.clone(),
            input: m.path.clone(),
            input_name: m.name.clone(),
            output: out,
            args: b.args.clone(),
            duration: m.duration,
            target_desc: b.desc.clone(),
            skip,
            skip_reason,
            output_is_pattern: b.output_is_pattern,
            cleanup: None,
        })
    }

    fn out_dir_for(&self, m: &Media) -> PathBuf {
        match &self.settings.out_mode {
            OutMode::Dir(d) => d.clone(),
            OutMode::Source => m
                .path
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_default(),
        }
    }

    /// Kuyrugu baslatirken kullanilacak spec'ler + (isim, hata) ciftleri.
    pub fn build_specs(&self) -> (Vec<JobSpec>, Vec<(String, String)>) {
        let mut specs = Vec::new();
        let mut errors = Vec::new();
        for (i, m) in self.files.iter().enumerate() {
            match self.spec_for(m) {
                Ok(mut s) => {
                    s.job_index = i;
                    specs.push(s);
                }
                Err(e) => errors.push((m.name.clone(), e)),
            }
        }
        (specs, errors)
    }

    /// Tek plan: hem onizleme hem calistirma BU fonksiyondan gecer.
    ///
    /// `runnable` icindeki argumanlar aynen `start_jobs`'a verilir; onizleme
    /// de ayni elemandan uretilir -> ekranda gorunen komut = calisan komut.
    pub fn job_plan(&self) -> JobPlan {
        let (specs, errors) = self.build_specs();
        let n_run = specs.iter().filter(|s| !s.skip).count();
        let (workers, threads) = self.plan(n_run);
        let runnable: Vec<JobSpec> = specs
            .iter()
            .filter(|s| !s.skip)
            .cloned()
            .map(|mut s| {
                // otomatik -threads en basa; varsa kullanicinin -threads'i
                // komutun SONUNDA kalir ve ffmpeg'de son gecerli olan kazanir.
                s.args.insert(0, "-threads".into());
                s.args.insert(1, threads.to_string());
                s
            })
            .collect();
        JobPlan {
            specs,
            errors,
            runnable,
            workers,
            threads,
        }
    }

    /// Paralel plan (core::cpu ile ayni mantik).
    pub fn plan(&self, n_jobs: usize) -> (usize, u32) {
        cpu::plan(&self.cpu, self.settings.workers, n_jobs)
    }

    /// Ekranda gosterilecek komut: GERCEK cagrinin aynisi.
    ///
    /// `job_plan().runnable` icindeki spec'in TA KENDISI kullanilir; yani
    /// calistirmada `start_jobs`'a giden arguman listesi ile birebir aynidir.
    pub fn preview_cmd_for(&self, m: &Media) -> String {
        let plan = self.job_plan();
        if let Some(s) = plan.runnable.iter().find(|s| s.input == m.path) {
            return cmd_string(&self.ff_bin(), s);
        }
        if let Some(s) = plan.specs.iter().find(|s| s.input == m.path) {
            if let Some(r) = &s.skip_reason {
                return format!("({})", r);
            }
        }
        if let Some((_, e)) = plan.errors.iter().find(|(n, _)| *n == m.name) {
            return format!("({e})");
        }
        "-".to_string()
    }

    fn ff_bin(&self) -> String {
        self.ff
            .as_ref()
            .map(|f| f.ffmpeg.display().to_string())
            .unwrap_or_else(|_| "ffmpeg".into())
    }

    // ------------------------------------------------------------------
    // Calistirma
    // ------------------------------------------------------------------

    pub fn start(&mut self) {
        if self.running {
            return;
        }
        let Ok(ff) = self.ff.as_ref() else {
            self.push_log(
                format!("{} {}", tr(self.lang, Key::LogError), tr(self.lang, Key::NoFfmpegStart)),
                true,
            );
            return;
        };
        let ffmpeg_path = ff.ffmpeg.clone();

        let plan = self.job_plan();
        for (name, e) in &plan.errors {
            let (name, e) = (name.clone(), e.clone());
            self.push_log(
                format!("[{}] {name}: {e}", tr(self.lang, Key::QueueSkipped)),
                true,
            );
        }
        let specs = plan.specs.clone();
        if specs.is_empty() {
            self.push_log(tr(self.lang, Key::NoFilesToConvert).to_string(), true);
            return;
        }
        let sizes: Vec<u64> = self.files.iter().map(|m| m.size).collect();
        let (workers, threads) = (plan.workers, plan.threads);
        let runnable = plan.runnable.clone();
        if runnable.is_empty() {
            self.push_log(tr(self.lang, Key::NoFilesToConvert).to_string(), true);
            return;
        }

        // kuyruk gorunumu
        self.jobs = specs
            .iter()
            .map(|s| JobRow {
                name: s.input_name.clone(),
                target: s.target_desc.clone(),
                src_size: sizes.get(s.job_index).copied().unwrap_or(0),
                frac: 0.0,
                speed: None,
                state: match &s.skip_reason {
                    Some(r) => JobState::Skipped(r.clone()),
                    None => JobState::Queued,
                },
            })
            .collect();
        self.started = runnable.clone();

        let skipped = specs.iter().filter(|s| s.skip).count();
        self.push_log(
            format!(
                "{} {} {} ({} × {}t), {} {}",
                tr(self.lang, Key::LogQueue),
                runnable.len(),
                tr(self.lang, Key::QueueWillRun),
                workers,
                threads,
                skipped,
                tr(self.lang, Key::QueueSkipped)
            ),
            false,
        );
        for s in &runnable {
            self.push_log(
                format!(
                    "{} {}: {}",
                    tr(self.lang, Key::LogCommand),
                    s.input_name,
                    cmd_string(&ffmpeg_path.display().to_string(), s)
                ),
                false,
            );
        }

        let (tx, rx) = mpsc::channel();
        self.rx = Some(rx);
        self.running = true;
        ffmpeg::start_jobs(ffmpeg_path, runnable, tx, workers);
    }

    /// Kanal mesajlarini isle (her turda cagrilir).
    pub fn poll_jobs(&mut self) {
        let msgs: Vec<JobMsg> = match self.rx.as_ref() {
            Some(rx) => {
                let mut v = Vec::new();
                while let Ok(m) = rx.try_recv() {
                    v.push(m);
                }
                v
            }
            None => return,
        };
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
                JobMsg::Finished {
                    idx,
                    ok,
                    msg,
                    out_size,
                    secs,
                } => {
                    let name = self.jobs.get(idx).map(|j| j.name.clone());
                    let lang = self.lang;
                    if let Some(j) = self.jobs.get_mut(idx) {
                        if ok {
                            j.frac = 1.0;
                            j.state = JobState::Done;
                        } else {
                            j.state = JobState::Failed(msg.clone());
                        }
                    }
                    match (ok, name) {
                        (true, Some(n)) => {
                            let size_s = out_size.map(util::fmt_size).unwrap_or_default();
                            self.push_log(
                                format!(
                                    "{} {n} - {size_s} ({secs:.1} {})",
                                    tr(lang, Key::LogDone),
                                    tr(lang, Key::UnitSecond)
                                ),
                                false,
                            );
                            self.apply_source_action(idx);
                        }
                        (false, Some(n)) => {
                            self.push_log(
                                format!("{} {n}: {msg}", tr(lang, Key::LogError)),
                                true,
                            );
                        }
                        _ => {}
                    }
                }
                JobMsg::AllDone => {
                    self.running = false;
                    let lang = self.lang;
                    let done = self
                        .jobs
                        .iter()
                        .filter(|j| matches!(j.state, JobState::Done))
                        .count();
                    let fail = self
                        .jobs
                        .iter()
                        .filter(|j| matches!(j.state, JobState::Failed(_)))
                        .count();
                    self.push_log(
                        format!(
                            "{} {}: {} {}, {} {}",
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
    }

    /// Basarili donusumden sonra kaynak dosyaya islem (aynen kal / sil / tasi).
    /// Cikti dogrulanmadan (util::output_ok) kaynaga ASLA dokunulmaz.
    pub fn apply_source_action(&mut self, idx: usize) {
        let Some(spec) = self.find_spec(idx) else {
            return;
        };
        let (input, output, pattern, name) = (
            spec.input.clone(),
            spec.output.clone(),
            spec.output_is_pattern,
            spec.input_name.clone(),
        );
        let lang = self.lang;
        match self.settings.src_action {
            SrcAction::Keep => {}
            SrcAction::Delete => {
                if util::output_ok(&output, pattern) {
                    match std::fs::remove_file(&input) {
                        Ok(_) => self.push_log(
                            format!(
                                "{} {}: {}",
                                tr(lang, Key::LogSource),
                                tr(lang, Key::SrcDeleted),
                                name
                            ),
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
                        format!(
                            "{} {}: {}",
                            tr(lang, Key::LogWarning),
                            name,
                            tr(lang, Key::OutNotVerifiedDel)
                        ),
                        true,
                    );
                }
            }
            SrcAction::Move => {
                let Some(dir) = self.settings.src_move_dir.clone() else {
                    self.push_log(
                        format!("{} {}", tr(lang, Key::LogWarning), tr(lang, Key::MoveNoDir)),
                        true,
                    );
                    return;
                };
                if util::output_ok(&output, pattern) {
                    match util::move_to_dir(&input, &dir) {
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
                        format!(
                            "{} {}: {}",
                            tr(lang, Key::LogWarning),
                            name,
                            tr(lang, Key::OutNotVerifiedMove)
                        ),
                        true,
                    );
                }
            }
        }
    }

    fn find_spec(&self, idx: usize) -> Option<&JobSpec> {
        let name = self.jobs.get(idx)?.name.clone();
        self.started.iter().find(|s| s.input_name == name)
    }

    // ------------------------------------------------------------------
    // Kucuk yardimcilar (UI icin)
    // ------------------------------------------------------------------

    pub fn selected_media(&self) -> Option<&Media> {
        self.files.get(self.sel)
    }

    pub fn counts(&self) -> (usize, usize, usize, usize) {
        let running = self
            .jobs
            .iter()
            .filter(|j| matches!(j.state, JobState::Running))
            .count();
        let queued = self
            .jobs
            .iter()
            .filter(|j| matches!(j.state, JobState::Queued))
            .count();
        let done = self
            .jobs
            .iter()
            .filter(|j| matches!(j.state, JobState::Done))
            .count();
        let failed = self
            .jobs
            .iter()
            .filter(|j| matches!(j.state, JobState::Failed(_)))
            .count();
        (running, queued, done, failed)
    }

    pub fn set_lang(&mut self, l: Lang) {
        self.lang = l;
        self.settings.set_lang(l);
        self.settings.save();
    }

    pub fn out_label(&self) -> String {
        match &self.settings.out_mode {
            OutMode::Source => tr(self.lang, Key::OutSource).to_string(),
            OutMode::Dir(d) => d.display().to_string(),
        }
    }
}

/// Kuyruk plani: hem onizleme hem calistirma bu yapidan beslenir.
pub struct JobPlan {
    /// Tum dosyalar icin spec'ler (atlananlar dahil)
    pub specs: Vec<JobSpec>,
    /// Preset'i uygulanamayan dosyalar: (isim, mesaj)
    pub errors: Vec<(String, String)>,
    /// Calistirilacaklar: -threads eklenmis SON arguman listeleri
    pub runnable: Vec<JobSpec>,
    pub workers: usize,
    pub threads: u32,
}

/// Gercek cagrinin komut satiri: `job_args` (core) + binary.
/// start_jobs da AYNI `job_args` fonksiyonunu kullanir; bu yuzden ekranda
/// gorunen komut ile calisan komut birebir aynidir.
pub fn cmd_string(ffmpeg_bin: &str, spec: &JobSpec) -> String {
    let args = ffmpeg::job_args(spec);
    format!("{ffmpeg_bin} {}", args.join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffstudio_core::ffmpeg::testutil;

    fn app_with_files() -> App {
        let mut app = App::new(Settings::default());
        app.ff = Err("test".into()); // ffmpeg yok; sadece spec uretimi test edilir
        app.files.push(testutil::audio_media("sarki.flac", 44.0, false));
        app.files.push(testutil::video_media("film.mkv", 120.0));
        app
    }

    /// ONIZLEME == CALISTIRILAN komut: onizleme de, calistirma da
    /// `job_plan().runnable` icindeki AYNI spec'ten turer. Bu test o esitligi
    /// kilitler: ekranda ne gorunuyorsa o calisir.
    #[test]
    fn onizleme_calistirilanla_ayni() {
        let mut app = app_with_files();
        app.profile.preset = ffstudio_core::profiles::Preset::Mp3V0;
        let m = app.files[0].clone();

        let preview = app.preview_cmd_for(&m);
        let plan = app.job_plan();
        assert_eq!(plan.runnable.len(), 2, "iki dosya da sirada olmali (cikti yok)");
        let run = cmd_string(&app.ff_bin(), &plan.runnable[0]);

        assert_eq!(preview, run, "onizleme ile calisan komut birebir ayni olmali");
        assert!(run.contains("-i"), "-i eksik: {run}");
        assert!(run.contains("sarki.flac"));
        assert!(run.contains(".mp3"), "cikti mp3 olmali: {run}");
        assert!(run.contains("-threads"), "otomatik threads eklenmeli: {run}");
        // ayni girdi -> ayni komut (deterministik)
        assert_eq!(preview, app.preview_cmd_for(&m));
    }

    /// Kullanici kendi -threads degerini verdiyse O kazanir: otomatik -threads
    /// komutun basina, kullanicinin argumanlari sona eklenir (ffmpeg'de son
    /// gecerli olan kazanir).
    #[test]
    fn kullanici_threads_kazanir() {
        let mut app = app_with_files();
        app.profile.custom_args = "-threads 1".into();
        let plan = app.job_plan();
        let spec = &plan.runnable[0];
        let cmd = cmd_string(&app.ff_bin(), spec);

        let first = cmd.find("-threads").expect("otomatik -threads yok");
        let last = cmd.rfind("-threads").expect("-threads yok");
        assert_ne!(first, last, "iki -threads olmali (otomatik + kullanici): {cmd}");
        assert!(
            cmd[last..].starts_with("-threads 1"),
            "kullanicinin -threads'i sonda olmali: {cmd}"
        );
        // otomatik olan basta
        assert!(first < last);
    }

    /// Mevcut cikti varsa varsayilan olarak ATLANIR.
    #[test]
    fn mevcut_cikti_atlanir() {
        let dir = std::env::temp_dir().join(format!("ffstudio_tui_skip_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let input = dir.join("parca.flac");
        std::fs::write(&input, b"x").unwrap();
        let existing = dir.join("parca.mp3");
        std::fs::write(&existing, b"y").unwrap();

        let mut app = App::new(Settings::default());
        app.ff = Err("test".into());
        app.profile.preset = ffstudio_core::profiles::Preset::Mp3V0;
        app.files
            .push(testutil::audio_media("parca.flac", 10.0, false));
        app.files[0].path = input;

        let spec = app.spec_for(&app.files[0].clone()).unwrap();
        assert!(spec.skip, "cikti varken skip olmali");
        assert!(spec.skip_reason.is_some());

        // uzerine yaz acikken atlanmamali
        app.settings.overwrite = true;
        let spec2 = app.spec_for(&app.files[0].clone()).unwrap();
        assert!(!spec2.skip, "overwrite acikken atlanmamali");

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Girdi ile cikti ayni yol olamaz: mp4 -> mp4 donusumunde kaynak ezilmez.
    #[test]
    fn girdi_cikti_cakismasi_engellenir() {
        let mut app = App::new(Settings::default());
        app.ff = Err("test".into());
        app.profile.preset = ffstudio_core::profiles::Preset::H264Mp4;
        app.files.push(testutil::video_media("film.mp4", 30.0));
        let m = app.files[0].clone();
        let spec = app.spec_for(&m).unwrap();
        assert_ne!(spec.output, m.path, "cikti girdiyi ezmemeli");
        assert!(
            spec.output.to_string_lossy().contains("_donusen"),
            "cakismada _donusen son eki: {}",
            spec.output.display()
        );
        // farkli uzantiya ceviride (mp4 -> mkv) ek yok
        app.profile.preset = ffstudio_core::profiles::Preset::H265Mkv;
        let spec2 = app.spec_for(&m).unwrap();
        assert_eq!(spec2.output.file_name().unwrap(), "film.mkv");
    }

    /// Paralel plan core::cpu'dan gelir ve ayarla sabitlenebilir.
    #[test]
    fn paralel_plan_ayardan_sabitlenir() {
        let mut app = App::new(Settings::default());
        app.cpu = CpuInfo {
            model: "Test".into(),
            physical: Some(4),
            logical: 8,
        };
        app.settings.workers = 0;
        let (w_auto, t_auto) = app.plan(4);
        assert_eq!((w_auto, t_auto), (4, 2), "8T -> 4 is x 2 thread");

        app.settings.workers = 2;
        let (w, t) = app.plan(4);
        assert_eq!((w, t), (2, 4), "manuel 2 is -> 4 thread");

        // tek is: CPU'nun tamami
        app.settings.workers = 0;
        let (w1, t1) = app.plan(1);
        assert_eq!((w1, t1), (1, 8), "tek is -> tum thread'ler");
    }
}
