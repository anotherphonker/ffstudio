//! Uctan uca test: gercek ffmpeg ile klasor tarama + paralel donusum.
//!
//! Kabul kriterleri:
//!  3) klasor sec -> recursive tarama -> FLAC'lar -> MP3 V0 paralel
//!  4) onizleme komutu == calistirilan komut (app::tests ile de kilitli)
//!  5) mevcut ciktilar varsayilan olarak atlanir
//!
//! ffmpeg/ffprobe PATH'te yoksa test atlanir (Termux'ta `pkg install ffmpeg`).

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use ffstudio_core::ffmpeg::Ffmpeg;
use ffstudio_core::lang::Lang;
use ffstudio_core::profiles::Preset;

use ffstudio_tui::app::App;
use ffstudio_tui::config::Settings;

fn ffmpeg_var() -> bool {
    Ffmpeg::locate(Lang::Tr).is_ok()
}

fn tmp(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!(
        "ffstudio_tui_e2e_{}_{}",
        std::process::id(),
        tag
    ));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// Test icin kisa bir FLAC uretir (ffmpeg ile).
fn make_flac(ffmpeg: &Path, out: &Path, freq: u32, secs: u32) {
    let st = Command::new(ffmpeg)
        .args([
            "-nostdin",
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            &format!("sine=frequency={freq}:duration={secs}"),
            "-c:a",
            "flac",
        ])
        .arg(out)
        .status()
        .expect("ffmpeg calistirilamadi");
    assert!(st.success(), "flac uretilemedi: {}", out.display());
}

fn probe_video_codec(ffprobe: &Path, f: &Path) -> String {
    let out = Command::new(ffprobe)
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=codec_name",
            "-of",
            "csv=p=0",
        ])
        .arg(f)
        .output()
        .expect("ffprobe calistirilamadi");
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn probe_codec(ffprobe: &Path, f: &Path) -> String {
    let out = Command::new(ffprobe)
        .args([
            "-v",
            "error",
            "-select_streams",
            "a:0",
            "-show_entries",
            "stream=codec_name",
            "-of",
            "csv=p=0",
        ])
        .arg(f)
        .output()
        .expect("ffprobe calistirilamadi");
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// Kuyruk bitene kadar her 150 ms'de bir mesajlari isle (TUI dongusunun aynisi).
fn wait_done(app: &mut App, timeout: Duration) -> bool {
    let t0 = Instant::now();
    while app.running && t0.elapsed() < timeout {
        app.poll_jobs();
        std::thread::sleep(Duration::from_millis(150));
    }
    !app.running
}

#[test]
fn klasor_tara_flac_to_mp3_paralel() {
    if !ffmpeg_var() {
        eprintln!("ffmpeg yok: E2E testi atlandi (Termux: pkg install ffmpeg)");
        return;
    }
    let ff = Ffmpeg::locate(Lang::Tr).unwrap();
    let dir = tmp("flac");
    // alt klasor dahil 5 flac: recursive tarama kanitlanir
    let sub = dir.join("alt_klasor");
    std::fs::create_dir_all(&sub).unwrap();
    let mut kaynaklar = Vec::new();
    for (i, freq) in [220u32, 330, 440, 550].iter().enumerate() {
        let p = dir.join(format!("parca_{i}.flac"));
        make_flac(&ff.ffmpeg, &p, *freq, 2);
        kaynaklar.push(p);
    }
    let p = sub.join("derin.flac");
    make_flac(&ff.ffmpeg, &p, 660, 2);
    kaynaklar.push(p);
    // medya olmayan dosya taranmamali
    std::fs::write(dir.join("notlar.txt"), b"bu medya degil").unwrap();

    let mut app = App::new(Settings::default());
    assert!(app.ff.is_ok(), "ffmpeg bulunamadi");

    // a) klasor secildi -> recursive tarama kuyruga alinir
    app.add_path(&dir);
    assert_eq!(app.pending_probe.len(), 5, "5 flac taranmali (txt degil)");
    let mut n = 0;
    while app.probing() && n < 500 {
        app.tick_probe();
        n += 1;
    }
    assert_eq!(app.files.len(), 5, "ffprobe ile 5 medya cozulmeli");

    // b) MP3 V0 + paralel plan
    app.profile.preset = Preset::Mp3V0;
    app.theme = ffstudio_tui::config::Theme::Default;
    let plan = app.job_plan();
    assert_eq!(plan.runnable.len(), 5);
    assert!(plan.workers >= 1 && plan.threads >= 1);
    // c) onizleme == calistirilacak komut
    let m0 = app.files[0].clone();
    let preview = app.preview_cmd_for(&m0);
    let gercek = ffstudio_tui::app::cmd_string(
        &app.ff.as_ref().unwrap().ffmpeg.display().to_string(),
        &plan.runnable[0],
    );
    assert_eq!(preview, gercek, "onizleme = calistirilan komut olmali");

    // d) paralel donusum
    app.settings.workers = 2;
    app.start();
    assert!(app.running, "kuyruk baslamali");
    assert!(
        wait_done(&mut app, Duration::from_secs(180)),
        "donusum zaman asimina ugradi"
    );

    let (_, _, done, fail) = app.counts();
    assert_eq!(
        (done, fail),
        (5, 0),
        "5 basarili, 0 hatali olmali; gunluk:\n{}",
        app.log.iter().map(|(l, _)| l.clone()).collect::<Vec<_>>().join("\n")
    );

    // e) ciktilar gercekten mp3 mu (ffprobe ile dogrula)
    for k in &kaynaklar {
        let out = k.with_extension("mp3");
        assert!(out.is_file(), "cikti yok: {}", out.display());
        assert!(
            out.metadata().unwrap().len() > 1000,
            "cikti cok kucuk: {}",
            out.display()
        );
        let codec = probe_codec(&ff.ffprobe, &out);
        assert_eq!(codec, "mp3", "cikti mp3 degil: {} -> {codec}", out.display());
    }

    // f) ikinci tur: mevcut ciktilar VARSAYILAN olarak atlanir
    let mut app2 = App::new(Settings::default());
    for k in &kaynaklar {
        app2.add_path(k);
    }
    while app2.probing() {
        app2.tick_probe();
    }
    app2.profile.preset = Preset::Mp3V0;
    assert_eq!(app2.files.len(), 5);
    let plan2 = app2.job_plan();
    assert_eq!(plan2.runnable.len(), 0, "hepsi atlanmali");
    assert_eq!(plan2.specs.iter().filter(|s| s.skip).count(), 5);
    app2.start();
    assert!(!app2.running, "atlananlarla kuyruk baslamamali");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn video_presetleri_gecerli_komut_uretir() {
    if !ffmpeg_var() {
        return;
    }
    let ff = Ffmpeg::locate(Lang::Tr).unwrap();
    let dir = tmp("video");
    let src = dir.join("kaynak.mp4");
    let st = Command::new(&ff.ffmpeg)
        .args([
            "-nostdin",
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            "testsrc=size=160x120:rate=15:duration=2",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:duration=2",
            "-c:v",
            "libx264",
            "-preset",
            "ultrafast",
            "-c:a",
            "aac",
            "-shortest",
        ])
        .arg(&src)
        .status()
        .unwrap();
    if !st.success() {
        eprintln!("libx264 yok: video testi atlandi");
        return;
    }

    let mut app = App::new(Settings::default());
    app.add_path(&dir);
    while app.probing() {
        app.tick_probe();
    }
    assert_eq!(app.files.len(), 1);
    assert_eq!(app.files[0].kind, ffstudio_core::ffmpeg::Kind::Video);

    // H.264/MP4 + kucultme + CRF
    app.profile.preset = Preset::H264Mp4;
    app.profile.crf = 28;
    app.profile.max_width = 120;
    let cmd = app.preview_cmd_for(&app.files[0].clone());
    assert!(cmd.contains("libx264"), "codec eksik: {cmd}");
    assert!(cmd.contains("scale=120:-2"), "kucultme eksik: {cmd}");
    // mp4 -> mp4 ayni klasorde: kaynak EZILMEZ, yeni ad kullanilir
    assert!(
        cmd.contains("kaynak_donusen.mp4"),
        "cakismada guvenli ad beklenir: {cmd}"
    );
    let src_len_before = std::fs::metadata(&src).unwrap().len();

    app.settings.workers = 1;
    app.start();
    assert!(wait_done(&mut app, Duration::from_secs(180)));
    let (_, _, done, fail) = app.counts();
    assert_eq!(
        (done, fail),
        (1, 0),
        "video donusumu basarisiz; gunluk:\n{}",
        app.log
            .iter()
            .map(|(l, _)| l.clone())
            .collect::<Vec<_>>()
            .join("\n")
    );

    let out = dir.join("kaynak_donusen.mp4");
    assert!(out.is_file(), "cikti yok: {}", out.display());
    let codec = probe_video_codec(&ff.ffprobe, &out);
    assert_eq!(codec, "h264", "cikti h264 olmali, bulunan: {codec}");
    // kaynak dosyaya dokunulmadi
    assert_eq!(
        std::fs::metadata(&src).unwrap().len(),
        src_len_before,
        "kaynak dosya degismemeli"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
