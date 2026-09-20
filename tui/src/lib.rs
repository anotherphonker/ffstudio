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
pub mod ui;

use std::io::{self, Stdout};
use std::time::Duration;

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

/// TUI'yi baslatir: terminali ayarlar, donguyu calistirir, cikista geri yukler.
pub fn run_tui() -> io::Result<()> {
    let settings = Settings::load();
    let mut app = App::new(settings);
    run(&mut app)
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

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    res
}

fn event_loop(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut App) -> io::Result<()> {
    loop {
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
        if let Event::Key(k) = ev {
            if k.kind != KeyEventKind::Press {
                continue;
            }
            if handle_key(app, k) {
                break;
            }
        }
    }
    Ok(())
}

/// true donerse uygulamadan cikilir.
fn handle_key(app: &mut App, k: KeyEvent) -> bool {
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
            app.quit_confirm = true;
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
        Preset::Clip | Preset::Gif => {
            pr.gif_fps = (pr.gif_fps as i32 + delta).clamp(5, 30) as u32;
            pr.gif_width = (pr.gif_width as i32 + delta * 40).clamp(160, 1920) as u32;
        }
        Preset::Split => pr.split_secs = (pr.split_secs as i32 + delta * 10).clamp(10, 3600) as u32,
        _ => {}
    }
}

