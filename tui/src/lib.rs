//! FF Studio TUI — Termux / Linux istemcisi (kutuphane + binary).
//!
//! Masaustu GUI'siyle AYNI cekirdegi (`core`) kullanir: ffmpeg cagrilari,
//! preset/arguman uretimi, CPU/paralellik plani ve ceviriler tek yerde.
//!
//! Kurulum (Termux):
//!   pkg install rust ffmpeg termux-api
//!   cargo build --release -p tui
//!   ./target/release/ffstudio-tui
//!
//! NDK/cross-compile YOKTUR: Termux'un kendi Rust'i ile derlenir.

pub mod app;
pub mod browse;
pub mod config;
pub mod errors;
pub mod storage;
pub mod ui;

use std::io::{self, Stdout};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use crossterm::cursor;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use ffstudio_core::lang::Lang;
use ffstudio_core::profiles::Preset;

use crate::app::{App, Focus, InputState, InputTarget, Mode};
use crate::browse::PickMode;
use crate::config::Settings;

/// SIGINT (Ctrl+C) geldi mi? Ana dongu bunu gorup temiz cikar.
static INTERRUPTED: AtomicBool = AtomicBool::new(false);

/// Ctrl+C ile kesildi mi?
pub fn interrupted() -> bool {
    INTERRUPTED.load(Ordering::SeqCst)
}

/// Panik olursa terminal raw mode'da kalmasin: kullanicinin shell'i bozulmaz.
///
/// ratatui/crossterm uygulamalarinda panik, raw mode + alternate screen
/// acikken olursa terminal kullanilamaz hale gelebiliyor; bu hook cikista
/// her seyi geri yukler.
fn install_panic_hook() {
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore_terminal();
        prev(info);
    }));
}

/// Ctrl+C aninda yapilacaklar: bayragi isaretle + aktif ffmpeg sureclerini durdur.
///
/// Iki yoldan da cagrilir:
///   - gercek SIGINT sinyali (disaridan `kill -INT`, Windows'ta Ctrl+C)
///   - terminal raw mode'da gelen Ctrl+C TUSU (0x03)
/// Raw mode'da ISIG kapali oldugu icin Ctrl+C sinyal URETMEZ; tus olarak gelir.
pub(crate) fn interrupt_now() -> usize {
    INTERRUPTED.store(true, Ordering::SeqCst);
    ffstudio_core::ffmpeg::terminate_active(Duration::from_millis(1500))
}

/// Ctrl+C: aktif ffmpeg sureclerine SIGTERM gonder, temiz cikis iste.
fn install_signal_handler() {
    let _ = ctrlc::set_handler(|| {
        // Orphan ffmpeg kalmasin: once SIGTERM, kapanmayanlara SIGKILL
        let _ = interrupt_now();
    });
}

/// Terminali ilk haline dondur (raw mode kapat + alternate screen'den cik).
fn restore_terminal() {
    let _ = disable_raw_mode();
    let _ = execute!(io::stdout(), LeaveAlternateScreen, cursor::Show);
}

/// TUI'yi baslatir: terminali ayarlar, donguyu calistirir, cikista geri yukler.
///
/// Donen deger: `true` = Ctrl+C ile kesildi (cikis kodu 130).
pub fn run_tui() -> io::Result<bool> {
    install_panic_hook();
    install_signal_handler();

    let settings = Settings::load();
    let mut app = App::new(settings);
    let res = run(&mut app);

    // Ne olursa olsun: calisan ffmpeg kalmasi (orphan) engellenir
    let _ = ffstudio_core::ffmpeg::terminate_active(Duration::from_millis(1500));

    res.map(|_| interrupted())
}

/// Termux yardimci komutu: varsa calistir, yoksa sessizce gec.
pub fn termux(cmd: &str) {
    let _ = std::process::Command::new(cmd)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
}

fn run(app: &mut App) -> io::Result<()> {
    enable_raw_mode()?;
    let mut out: Stdout = io::stdout();
    execute!(out, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(out);
    let mut terminal = Terminal::new(backend)?;

    let res = event_loop(&mut terminal, app);

    restore_terminal();
    terminal.show_cursor()?;
    res
}

fn event_loop(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut App) -> io::Result<()> {
    loop {
        // Ctrl+C: temiz cikis (ffmpeg cocuklari sinyal isleyicisinde durdurulur)
        if interrupted() {
            break;
        }
        // tarama: her turda bir dosya ffprobe ile cozulur (UI donmaz)
        if !matches!(app.mode, Mode::Help) {
            app.tick_probe();
        }
        app.poll_jobs();
        terminal.draw(|f| crate::ui::draw(f, app))?;

        if !event::poll(Duration::from_millis(150))? {
            continue;
        }
        let ev = event::read()?;
        match ev {
            // Terminal boyutu degisti (Termux'ta pinch-zoom ile font buyutme/
            // kucultme de bunu tetikler). Bazi terminaller resize sonrasi ilk
            // frame'de ESKI boyutu cache'te tutuyor; bu yuzden acikca temizleyip
            // hemen yeniden ciziyoruz -> hicbir panel tasmaz/ust uste binmez.
            Event::Resize(cols, rows) => {
                if cols > 0 && rows > 0 {
                    terminal.resize(ratatui::layout::Rect::new(0, 0, cols, rows))?;
                }
                // Boyut degisti: "paneller sigmiyor" uyarisi yeniden gosterilir
                // (kullanici zoom out yaptiysa kosul zaten ortadan kalkar).
                app.fit_ack = false;
                terminal.clear()?;
                terminal.draw(|f| crate::ui::draw(f, app))?;
            }
            Event::Key(k) => {
                if k.kind != KeyEventKind::Press {
                    continue;
                }
                if handle_key(app, k) {
                    break;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

/// true donerse uygulamadan cikilir.
fn handle_key(app: &mut App, k: KeyEvent) -> bool {
    // Termux depolama izni yoksa: arayuz yerine hata ekrani var.
    // Sadece cikis / tekrar dene / dil / tema tuslari calisir.
    if app.storage_error.is_some() {
        match k.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('r') => app.recheck_storage(),
            KeyCode::Char('l') => {
                let l = if app.lang == Lang::Tr { Lang::En } else { Lang::Tr };
                app.set_lang(l);
                app.storage_error = crate::storage::health(app.lang);
            }
            KeyCode::Char('t') => {
                let i = crate::config::Theme::ALL
                    .iter()
                    .position(|t| *t == app.theme)
                    .unwrap_or(0);
                app.theme = crate::config::Theme::ALL[(i + 1) % crate::config::Theme::ALL.len()];
                app.settings.theme = app.theme;
                app.settings.save();
            }
            // "Paneller sigmiyor" uyarisini yoksay (uyari TUI'yi kapladigi
            // icin bu kapi acikken de calismali).
            KeyCode::Char('u') => app.fit_ack = true,
            _ => {}
        }
        return false;
    }

    // Metin girisi modu: her seyi yutar
    if app.input.is_some() {
        match k.code {
            KeyCode::Esc => app.input = None,
            KeyCode::Enter => commit_input(app),
            KeyCode::Backspace => {
                if let Some(i) = app.input.as_mut() {
                    i.buf.pop();
                }
            }
            KeyCode::Char(c) => {
                if let Some(i) = app.input.as_mut() {
                    i.buf.push(c);
                }
            }
            _ => {}
        }
        return false;
    }

    // Gezici modu: tam ekran
    // "Paneller sigmiyor" uyarisini yoksay: uyari TUI'yi kapladigi icin bu kol
    // metin girisi DISINDA her modda (gezici, popup, ana ekran) calisir.
    if matches!(k.code, KeyCode::Char('u')) {
        app.fit_ack = true;
        return false;
    }

    if matches!(app.mode, Mode::Browse) {
        return handle_browse_key(app, k);
    }

    // Cikis onayi
    if app.quit_confirm {
        match k.code {
            KeyCode::Char('e') | KeyCode::Char('E') | KeyCode::Char('y') | KeyCode::Enter => {
                app.settings.save();
                return true;
            }
            _ => app.quit_confirm = false,
        }
        return false;
    }

    // Popuplar
    match app.mode {
        Mode::Preset => {
            handle_preset_key(app, k);
            return false;
        }
        Mode::Settings => {
            handle_settings_key(app, k);
            return false;
        }
        Mode::Help => {
            if matches!(
                k.code,
                KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q') | KeyCode::Enter
            ) {
                app.mode = Mode::Main;
            }
            return false;
        }
        _ => {}
    }

    // Ana ekran
    match (k.code, k.modifiers) {
        (KeyCode::Char('q'), _) => {
            app.quit_confirm = true;
        }
        (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
            // Raw mode'da Ctrl+C sinyal degil tus olarak gelir: dogrudan
            // ffmpeg cocuklerini durdurup temiz cikar (onay sorulmaz).
            interrupt_now();
            return true;
        }
        (KeyCode::Esc, _) => {
            app.quit_confirm = true;
        }
        (KeyCode::Tab, _) => app.focus = app.focus.next(),
        (KeyCode::BackTab, _) => app.focus = app.focus.prev(),
        (KeyCode::Up, _) | (KeyCode::Char('k'), _) => match app.focus {
            Focus::Files => {
                if app.sel > 0 {
                    app.sel -= 1;
                }
            }
            _ => {}
        },
        (KeyCode::Down, _) | (KeyCode::Char('j'), _) => match app.focus {
            Focus::Files => {
                if app.sel + 1 < app.files.len() {
                    app.sel += 1;
                }
            }
            _ => {}
        },
        (KeyCode::Char('a'), _) => app.open_browse(PickMode::Files),
        (KeyCode::Char('d'), _) => app.open_browse(PickMode::Folder),
        (KeyCode::Char('p'), _) => {
            app.mode = Mode::Preset;
            // imleci aktif preset'e getir
            if let Some(i) = Preset::ALL.iter().position(|p| *p == app.profile.preset) {
                app.preset_sel = i;
            }
        }
        (KeyCode::Char('s'), _) => {
            app.mode = Mode::Settings;
            app.settings_sel = 0;
        }
        (KeyCode::Char('?'), _) | (KeyCode::F(1), _) => app.mode = Mode::Help,
        (KeyCode::Char('c'), _) => app.start(),
        (KeyCode::Enter, _) => app.start(),
        (KeyCode::Char('o'), _) => app.open_browse(PickMode::OutDir),
        (KeyCode::Char('r'), _) => {
            app.settings.src_action = app.settings.src_action.next();
            app.settings.save();
        }
        (KeyCode::Char('w'), _) => {
            app.settings.overwrite = !app.settings.overwrite;
            app.settings.save();
        }
        (KeyCode::Char('t'), _) => {
            // tema degistir (ayni zamanda ayarlardan da secilir)
            let i = crate::config::Theme::ALL
                .iter()
                .position(|t| *t == app.theme)
                .unwrap_or(0);
            app.theme = crate::config::Theme::ALL[(i + 1) % crate::config::Theme::ALL.len()];
            app.settings.theme = app.theme;
            app.settings.save();
        }
        (KeyCode::Char('l'), _) => {
            let l = if app.lang == Lang::Tr { Lang::En } else { Lang::Tr };
            app.set_lang(l);
        }
        (KeyCode::Char('x'), _) => {
            // secili dosyayi listeden cikar
            if app.sel < app.files.len() {
                app.files.remove(app.sel);
                if app.sel >= app.files.len() && app.sel > 0 {
                    app.sel -= 1;
                }
            }
        }
        (KeyCode::Char('C'), _) => {
            // ozel argumanlar
            app.input = Some(InputState {
                target: InputTarget::CustomArgs,
                buf: app.profile.custom_args.clone(),
            });
        }
        (KeyCode::Char('g'), _) => {
            // trim baslangic
            app.input = Some(InputState {
                target: InputTarget::TrimStart,
                buf: app.profile.trim_start.clone(),
            });
        }
        (KeyCode::Char('G'), _) => {
            // trim bitis
            app.input = Some(InputState {
                target: InputTarget::TrimEnd,
                buf: app.profile.trim_end.clone(),
            });
        }
        (KeyCode::Char('m'), _) => {
            app.input = Some(InputState {
                target: InputTarget::MergeName,
                buf: app.profile.merge_name.clone(),
            });
        }
        (KeyCode::Char('M'), _) => {
            // kaynak dosyalarin tasinacagi klasor (kaynak islemi = "tasi" iken)
            app.open_browse(PickMode::MoveDir);
        }
        (KeyCode::Char('+'), _) | (KeyCode::Char('='), _) => {
            // esik deger artir (preset'e gore)
            bump(app, 1);
        }
        (KeyCode::Char('-'), _) => bump(app, -1),
        _ => {}
    }
    false
}

fn handle_browse_key(app: &mut App, k: KeyEvent) -> bool {
    let Some(b) = app.browse.as_mut() else {
        app.mode = Mode::Main;
        return false;
    };
    match k.code {
        KeyCode::Esc => app.cancel_browse(),
        KeyCode::Up | KeyCode::Char('k') => b.up(),
        KeyCode::Down | KeyCode::Char('j') => b.down(),
        KeyCode::Char(' ') => b.toggle_mark(),
        KeyCode::Enter => b.enter(),
        KeyCode::Backspace | KeyCode::Left | KeyCode::Char('h') => b.go_up(),
        KeyCode::Char('a') => b.mark_all_files(),
        KeyCode::Char('n') => b.clear_marks(),
        KeyCode::Char('d') => {
            let r = b.confirm();
            app.apply_pick(r);
        }
        // 1..9: hizli baslangic klasorleri (Termux: /sdcard, /sdcard/Download...)
        KeyCode::Char(c) if c.is_ascii_digit() && c != '0' => {
            let i = (c as u8 - b'1') as usize;
            if let Some((_, p)) = crate::browse::quick_starts().get(i) {
                b.cur = p.clone();
                b.sel = 0;
                b.reload();
            }
        }
        _ => {}
    }
    false
}

fn handle_preset_key(app: &mut App, k: KeyEvent) {
    let n = Preset::ALL.len();
    match k.code {
        KeyCode::Esc => app.mode = Mode::Main,
        KeyCode::Char('q') => app.mode = Mode::Main,
        KeyCode::Up | KeyCode::Char('k') => {
            app.preset_sel = app.preset_sel.saturating_sub(1);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.preset_sel = (app.preset_sel + 1).min(n - 1);
        }
        KeyCode::Enter => {
            let p = Preset::ALL[app.preset_sel];
            // dosya turune uymayan preset secilemez (menude soluk gosterilir)
            let ok = app.files.is_empty() || app.files.iter().any(|m| p.applies_to(m.kind));
            if ok {
                app.profile.preset = p;
                // preset'in varsayilan CRF'i (kullanici degistirmediyse)
                if let Some(c) = p.default_crf() {
                    if !app.profile.crf_touched {
                        app.profile.crf = c;
                    }
                }
            }
            app.mode = Mode::Main;
        }
        _ => {}
    }
}

fn handle_settings_key(app: &mut App, k: KeyEvent) {
    let rows = 4;
    match k.code {
        KeyCode::Esc | KeyCode::Char('q') => app.mode = Mode::Main,
        KeyCode::Up | KeyCode::Char('k') => app.settings_sel = app.settings_sel.saturating_sub(1),
        KeyCode::Down | KeyCode::Char('j') => {
            app.settings_sel = (app.settings_sel + 1).min(rows - 1)
        }
        KeyCode::Left | KeyCode::Char('h') | KeyCode::Right | KeyCode::Char('l')
        | KeyCode::Enter | KeyCode::Char(' ') => {
            let dir = matches!(k.code, KeyCode::Left | KeyCode::Char('h')) as i32
                - matches!(k.code, KeyCode::Right | KeyCode::Char('l')) as i32;
            match app.settings_sel {
                0 => {
                    // CPU / paralel is: 0(oto) ..= 16
                    let v = app.settings.workers as i32 - dir;
                    app.settings.workers = v.clamp(0, 16) as u32;
                }
                1 => {
                    let l = if app.lang == Lang::Tr { Lang::En } else { Lang::Tr };
                    app.set_lang(l);
                }
                2 => {
                    let i = crate::config::Theme::ALL
                        .iter()
                        .position(|t| *t == app.theme)
                        .unwrap_or(0);
                    let n = crate::config::Theme::ALL.len() as i32;
                    let j = ((i as i32 - dir).rem_euclid(n)) as usize;
                    app.theme = crate::config::Theme::ALL[j];
                    app.settings.theme = app.theme;
                }
                _ => app.settings.overwrite = !app.settings.overwrite,
            }
            app.settings.save();
        }
        _ => {}
    }
}

fn commit_input(app: &mut App) {
    let Some(inp) = app.input.take() else {
        return;
    };
    match inp.target {
        InputTarget::CustomArgs => app.profile.custom_args = inp.buf,
        InputTarget::TrimStart => app.profile.trim_start = inp.buf,
        InputTarget::TrimEnd => app.profile.trim_end = inp.buf,
        InputTarget::MergeName => app.profile.merge_name = inp.buf,
    }
}

/// +/- ile preset parametresini degistir (CRF, bitrate, kalite, sure...).
fn bump(app: &mut App, delta: i32) {
    let pr = &mut app.profile;
    match pr.preset {
        Preset::Mp3V0 | Preset::Mp3V2 | Preset::Vorbis | Preset::Flac | Preset::Wav => {
            pr.opus_kbps = (pr.opus_kbps as i32 + delta * 8).clamp(64, 320) as u32;
        }
        Preset::Mp3Cbr => pr.mp3_cbr = (pr.mp3_cbr as i32 + delta * 16).clamp(128, 320) as u32,
        Preset::Aac => {
            let v = pr.aac_kbps as i32 + delta * 16;
            pr.aac_kbps = v.clamp(0, 320) as u32;
        }
        Preset::Opus => pr.opus_kbps = (pr.opus_kbps as i32 + delta * 8).clamp(64, 320) as u32,
        p if p.is_video() => {
            pr.crf = (pr.crf as i32 - delta).clamp(0, 51) as u32;
            pr.crf_touched = true;
            pr.video_audio_kbps =
                (pr.video_audio_kbps as i32 + delta * 16).clamp(64, 320) as u32;
        }
        p if p.is_image() => {
            pr.img_quality = (pr.img_quality as i32 + delta * 5).clamp(1, 100) as u32;
        }
        Preset::TargetSize => pr.target_mb = (pr.target_mb as i32 + delta).clamp(1, 4096) as u32,
        // NOT: Clip BURADA DEGIL: Clip de -crf kullanir, asagidaki is_video
        // dalinda CRF/ses bitrate'i degisir. (Onceden Clip de bu dala dusuyordu
        // ve +/- gorunmez gif_fps/gif_width'i degistirdigi icin etkisizdi.)
        Preset::Gif => {
            pr.gif_fps = (pr.gif_fps as i32 + delta).clamp(5, 30) as u32;
            pr.gif_width = (pr.gif_width as i32 + delta * 40).clamp(160, 1920) as u32;
        }
        Preset::Split => pr.split_secs = (pr.split_secs as i32 + delta * 10).clamp(10, 3600) as u32,
        _ => {}
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicBool;
    use std::sync::Arc;

    /// Panik hook'u zincirlemeli: once terminali geri yukler, sonra eski
    /// (varsayilan) hook'u cagirir. Boylece panikte shell bozulmaz.
    #[test]
    fn panikte_terminal_geri_yuklenir_ve_eski_hook_calisir() {
        let called = Arc::new(AtomicBool::new(false));
        let c = called.clone();
        // gurultuyu kesen sahte "onceki" hook
        std::panic::set_hook(Box::new(move |_| {
            c.store(true, Ordering::SeqCst);
        }));
        install_panic_hook();
        let r = std::panic::catch_unwind(|| panic!("test panik"));
        assert!(r.is_err(), "panik yakalanmali");
        assert!(
            called.load(Ordering::SeqCst),
            "onceki hook (varsayilan davranis) cagrilmali"
        );
        let _ = std::panic::take_hook(); // temizlik
    }

    /// NO_COLOR doluysa palet renksiz olmali.
    #[test]
    fn no_color_renksiz_palet() {
        let p = crate::ui::Palette::plain();
        assert!(p.plain);
        assert_eq!(p.accent, ratatui::style::Color::Reset);
    }

    // ------------------------------------------------------------------
    // "MP3 CBR bitrate secimi" ve "kaynagi tasi -> hedef klasor" akislari
    // ------------------------------------------------------------------

    fn tus(kod: KeyCode) -> KeyEvent {
        KeyEvent::new(kod, KeyModifiers::NONE)
    }

    /// Bu testler 'r' tusuyla ayar kaydettigi icin config gecici klasore
    /// yonlendirilir; kullanicinin gercek ayar dosyasi ezilmez.
    fn gecici_config() -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("ffstudio_tui_cfg_{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        std::env::set_var("XDG_CONFIG_HOME", &d);
        d
    }

    /// SORU 1: MP3 CBR bitrate seviyesi AYRI bir state'te tutulur mu?
    /// Cevap: evet -> `Profile::mp3_cbr` (core/src/profiles.rs:259, varsayilan
    /// 320 -> satir 296). '+'/'-' tuslari `bump()` (tui/src/lib.rs:490,496)
    /// uzerinden 16'lik adimlarla 128..=320 araliginda degistirir. Ayri bir
    /// "320/256/192/160/128" listesi YOKTUR; bu 5 seviyeye adimlarla tam
    /// isabet edilir. Ekranda "Bitrate: N kbps" olarak cizilir
    /// (tui/src/ui.rs:473-474), ffmpeg'e `-b:a {N}k` olarak gider
    /// (core/src/profiles.rs:414).
    #[test]
    fn mp3_cbr_bitrate_ayari_adimlarla_degisir() {
        let _cfg = gecici_config();
        let mut app = App::new(Settings::default());

        // 'p' preset menusunu acar; Preset::ALL[3] == Mp3Cbr
        assert!(!handle_key(&mut app, tus(KeyCode::Char('p'))), "p tusu cikis degil");
        assert!(matches!(app.mode, Mode::Preset), "'p' preset menusunu acmali");
        for _ in 0..3 {
            handle_key(&mut app, tus(KeyCode::Down));
        }
        assert_eq!(Preset::ALL[app.preset_sel], Preset::Mp3Cbr);
        handle_key(&mut app, tus(KeyCode::Enter));
        assert!(matches!(app.mode, Mode::Main), "secim sonrasi ana ekran");

        // state: mp3_cbr, varsayilan 320
        assert_eq!(app.profile.preset, Preset::Mp3Cbr);
        assert_eq!(app.profile.mp3_cbr, 320, "MP3 CBR varsayilani 320");

        // '+' tavanda kalir
        handle_key(&mut app, tus(KeyCode::Char('+')));
        assert_eq!(app.profile.mp3_cbr, 320, "320'nin uzerine cikmamali");

        // '-' x4 -> 256 (istenen 5 seviyeden biri)
        for _ in 0..4 {
            handle_key(&mut app, tus(KeyCode::Char('-')));
        }
        assert_eq!(app.profile.mp3_cbr, 256);

        // alt sinir 128
        for _ in 0..20 {
            handle_key(&mut app, tus(KeyCode::Char('-')));
        }
        assert_eq!(app.profile.mp3_cbr, 128, "alt sinir 128");

        // 5 seviyenin tamami '+'/'-' ile tam isabet edilebilir mi?
        for _ in 0..20 {
            handle_key(&mut app, tus(KeyCode::Char('+')));
        }
        assert_eq!(app.profile.mp3_cbr, 320, "ust sinir 320");
        let mut gorulen = vec![320u32];
        for _ in 0..12 {
            handle_key(&mut app, tus(KeyCode::Char('-')));
            gorulen.push(app.profile.mp3_cbr);
        }
        for hedef in [320u32, 256, 192, 160, 128] {
            assert!(gorulen.contains(&hedef), "{hedef} kbps erisilebilir olmali");
        }

        // arayuz etiketleri (ui.rs:473 ve profiles.rs:752)
        let etiket = app.profile.preset.label(Lang::Tr);
        assert!(etiket.contains("320/256/192/160/128"), "preset basligi: {etiket}");
        let ozet = ffstudio_core::profiles::desc_of(app.profile.preset, &app.profile, Lang::Tr);
        assert!(ozet.contains("CBR"), "profil ozeti: {ozet}");
    }

    /// Paneller ekrana sigmadiginda cikan uyari 'u' ile yok sayilir; resize
    /// oldugunda (pinch-zoom) yeniden gosterilir.
    #[test]
    fn sigdirma_uyarisi_u_ile_yoksayilir() {
        let _cfg = gecici_config();
        let mut app = App::new(Settings::default());
        assert!(!app.fit_ack, "baslangicta uyari aktif");
        handle_key(&mut app, tus(KeyCode::Char('u')));
        assert!(app.fit_ack, "'u' uyariyi yoksaymali");
    }

    /// SORU 2: "Kaynagi tasi" secilip onaylandiginda hedef klasor sorulur mu?
    /// Cevap: evet, ayri bir adim vardir ama bu bir METIN input'u DEGIL, tam
    /// ekran klasor secicidir: 'M' -> `open_browse(PickMode::MoveDir)`
    /// (tui/src/lib.rs:361-364), secici 'd' ile onaylaninca
    /// `apply_pick` -> `settings.src_move_dir` (tui/src/app.rs:314-325).
    /// Klasor secilmemisse donusum sonrasi kaynak dosya TASINMAZ; log'a
    /// "Tasima icin klasor secilmedi - kaynak aynen birakildi." yazilir
    /// (tui/src/app.rs:724-731, core/src/lang.rs:996).
    #[test]
    fn tasi_secimi_hedef_klasor_ister() {
        let _cfg = gecici_config();
        let mut app = App::new(Settings::default());
        use crate::config::SrcAction;

        // 'r' dongusu: Keep -> Delete -> Move
        assert!(matches!(app.settings.src_action, SrcAction::Keep));
        handle_key(&mut app, tus(KeyCode::Char('r')));
        assert!(matches!(app.settings.src_action, SrcAction::Delete));
        handle_key(&mut app, tus(KeyCode::Char('r')));
        assert!(matches!(app.settings.src_action, SrcAction::Move), "'r' x2 -> Tasi");
        assert!(app.settings.src_move_dir.is_none(), "henuz klasor secilmedi");

        // 'M' -> tam ekran gezici, MoveDir modunda
        handle_key(&mut app, tus(KeyCode::Char('M')));
        assert!(matches!(app.mode, Mode::Browse), "'M' geziciyi acmali");
        assert!(
            matches!(app.browse.as_ref().unwrap().mode, PickMode::MoveDir),
            "gezici tasima-klasoru modunda olmali"
        );

        // 'd' -> o anki klasoru onayla
        let beklenen = app.browse.as_ref().unwrap().cur.clone();
        handle_key(&mut app, tus(KeyCode::Char('d')));
        assert_eq!(
            app.settings.src_move_dir.as_deref(),
            Some(beklenen.as_path()),
            "onaylanan klasor src_move_dir'e yazilmali"
        );
        assert!(matches!(app.mode, Mode::Main), "onay sonrasi ana ekran");

        // secim bir kez daha 'M' ile acildiginda ayni klasorden baslamali
        handle_key(&mut app, tus(KeyCode::Char('M')));
        assert_eq!(app.browse.as_ref().unwrap().cur, beklenen);
    }
}
