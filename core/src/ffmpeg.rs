//! ffmpeg/ffprobe yonetimi: binary bulma, gomulu zip cikarma,
//! medya bilgisi alma (ffprobe) ve sırayla dönüşüma (ffmpeg subprocess).

use crate::lang::{tr, Key, Lang};
use anyhow::{bail, Context, Result};
use std::fs;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// Platform binary uzantisi: Windows'ta ".exe", Linux/Termux'ta bos.
/// (Termux'ta ffmpeg/ffprobe uzantisiz olur: `pkg install ffmpeg`.)
pub const EXE: &str = if cfg!(windows) { ".exe" } else { "" };

#[cfg(embed_ffmpeg)]
const EMBEDDED_ZIP: &[u8] = include_bytes!("../ffmpeg.zip");

// ---------------------------------------------------------------------------
// Binary yonetimi
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct Ffmpeg {
    pub ffmpeg: PathBuf,
    pub ffprobe: PathBuf,
    pub source: String,
}

impl Ffmpeg {
    /// ffmpeg.exe / ffprobe.exe bul.
    /// Oncelik: 1) gomulu (ffmpeg.zip derleme anında içine gomuldu)
    ///          2) exe yanında  3) PATH
    pub fn locate(lang: Lang) -> Result<Self> {
        // 1) gomulu
        #[cfg(embed_ffmpeg)]
        if let Some(dir) = data_dir() {
            let bin = dir.join("ffmpeg");
            let ff = bin.join(format!("ffmpeg{EXE}"));
            let fp = bin.join(format!("ffprobe{EXE}"));
            if ff.is_file() && fp.is_file() {
                return Ok(Self {
                    ffmpeg: ff,
                    ffprobe: fp,
                    source: tr(lang, Key::SrcEmbedded).to_string(),
                });
            }
            fs::create_dir_all(&bin).context("ffmpeg klasörü oluşturulamadı")?;
            extract_zip(EMBEDDED_ZIP, &bin).context("gömülü ffmpeg çıkarılamadı")?;
            if ff.is_file() && fp.is_file() {
                return Ok(Self {
                    ffmpeg: ff,
                    ffprobe: fp,
                    source: tr(lang, Key::SrcEmbedded).to_string(),
                });
            }
        }

        // 2) exe yanında (ffmpeg.exe veya ffmpeg/ffmpeg.exe)
        if let Ok(exe) = std::env::current_exe() {
            if let Some(d) = exe.parent() {
                for base in [d.to_path_buf(), d.join("ffmpeg")] {
                    let ff = base.join(format!("ffmpeg{EXE}"));
                    let fp = base.join(format!("ffprobe{EXE}"));
                    if ff.is_file() && fp.is_file() {
                        return Ok(Self {
                            ffmpeg: ff,
                            ffprobe: fp,
                            source: tr(lang, Key::SrcExeSide).to_string(),
                        });
                    }
                }
            }
        }

        // 3) PATH
        if let (Some(ff), Some(fp)) = (find_in_path("ffmpeg"), find_in_path("ffprobe")) {
            return Ok(Self {
                ffmpeg: ff,
                ffprobe: fp,
                source: tr(lang, Key::SrcPath).to_string(),
            });
        }

        bail!("{}", tr(lang, Key::FfmpegNotFound));
    }

    pub fn version_line(&self) -> String {
        let mut cmd = Command::new(&self.ffmpeg);
        cmd.arg("-version");
        #[cfg(windows)]
        cmd.creation_flags(CREATE_NO_WINDOW);
        cmd.output()
            .ok()
            .and_then(|o| {
                String::from_utf8_lossy(&o.stdout)
                    .lines()
                    .next()
                    .map(|l| l.to_string())
            })
            .unwrap_or_default()
    }
}

#[cfg_attr(not(embed_ffmpeg), allow(dead_code))]
fn data_dir() -> Option<PathBuf> {
    dirs::data_local_dir().map(|d| d.join("ffstudio"))
}

fn find_in_path(bin: &str) -> Option<PathBuf> {
    let name = if cfg!(windows) {
        format!("{bin}.exe")
    } else {
        bin.to_string()
    };
    let var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&var) {
        let c = dir.join(&name);
        if c.is_file() {
            return Some(c);
        }
    }
    None
}

/// Gomulu zipten sadece ffmpeg.exe + ffprobe.exe cikarir (doc/license atlanir).
#[cfg_attr(not(embed_ffmpeg), allow(dead_code))]
#[cfg(feature = "embed_ffmpeg")]
#[allow(dead_code)] // feature acik ama ffmpeg.zip yoksa (embed_ffmpeg cfg kapali) kullanilmaz
fn extract_zip(bytes: &[u8], target: &Path) -> Result<()> {
    use std::io::Cursor;
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).context("zip açılamadı")?;
    let mut found_ff = false;
    let mut found_fp = false;
    for i in 0..archive.len() {
        let mut f = archive.by_index(i).with_context(|| format!("zip girdi #{} okunamadı", i))?;
        let name = f.name().to_string();
        let fname = Path::new(&name)
            .file_name()
            .map(|n| n.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        if name.contains("..") {
            continue;
        }
        let want_ff = format!("ffmpeg{EXE}");
        let want_fp = format!("ffprobe{EXE}");
        if fname == want_ff || fname == want_fp {
            let mut buf = Vec::with_capacity(f.size() as usize);
            f.read_to_end(&mut buf)
                .with_context(|| format!("'{name}' zipten okunamadı"))?;
            let out = target.join(&fname);
            fs::write(&out, &buf).with_context(|| format!("yazılamadı: {}", out.display()))?;
            if fname == want_ff {
                found_ff = true;
            } else {
                found_fp = true;
            }
        }
    }
    if !found_ff || !found_fp {
        bail!("zip içinde ffmpeg{EXE} ve/veya ffprobe{EXE} bulunamadı.");
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Medya bilgisi (ffprobe)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Audio,
    Video,
    Image,
}

#[derive(Clone, Debug)]
pub struct VideoInfo {
    pub codec: String,
    pub profile: String,
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub pix_fmt: String,
    pub bitrate: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct AudioInfo {
    pub codec: String,
    pub sample_rate: u32,
    pub channels: u32,
    pub bitrate: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct Media {
    pub path: PathBuf,
    pub name: String,
    pub kind: Kind,
    pub size: u64,
    pub duration: f64, // saniye; resimlerde 0
    pub container: String,
    pub video: Option<VideoInfo>,
    pub audio: Option<AudioInfo>,
    pub bit_rate: Option<u64>, // genel, format seviyesinde (bps)
    pub extra: Vec<String>,
    pub has_cover: bool, // attached_pic (kapak gorevli) var mi
}

impl Media {
    pub fn probe(ffprobe: &Path, path: &Path, lang: Lang) -> Result<Media> {
        let mut cmd = Command::new(ffprobe);
        cmd.args(["-v", "error", "-print_format", "json", "-show_format", "-show_streams"]);
        cmd.arg(path);
        #[cfg(windows)]
        cmd.creation_flags(CREATE_NO_WINDOW);
        let out = cmd
            .output()
            .with_context(|| format!("ffprobe çalıştırılamadı: {}", path.display()))?;
        if !out.status.success() {
            bail!(
                "ffprobe hatası: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            );
        }
        let v: serde_json::Value =
            serde_json::from_slice(&out.stdout).context("ffprobe çıktısı çözümlenemedi")?;
        let fmt = v.get("format").cloned().unwrap_or(serde_json::Value::Null);
        let mut video: Option<VideoInfo> = None;
        let mut audio: Option<AudioInfo> = None;
        let mut extra: Vec<String> = Vec::new();
        let mut is_image = false;
        let mut has_cover = false;
        let streams = v
            .get("streams")
            .and_then(|s| s.as_array())
            .cloned()
            .unwrap_or_default();

        for s in &streams {
            let t = s.get("codec_type").and_then(|x| x.as_str()).unwrap_or("");
            let name = s
                .get("codec_name")
                .and_then(|x| x.as_str())
                .unwrap_or("?")
                .to_string();
            match t {
                "video" => {
                    let attached = s
                        .get("disposition")
                        .and_then(|d| d.get("attached_pic"))
                        .and_then(|x| x.as_i64())
                        .unwrap_or(0)
                        == 1;
                    if attached {
                        extra.push(format!("{}: {name}", tr(lang, Key::ExtraCover)));
                        has_cover = true;
                        continue;
                    }
                    let vi = VideoInfo {
                        codec: name.clone(),
                        profile: s
                            .get("profile")
                            .and_then(|x| x.as_str())
                            .unwrap_or("")
                            .to_string(),
                        width: s.get("width").and_then(|x| x.as_u64()).unwrap_or(0) as u32,
                        height: s.get("height").and_then(|x| x.as_u64()).unwrap_or(0) as u32,
                        fps: parse_rate(s.get("avg_frame_rate"))
                            .or_else(|| parse_rate(s.get("r_frame_rate")))
                            .unwrap_or(0.0),
                        pix_fmt: s
                            .get("pix_fmt")
                            .and_then(|x| x.as_str())
                            .unwrap_or("")
                            .to_string(),
                        bitrate: s
                            .get("bit_rate")
                            .and_then(|x| x.as_str())
                            .and_then(|x| x.parse().ok()),
                    };
                    let nb = s.get("nb_frames").and_then(|x| x.as_str());
                    if s.get("duration").is_none() && (nb == Some("1") || vi.fps <= 0.0) {
                        is_image = true;
                    }
                    extra.push(format!("{}: {name} {}x{}", tr(lang, Key::ExtraVideo), vi.width, vi.height));
                    video = Some(vi);
                }
                "audio" => {
                    let ai = AudioInfo {
                        codec: name.clone(),
                        // ffprobe JSON'da sample_rate STRING'tir ("44100") — saymi olarak okumak 0 verir
                        sample_rate: s
                            .get("sample_rate")
                            .and_then(|x| x.as_str())
                            .and_then(|x| x.parse::<u64>().ok())
                            .unwrap_or(0) as u32,
                        channels: s.get("channels").and_then(|x| x.as_u64()).unwrap_or(0) as u32,
                        bitrate: s
                            .get("bit_rate")
                            .and_then(|x| x.as_str())
                            .and_then(|x| x.parse().ok()),
                    };
                    extra.push(format!(
                        "{}: {name} {} Hz / {} {}",
                        tr(lang, Key::ExtraAudio),
                        ai.sample_rate,
                        ai.channels,
                        tr(lang, Key::UnitChannel)
                    ));
                    audio = Some(ai);
                }
                other => extra.push(format!("{other} {}: {name}", tr(lang, Key::ExtraStream))),
            }
        }

        let duration = fmt
            .get("duration")
            .and_then(|x| x.as_str())
            .and_then(|x| x.parse::<f64>().ok())
            .unwrap_or(0.0);

        Ok(Media {
            path: path.to_path_buf(),
            name: path
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| path.display().to_string()),
            kind: if is_image {
                Kind::Image
            } else if video.is_some() {
                Kind::Video
            } else {
                Kind::Audio
            },
            size: path.metadata().map(|m| m.len()).unwrap_or(0),
            duration,
            container: fmt
                .get("format_name")
                .and_then(|x| x.as_str())
                .unwrap_or("?")
                .to_string(),
            video,
            audio,
            bit_rate: fmt
                .get("bit_rate")
                .and_then(|x| x.as_str())
                .and_then(|x| x.parse().ok()),
            extra,
            has_cover,
        })
    }

    pub fn summary(&self) -> String {
        match self.kind {
            Kind::Image => {
                if let Some(v) = &self.video {
                    format!("{}x{} - {}", v.width, v.height, v.codec)
                } else {
                    "resim".into()
                }
            }
            Kind::Video => {
                let mut s = String::new();
                if let Some(v) = &self.video {
                    s.push_str(&format!(
                        "{}x{} @ {:.0}fps {} ",
                        v.width, v.height, v.fps, v.codec
                    ));
                }
                if let Some(a) = &self.audio {
                    s.push_str(&format!("+ {}", a.codec));
                }
                if s.trim().is_empty() {
                    "?".into()
                } else {
                    s
                }
            }
            Kind::Audio => {
                if let Some(a) = &self.audio {
                    format!("{} - {} Hz - {} kanal", a.codec, a.sample_rate, a.channels)
                } else {
                    "ses".into()
                }
            }
        }
    }
}

fn parse_rate(v: Option<&serde_json::Value>) -> Option<f64> {
    let s = v?.as_str()?;
    let mut it = s.splitn(2, '/');
    let a: f64 = it.next()?.parse().ok()?;
    let b: f64 = it.next()?.parse().ok()?;
    if b > 0.0 {
        Some(a / b)
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Dönustürme (ffmpeg subprocess)
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct JobSpec {
    pub job_index: usize,
    /// `-i` ARGINDAN önce gelen argümanlar (örn: concat demuxer için `-f concat -safe 0`)
    pub input_prefix_args: Vec<String>,
    pub input: PathBuf,
    pub input_name: String,
    pub output: PathBuf,
    pub args: Vec<String>,
    pub duration: f64,
    pub target_desc: String,
    pub skip: bool,
    /// atlanirsa nedensinin gorunur yazilisi (kuyrukta gosterilir)
    pub skip_reason: Option<String>,
    /// true ise çıktı bir desen (`%03d`) — boyut ölçme dene
    pub output_is_pattern: bool,
    /// iş bitince silinecek dosya (örn: geçici concat listesi)
    pub cleanup: Option<PathBuf>,
}

#[derive(Clone)]
pub enum JobMsg {
    Progress {
        idx: usize,
        frac: f32,
        speed: Option<String>,
    },
    Finished {
        idx: usize,
        ok: bool,
        msg: String,
        out_size: Option<u64>,
        secs: f64,
    },
    AllDone,
}

/// Pool thread sayisi: istenen deger 1..=16'ya cekilir ve — onemlisi —
/// IS SAYISIYLA SINIRLANIR.
///
/// ESKI BUG: `.clamp(1,16).max(is_sayisi)` yazilmisti; 140 dosya kuyruğa
/// girince 140 worker = 140 ffmpeg procesi AYNI ANDA dogardi ve makine
/// kilitleniyordu. Dogrusu `.min`: worker sayisi asla is sayisini
/// gecemez.
pub fn pool_workers(requested: usize, n_jobs: usize) -> usize {
    if n_jobs == 0 {
        return 0;
    }
    requested.clamp(1, 16).min(n_jobs)
}

/// Paralel calisan arka plan is-pool'u baslatir.
/// `workers` kadar thread, ortak kuyruktan is alir;
/// tumu bitince TEK AllDone gonderilir (son biten gonderir).
pub fn start_jobs(ffmpeg: PathBuf, specs: Vec<JobSpec>, tx: mpsc::Sender<JobMsg>, workers: usize) {
    let n_jobs = specs.iter().filter(|s| !s.skip).count();
    let workers = pool_workers(workers, n_jobs);
    if workers == 0 {
        let _ = tx.send(JobMsg::AllDone);
        return;
    }
    let specs = std::sync::Arc::new(specs);
    let queue: std::sync::Arc<std::sync::Mutex<std::collections::VecDeque<usize>>> = std::sync::Arc::new(std::sync::Mutex::new(
        specs
            .iter()
            .enumerate()
            .filter(|(_, s)| !s.skip)
            .map(|(i, _)| i)
            .collect(),
    ));
    let ffmpeg = std::sync::Arc::new(ffmpeg);
    let tx = std::sync::Arc::new(tx);
    let active = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(workers));

    for _ in 0..workers {
        let q = queue.clone();
        let specs = specs.clone();
        let ffmpeg = ffmpeg.clone();
        let tx = tx.clone();
        let active = active.clone();
        thread::spawn(move || {
            loop {
                let Some(pos) = q.lock().unwrap().pop_front() else {
                    break;
                };
                let t0 = Instant::now();
                let spec = specs[pos].clone();
                let (ok, msg, out_size) = run_one(&ffmpeg, &spec, &tx);
                let _ = tx.send(JobMsg::Finished {
                    idx: spec.job_index,
                    ok,
                    msg,
                    out_size,
                    secs: t0.elapsed().as_secs_f64(),
                });
            }
            // kuyruk bitti: son biten worker AllDone'u gonderir
            use std::sync::atomic::Ordering;
            if active.fetch_sub(1, Ordering::SeqCst) == 1 {
                let _ = tx.send(JobMsg::AllDone);
            }
        });
    }
}

/// ffmpeg argüman vektoru (ayri bir fonksiyon: test edilebilir,
/// cunku eksik "-i" gercek dosyalari yikayan bir bug'a yol acmisti)
pub fn job_args(job: &JobSpec) -> Vec<String> {
    let mut v = vec![
        "-nostdin".into(),
        "-hide_banner".into(),
        "-loglevel".into(),
        "error".into(),
        "-y".into(),
    ];
    v.extend(job.input_prefix_args.iter().cloned());
    v.push("-i".into());
    v.push(job.input.display().to_string());
    v.extend(job.args.iter().cloned());
    v.push(job.output.display().to_string());
    v.push("-progress".into());
    v.push("pipe:1".into());
    v
}

fn run_one(ffmpeg: &Path, job: &JobSpec, tx: &mpsc::Sender<JobMsg>) -> (bool, String, Option<u64>) {
    let total_ms = job.duration * 1000.0;

    let mut cmd = Command::new(ffmpeg);
    cmd.args(job_args(job));
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => return (false, format!("ffmpeg çalıştırılamadı: {e}"), None),
    };
    let stdout = child.stdout.take().expect("stdout");
    let mut stderr = child.stderr.take().expect("stderr");

    // stdout: -progress satirlarini oku, ilerlemeyi ilet
    let (ptx, prx) = mpsc::channel::<(f32, Option<String>)>();
    let pthr = thread::spawn(move || {
        let mut frac = 0.0f32;
        for line in BufReader::new(stdout).lines() {
            let Ok(line) = line else { break };
            if let Some(v) = line.strip_prefix("out_time_us=") {
                if let Ok(us) = v.parse::<f64>() {
                    if total_ms > 0.0 {
                        frac = ((us / 1_000_000.0) / (total_ms / 1000.0)).clamp(0.0, 1.0) as f32;
                        let _ = ptx.send((frac, None));
                    }
                }
            } else if let Some(v) = line.strip_prefix("out_time_ms=") {
                if let Ok(ms) = v.parse::<f64>() {
                    if total_ms > 0.0 {
                        frac = ((ms / 1000.0) / (total_ms / 1000.0)).clamp(0.0, 1.0) as f32;
                        let _ = ptx.send((frac, None));
                    }
                }
            } else if let Some(v) = line.strip_prefix("speed=") {
                let t = v.trim().to_string();
                if !t.is_empty() && t != "0.0" && t != "N/A" {
                    let _ = ptx.send((frac, Some(format!("x{}", t))));
                }
            }
        }
        let _ = ptx.send((frac, None));
    });

    // stderr: hata mesajlari
    let ethr = thread::spawn(move || {
        let mut s = String::new();
        let _ = Read::read_to_string(&mut stderr, &mut s);
        s
    });

    for (frac, speed) in prx {
        let _ = tx.send(JobMsg::Progress {
            idx: job.job_index,
            frac,
            speed,
        });
    }
    pthr.join().ok();
    let stderr_text = ethr.join().unwrap_or_default();
    let status = child.wait().ok();
    if let Some(f) = &job.cleanup {
        let _ = fs::remove_file(f);
    }
    let ok = status.map(|s| s.success()).unwrap_or(false);
    let out_size = if ok && !job.output_is_pattern {
        Some(job.output.metadata().map(|m| m.len()).unwrap_or(0))
    } else {
        None
    };
    let msg = if ok {
        String::new()
    } else {
        let all: Vec<&str> = stderr_text.lines().collect();
        let start = all.len().saturating_sub(10);
        let t = all[start..].join(" | ");
        if t.trim().is_empty() {
            "ffmpeg hata kodu döndü (detay yok)".into()
        } else {
            t
        }
    };
    (ok, msg, out_size)
}

#[cfg(any(test, feature = "testutil"))]
pub mod testutil {
    use super::*;

    pub fn audio_media(name: &str, duration: f64, cover: bool) -> Media {
        Media {
            path: PathBuf::from(format!("/x/{name}")),
            name: name.to_string(),
            kind: Kind::Audio,
            size: 4_000_000,
            duration,
            container: "flac".into(),
            video: None,
            audio: Some(AudioInfo {
                codec: "flac".into(),
                sample_rate: 44100,
                channels: 2,
                bitrate: Some(1_000_000),
            }),
            bit_rate: Some(1_000_000),
            extra: Vec::new(),
            has_cover: cover,
        }
    }

    pub fn video_media(name: &str, duration: f64) -> Media {
        Media {
            path: PathBuf::from(format!("/x/{name}")),
            name: name.to_string(),
            kind: Kind::Video,
            size: 20_000_000,
            duration,
            container: "mkv".into(),
            video: Some(VideoInfo {
                codec: "h264".into(),
                profile: "High".into(),
                width: 1920,
                height: 1080,
                fps: 30.0,
                pix_fmt: "yuv420p".into(),
                bitrate: Some(6_000_000),
            }),
            audio: Some(AudioInfo {
                codec: "aac".into(),
                sample_rate: 48000,
                channels: 2,
                bitrate: Some(192_000),
            }),
            bit_rate: Some(6_200_000),
            extra: Vec::new(),
            has_cover: false,
        }
    }

    pub fn image_media(name: &str) -> Media {
        Media {
            path: PathBuf::from(format!("/x/{name}")),
            name: name.to_string(),
            kind: Kind::Image,
            size: 1_000_000,
            duration: 0.0,
            container: "png".into(),
            video: Some(VideoInfo {
                codec: "png".into(),
                profile: String::new(),
                width: 800,
                height: 600,
                fps: 0.0,
                pix_fmt: "rgb24".into(),
                bitrate: None,
            }),
            audio: None,
            bit_rate: None,
            extra: Vec::new(),
            has_cover: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_job(prefix: Vec<String>) -> JobSpec {
        JobSpec {
            job_index: 0,
            input_prefix_args: prefix,
            input: PathBuf::from("/x/in.flac"),
            input_name: "in.flac".into(),
            output: PathBuf::from("/x/out.mp3"),
            args: vec!["-c:a".into(), "libmp3lame".into(), "-q:a".into(), "0".into()],
            duration: 3.0,
            target_desc: "test".into(),
            skip: false,
            skip_reason: None,
            output_is_pattern: false,
            cleanup: None,
        }
    }

    #[test]
    fn komutta_i_var() {
        // regresyon testi: eksik "-i" ffmpeg'in input'u output sanmasina yol aciyordu
        let a = job_args(&test_job(vec![]));
        let i_pos = a.iter().position(|x| x == "-i").expect("komutta -i olmali");
        assert_eq!(a[i_pos + 1], "/x/in.flac", "-i'den hemen sonra input olmali");
        assert_eq!(a[i_pos.saturating_sub(1)], "-y", "-i'den hemen once -y olmali");
        assert_eq!(*a.last().unwrap(), "pipe:1");
        assert_eq!(a[a.len() - 2], "-progress");
    }

    #[test]
    fn concat_prefix_i_oncesine() {
        let job = test_job(vec!["-f".into(), "concat".into(), "-safe".into(), "0".into()]);
        let a = job_args(&job);
        let i_pos = a.iter().position(|x| x == "-i").unwrap();
        assert_eq!(a[i_pos - 1], "0");
        assert_eq!(a[i_pos - 2], "-safe");
    }

    /// Worker pool: her is (hata bile olsa) Finished gondermeli,
    /// pool sonlandiginda TEK AllDone gelmeli. (binary yok → isler
    /// aninda hata verir; amac ffmpeg DEGIL, pool mantigidir)
    #[test]
    fn worker_pool_her_is_biter_tek_alldone() {
        let (tx, rx) = mpsc::channel::<JobMsg>();
        let specs: Vec<JobSpec> = (0..4)
            .map(|i| JobSpec {
                job_index: i,
                input_prefix_args: vec![],
                input: PathBuf::from(format!("/x/missing_{}.flac", i)),
                input_name: format!("missing_{}.flac", i),
                output: PathBuf::from(format!("/x/out_{}.mp3", i)),
                args: vec!["-c:a".into(), "libmp3lame".into(), "-q:a".into(), "0".into()],
                duration: 1.0,
                target_desc: "test".into(),
                skip: false,
                skip_reason: None,
                output_is_pattern: false,
                cleanup: None,
            })
            .collect();
        start_jobs(
            PathBuf::from("definitely_not_a_binary_xyz"),
            specs,
            tx,
            3,
        );
        let mut finished = 0;
        let mut alldone = 0;
        let deadline = Instant::now() + std::time::Duration::from_secs(15);
        while (finished < 4 || alldone < 1) && Instant::now() < deadline {
            match rx.recv_timeout(std::time::Duration::from_secs(1)) {
                Ok(JobMsg::Finished { .. }) => finished += 1,
                Ok(JobMsg::AllDone) => alldone += 1,
                Ok(JobMsg::Progress { .. }) => {}
                Err(_) => break,
            }
        }
        assert_eq!(finished, 4, "her is Finished bildirmeli");
        assert_eq!(alldone, 1, "tam olarak bir AllDone olmali");
        assert!(matches!(rx.try_recv(), Err(_)), "AllDone'dan sonra mesaj kalmamali");
    }

    /// Kilitlenme bug'i regresyon test'i: istenen worker sayisi ASLA is
    /// sayisini gecemez (eski `.max()` ile 140 dosya = 140 ayni anda
    /// ffmpeg procesiydi).
    #[test]
    fn pool_workers_hesap() {
        assert_eq!(pool_workers(2, 140), 2, "140 is icin 2 worker kalmali");
        assert_eq!(pool_workers(16, 3), 3, "worker is sayisini gecememeli");
        assert_eq!(pool_workers(0, 5), 1, "0 istenirse en az 1");
        assert_eq!(pool_workers(99, 10), 10, "16 ustunde istek clamp + min");
        assert_eq!(pool_workers(16, 16), 16, "siner");
        assert_eq!(pool_workers(2, 0), 0, "is yoksa worker da yok");
    }

    /// Uc boyutlu kanit: uyuyan sahte ffmpeg ile 6 is / 2 worker =>
    /// 3 dalga x 0.3s ~= 0.9s surer. Pool sinsi sekilde 6 proses
    /// dogurssaydi (eski bug) ~0.3s'de biterdi.
    #[cfg(unix)]
    #[test]
    fn pool_concurrency_is_sayisiyla_sinirli() {
        use std::os::unix::fs::PermissionsExt;
        let dir = std::env::temp_dir().join(format!("ffstudio_pooltest_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let ff = dir.join("ffmpeg");
        std::fs::write(&ff, "#!/bin/sh\nsleep 0.3\n").unwrap();
        let mut p = std::fs::metadata(&ff).unwrap().permissions();
        p.set_mode(0o755);
        std::fs::set_permissions(&ff, p).unwrap();

        let specs: Vec<JobSpec> = (0..6)
            .map(|i| JobSpec {
                job_index: i,
                input_prefix_args: vec![],
                input: PathBuf::from(format!("/x/song_{}.mp3", i)),
                input_name: format!("song_{}.mp3", i),
                output: PathBuf::from(format!("/x/out_{}.mp3", i)),
                args: vec![],
                duration: 1.0,
                target_desc: "test".into(),
                skip: false,
                skip_reason: None,
                output_is_pattern: false,
                cleanup: None,
            })
            .collect();

        let (tx, rx) = mpsc::channel::<JobMsg>();
        let t0 = Instant::now();
        start_jobs(ff, specs, tx, 2);
        let mut finished = 0;
        let mut alldone = 0;
        let deadline = t0 + std::time::Duration::from_secs(20);
        while (finished < 6 || alldone < 1) && Instant::now() < deadline {
            match rx.recv_timeout(std::time::Duration::from_secs(1)) {
                Ok(JobMsg::Finished { .. }) => finished += 1,
                Ok(JobMsg::AllDone) => alldone += 1,
                Ok(JobMsg::Progress { .. }) => {}
                Err(_) => break,
            }
        }
        let secs = t0.elapsed().as_secs_f64();
        assert_eq!(finished, 6, "her is Finished bildirmeli");
        assert_eq!(alldone, 1, "tam olarak bir AllDone olmali");
        // 2 worker x 6 is x 0.3s = 3 dalga = ~0.9s; sinirli calistigini
        // kanitlamak icin en az 0.6s surmus olmali (6 paralel ise ~0.3s).
        assert!(secs >= 0.6, "6 is 2 worker'la en az 0.6s surmeli (goc: {secs:.2}s)");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
