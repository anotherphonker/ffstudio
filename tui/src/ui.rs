//! ratatui ile cizim.
//!
//! Dar terminalde (telefon) tek panel + sekme; genis terminalde uc sutun.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Row, Table, Wrap};
use ratatui::Frame;

use ffstudio_core::lang::{tr, Key};
use ffstudio_core::profiles::Preset;
use ffstudio_core::util;

use crate::app::{App, Focus, JobState, Mode};
use crate::browse::{self, PickMode};
use crate::config::Theme;

/// Sabit renk semalari (terminal-safe; RGB slider yok).
pub struct Palette {
    pub bg: Color,
    pub fg: Color,
    pub accent: Color,
    pub dim: Color,
    pub ok: Color,
    pub err: Color,
    pub warn: Color,
    pub sel: Color,
    /// NO_COLOR: renk yok, vurgular ters-video (REVERSED) ile yapilir
    pub plain: bool,
}

impl Palette {
    /// NO_COLOR icin: hicbir renk kullanilmaz, vurgu ters-video ile verilir.
    pub fn plain() -> Self {
        Palette {
            bg: Color::Reset,
            fg: Color::Reset,
            accent: Color::Reset,
            dim: Color::Reset,
            ok: Color::Reset,
            err: Color::Reset,
            warn: Color::Reset,
            sel: Color::Reset,
            plain: true,
        }
    }

    /// Uygulamaya gore palet: NO_COLOR doluysa renksiz, degilse secili tema.
    pub fn for_app(app: &App) -> Self {
        if app.no_color {
            Self::plain()
        } else {
            Self::of(app.theme)
        }
    }

    /// Secili satir stili.
    pub fn selected(&self) -> Style {
        if self.plain {
            Style::default().add_modifier(Modifier::REVERSED | Modifier::BOLD)
        } else {
            Style::default().bg(self.sel).add_modifier(Modifier::BOLD)
        }
    }

    /// Vurgu "cip"i (baslik, aktif sekme).
    pub fn chip(&self) -> Style {
        if self.plain {
            Style::default().add_modifier(Modifier::REVERSED | Modifier::BOLD)
        } else {
            Style::default().fg(self.bg).bg(self.accent).add_modifier(Modifier::BOLD)
        }
    }

    pub fn of(theme: Theme) -> Self {
        match theme {
            Theme::Default => Palette {
                bg: Color::Rgb(14, 18, 28),
                fg: Color::Rgb(224, 228, 236),
                accent: Color::Rgb(91, 157, 255),
                dim: Color::Rgb(120, 130, 150),
                ok: Color::Rgb(110, 205, 130),
                err: Color::Rgb(240, 110, 110),
                warn: Color::Rgb(230, 180, 90),
                sel: Color::Rgb(40, 60, 95),
                plain: false,
            },
            Theme::Gruvbox => Palette {
                bg: Color::Rgb(40, 40, 40),
                fg: Color::Rgb(235, 219, 178),
                accent: Color::Rgb(250, 189, 47),
                dim: Color::Rgb(146, 131, 116),
                ok: Color::Rgb(184, 187, 38),
                err: Color::Rgb(251, 73, 52),
                warn: Color::Rgb(254, 128, 25),
                sel: Color::Rgb(80, 73, 69),
                plain: false,
            },
            Theme::Mono => Palette {
                bg: Color::Black,
                fg: Color::White,
                accent: Color::Gray,
                dim: Color::DarkGray,
                ok: Color::White,
                err: Color::White,
                warn: Color::Gray,
                sel: Color::DarkGray,
                plain: false,
            },
            Theme::Solarized => Palette {
                bg: Color::Rgb(0, 43, 54),
                fg: Color::Rgb(147, 161, 161),
                accent: Color::Rgb(38, 139, 210),
                dim: Color::Rgb(88, 110, 117),
                ok: Color::Rgb(133, 153, 0),
                err: Color::Rgb(220, 50, 47),
                warn: Color::Rgb(181, 137, 0),
                sel: Color::Rgb(7, 54, 66),
                plain: false,
            },
        }
    }
}

/// Ceviri metnini etiket olarak bicimle: sonunda ':' yoksa ekle.
/// (Cevirilerin bir kismi zaten ':' ile bitiyor -> cift iki nokta olmasin.)
fn lab(lang: ffstudio_core::lang::Lang, k: Key) -> String {
    let s = tr(lang, k);
    if s.ends_with(':') {
        s.to_string()
    } else {
        format!("{s}:")
    }
}

/// Listede gorunur pencereyi hesapla (seciliyi ortala).
fn window(len: usize, sel: usize, height: usize) -> (usize, usize) {
    if height == 0 || len <= height {
        return (0, len);
    }
    let half = height / 2;
    let start = sel.saturating_sub(half).min(len - height);
    (start, start + height)
}

/// Bu boyutun altinda normal arayuz CIZILMEZ (bkz. `draw`).
/// (Termux'ta pinch-zoom ile font buyutulunce satir/sutun sayisi duser.)
pub const MIN_COLS: u16 = 20;
pub const MIN_ROWS: u16 = 8;

/// Ust/yan panellerin toplam yuzdesi; artani ESMEYEN govde paneli alir.
/// Ornek: 13+15+13+4 = %45 yan/ust bantlar, kalan ~%55 govde.
const PCT_HEADER: u16 = 13;
const PCT_QUEUE: u16 = 15;
const PCT_LOG: u16 = 13;
const PCT_STATUS: u16 = 4;

/// Ana ekran satir plani.
///
/// SABIT `Length` YOK: sabitler yuzde olarak verilir, artan satirlar
/// `Min` ile esneyen govde paneline gider. Boylece terminal buyuyup
/// kuculdukce (pinch-zoom, pencere boyutu) hicbir panel tasmaz,
/// ust uste binmez; toplam her zaman tam olarak alan yuksekligine esittir.
pub fn main_layout(area: Rect) -> std::rc::Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(PCT_HEADER), // baslik
            Constraint::Min(3),                 // govde (esneyen)
            Constraint::Percentage(PCT_QUEUE),  // kuyruk
            Constraint::Percentage(PCT_LOG),    // log
            Constraint::Percentage(PCT_STATUS), // durum satiri
        ])
        .split(area)
}

pub fn draw(f: &mut Frame, app: &mut App) {
    let p = Palette::for_app(app);
    let area = f.area();

    // Renksiz (NO_COLOR) modda arka plan da boyanmaz
    if !p.plain {
        f.render_widget(Block::default().style(Style::default().bg(p.bg)), area);
    }

    // 1) Terminal cok kucukse: kirik layout yerine tek satir mesaj.
    //    (Termux'ta pinch-zoom ile font buyutulunce satir/sutun sayisi duser;
    //     bu koruma olmadan paneller ust uste binerdi.)
    if area.width < MIN_COLS || area.height < MIN_ROWS {
        let msg = format!(
            "{} — {}x{} (min {}x{})",
            tr(app.lang, Key::TooSmallTitle),
            area.width,
            area.height,
            MIN_COLS,
            MIN_ROWS
        );
        let satir = Line::from(Span::styled(
            util::truncate(&msg, area.width as usize),
            Style::default().fg(p.warn).add_modifier(Modifier::BOLD),
        ));
        f.render_widget(
            Paragraph::new(satir)
                .alignment(ratatui::layout::Alignment::Center)
                .wrap(Wrap { trim: true }),
            Rect {
                x: area.x,
                y: area.y + area.height / 2,
                width: area.width,
                height: 1.min(area.height),
            },
        );
        return;
    }

    // 2) Termux depolama izni yoksa: dosya gezici yerine net Turkce hata ekrani
    if app.storage_error.is_some() {
        draw_storage_error(f, app, &p, area);
        return;
    }

    if matches!(app.mode, Mode::Browse) {
        draw_browse(f, app, &p);
        return;
    }

    let narrow = area.width < 110;
    let chunks = main_layout(area);

    draw_header(f, app, &p, chunks[0]);

    if narrow {
        // Dar ekran: odaktaki paneli tam boy ciz
        f.render_widget(
            tabs_line(app, &p, chunks[1]).block(Block::default().borders(Borders::BOTTOM)),
            chunks[1],
        );
        let inner = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(10), // sekme satiri
                Constraint::Min(2),         // panel (esneyen)
            ])
            .split(chunks[1]);
        let body = inner[1];
        match app.focus {
            Focus::Files => draw_files(f, app, &p, body),
            Focus::Profile => draw_profile(f, app, &p, body),
            Focus::Details => draw_details(f, app, &p, body),
            Focus::Queue => draw_queue(f, app, &p, body),
            Focus::Log => draw_log(f, app, &p, body),
        }
    } else {
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(26),
                Constraint::Percentage(41),
                Constraint::Percentage(33),
            ])
            .split(chunks[1]);
        draw_files(f, app, &p, cols[0]);
        draw_profile(f, app, &p, cols[1]);
        if app.files.is_empty() {
            draw_help_block(f, &p, cols[2]);
        } else {
            draw_details(f, app, &p, cols[2]);
        }
    }

    draw_queue(f, app, &p, chunks[2]);
    draw_log(f, app, &p, chunks[3]);
    draw_status(f, app, &p, chunks[4]);

    // Popuplar
    match app.mode {
        Mode::Preset => draw_preset_popup(f, app, &p),
        Mode::Settings => draw_settings_popup(f, app, &p),
        Mode::Help => draw_help_popup(f, app, &p),
        _ => {}
    }
    if app.input.is_some() {
        draw_input_popup(f, app, &p);
    }
    if app.quit_confirm {
        draw_quit_popup(f, app, &p);
    }
}

// ---------------------------------------------------------------------------
// Baslik / sekme / durum
// ---------------------------------------------------------------------------

fn draw_header(f: &mut Frame, app: &App, p: &Palette, area: Rect) {
    let ff_info = match &app.ff {
        Ok(_) => {
            // "ffmpeg version 7.1.5-0+deb13u1 Copyright (c) ..." -> "7.1.5-0+deb13u1"
            let v = app
                .ff_version
                .trim_start_matches("ffmpeg version ")
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_string();
            format!("ffmpeg {}", util::truncate(&v, 26))
        }
        Err(_) => tr(app.lang, Key::FfmpegMissing).to_string(),
    };
    let mut spans = vec![
        Span::styled(" FF Studio TUI ", p.chip()),
        Span::raw("  "),
        Span::styled(
            ff_info,
            Style::default().fg(if app.ff.is_ok() { p.dim } else { p.err }),
        ),
        Span::raw("   "),
        Span::styled(
            format!("{} {}", lab(app.lang, Key::LabelOutput), util::truncate(&app.out_label(), 40)),
            Style::default().fg(p.fg),
        ),
        Span::raw("   "),
        Span::styled(
            format!("{} {}", lab(app.lang, Key::LabelSrcAction), app.settings.src_action.label(app.lang)),
            Style::default().fg(p.fg),
        ),
    ];
    if app.probing() {
        spans.push(Span::styled(
            format!("   {} {}", tr(app.lang, Key::StScanning), app.pending_probe.len()),
            Style::default().fg(p.warn),
        ));
    }
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(p.dim));
    f.render_widget(
        Paragraph::new(Line::from(spans))
            .wrap(Wrap { trim: true })
            .block(block),
        area,
    );
}

fn tabs_line<'a>(app: &App, p: &Palette, _area: Rect) -> Paragraph<'a> {
    let names = [
        (Focus::Files, tr(app.lang, Key::SecFiles)),
        (Focus::Profile, tr(app.lang, Key::SecProfile)),
        (Focus::Details, tr(app.lang, Key::SecDetails)),
        (Focus::Queue, tr(app.lang, Key::SecQueue)),
        (Focus::Log, tr(app.lang, Key::LogHdr)),
    ];
    let mut spans: Vec<Span> = Vec::new();
    for (foc, name) in names {
        let active = app.focus == foc;
        spans.push(Span::styled(
            format!(" {name} "),
            if active {
                p.chip()
            } else {
                Style::default().fg(p.dim)
            },
        ));
        spans.push(Span::raw(" "));
    }
    Paragraph::new(Line::from(spans)).wrap(Wrap { trim: true })
}

fn draw_status(f: &mut Frame, app: &App, p: &Palette, area: Rect) {
    let hint = if app.running {
        "q: cikis (bekle)   Ctrl+C: durdur ve cik   Tab: panel"
    } else if app.files.is_empty() {
        "a: dosya/klasor ekle   ?: yardim   q: cikis   Ctrl+C: cikis"
    } else {
        "a:ekle  c:donustur  p:preset  o:cikti  r:kaynak  w:uzerine-yaz  s:ayarlar  ?:yardim  q:cikis"
    };
    let line = Line::from(vec![
        Span::styled(hint, Style::default().fg(p.dim)),
    ]);
    f.render_widget(Paragraph::new(line).wrap(Wrap { trim: true }), area);
}

// ---------------------------------------------------------------------------
// Dosyalar
// ---------------------------------------------------------------------------

fn draw_files(f: &mut Frame, app: &App, p: &Palette, area: Rect) {
    let title = format!(" {} ({}) ", tr(app.lang, Key::AddFile), app.files.len());
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(if app.focus == Focus::Files { p.accent } else { p.dim }));
    if app.files.is_empty() {
        let txt = Text::from(vec![
            Line::from(""),
            Line::from(Span::styled(
                format!("  {}", tr(app.lang, Key::NoFiles)),
                Style::default().fg(p.dim),
            )),
            Line::from(""),
            Line::from(Span::styled("  a = dosya / klasor ekle", Style::default().fg(p.dim))),
        ]);
        f.render_widget(Paragraph::new(txt).block(block).wrap(Wrap { trim: true }), area);
        return;
    }
    let inner_h = area.height.saturating_sub(2) as usize;
    let (start, end) = window(app.files.len(), app.sel, inner_h);
    let items: Vec<ListItem> = app.files[start..end]
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let idx = start + i;
            let selected = idx == app.sel;
            let badge = match m.kind {
                ffstudio_core::ffmpeg::Kind::Audio => ("SES", Color::Rgb(96, 165, 250)),
                ffstudio_core::ffmpeg::Kind::Video => ("VID", Color::Rgb(167, 139, 250)),
                ffstudio_core::ffmpeg::Kind::Image => ("RES", Color::Rgb(52, 211, 153)),
            };
            let style = if selected { p.selected() } else { Style::default() };
            ListItem::new(Line::from(vec![
                Span::styled(format!("{} ", if selected { ">" } else { " " }), style),
                Span::styled(format!("[{}] ", badge.0), Style::default().fg(badge.1)),
                Span::styled(util::truncate(&m.name, 40), style),
                Span::styled(
                    format!("  {}", util::fmt_size(m.size)),
                    Style::default().fg(p.dim),
                ),
            ]))
            .style(style)
        })
        .collect();
    f.render_widget(List::new(items).block(block), area);
}

// ---------------------------------------------------------------------------
// Profil (preset + ayarlar + komut onizleme)
// ---------------------------------------------------------------------------

fn draw_profile(f: &mut Frame, app: &App, p: &Palette, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", tr(app.lang, Key::SecProfile)))
        .border_style(Style::default().fg(if app.focus == Focus::Profile { p.accent } else { p.dim }));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(62), // preset/ayar satirlari
            Constraint::Percentage(38), // komut onizlemesi
        ])
        .split(inner);
    let mut lines: Vec<Line> = Vec::new();
    let l = app.lang;
    let pr = &app.profile;

    lines.push(Line::from(vec![
        Span::styled(format!("{} ", lab(l, Key::LabelPreset)), Style::default().fg(p.dim)),
        Span::styled(
            pr.preset.label(l),
            Style::default().fg(p.accent).add_modifier(Modifier::BOLD),
        ),
        Span::styled("   (p)", Style::default().fg(p.dim)),
    ]));
    lines.push(Line::from(Span::styled(
        ffstudio_core::profiles::desc_of(pr.preset, pr, l),
        Style::default().fg(p.dim),
    )));
    lines.push(Line::from(""));

    match pr.preset {
        Preset::Mp3Cbr => {
            lines.push(kv(p, tr(l, Key::LabelBitrate), &format!("{} kbps", pr.mp3_cbr)));
        }
        Preset::Aac => {
            lines.push(kv(
                p,
                tr(l, Key::LabelBitrate),
                &if pr.aac_kbps == 0 { "VBR".to_string() } else { format!("{} kbps", pr.aac_kbps) },
            ));
        }
        Preset::Opus => {
            lines.push(kv(p, tr(l, Key::LabelBitrate), &format!("{} kbps", pr.opus_kbps)));
        }
        Preset::Flac | Preset::Wav => {}
        p2 if p2.is_video() => {
            lines.push(kv(p, tr(l, Key::Crf), &format!("{}", pr.crf)));
            lines.push(kv(
                p,
                tr(l, Key::EncodeSpeed),
                ffstudio_core::profiles::X264_PRESETS
                    .get(pr.x264_preset as usize)
                    .copied()
                    .unwrap_or("medium"),
            ));
            lines.push(kv(
                p,
                tr(l, Key::AudioInVideo),
                &format!("{} kbps", pr.video_audio_kbps),
            ));
            lines.push(kv(p, tr(l, Key::MaxWidth), &format!("{}", if pr.max_width == 0 { 0 } else { pr.max_width })));
        }
        p2 if p2.is_image() => {
            lines.push(kv(p, tr(l, Key::Quality), &format!("{}", pr.img_quality)));
            lines.push(kv(p, tr(l, Key::MaxWidth), &format!("{}", pr.max_width)));
        }
        _ => {}
    }
    if pr.preset == Preset::TargetSize {
        lines.push(kv(p, tr(l, Key::TargetSize), &format!("{} MB", pr.target_mb)));
    }
    if pr.preset == Preset::Clip || pr.preset == Preset::Gif {
        lines.push(kv(p, tr(l, Key::TrimStart), if pr.trim_start.is_empty() { "-" } else { &pr.trim_start }));
        lines.push(kv(p, tr(l, Key::TrimEnd), if pr.trim_end.is_empty() { "-" } else { &pr.trim_end }));
    }
    if pr.preset == Preset::Gif {
        lines.push(kv(p, tr(l, Key::GifWidthLabel), &format!("{}", pr.gif_width)));
        lines.push(kv(p, tr(l, Key::GifFpsLabel), &format!("{}", pr.gif_fps)));
    }
    if pr.preset == Preset::Split {
        lines.push(kv(p, tr(l, Key::SplitSecsLabel), &format!("{}", pr.split_secs)));
    }
    if pr.preset == Preset::Merge {
        lines.push(kv(p, tr(l, Key::MergeNameLabel), if pr.merge_name.is_empty() { "-" } else { &pr.merge_name }));
        lines.push(kv(p, tr(l, Key::Reencode), if pr.merge_reencode { "açık" } else { "kapalı" }));
    }
    if pr.preset == Preset::CoverArt {
        lines.push(kv(p, tr(l, Key::CoverPng), if pr.cover_png { "PNG" } else { "JPEG" }));
    }
    if pr.preset == Preset::Remux {
        lines.push(kv(p, tr(l, Key::DContainer), &format!("{}", pr.remux_container)));
    }
    lines.push(kv(p, tr(l, Key::CustomArgs), if pr.custom_args.is_empty() { "-" } else { &pr.custom_args }));
    lines.push(Line::from(Span::styled(
        format!(
            "  [{}] / [{}]",
            tr(l, Key::LabelSrcAction),
            tr(l, Key::Overwrite)
        ),
        Style::default().fg(p.dim),
    )));

    f.render_widget(
        Paragraph::new(Text::from(lines))
            .block(Block::default())
            .wrap(Wrap { trim: true }),
        chunks[0],
    );

    // Komut onizleme: ekranda gorunen = calistirilan
    let preview = match app.selected_media() {
        Some(m) => app.preview_cmd_for(m),
        None => format!("({})", tr(l, Key::NoFiles)),
    };
    let mut plines = vec![Line::from(Span::styled(
        tr(l, Key::CmdPreviewSel),
        Style::default().fg(p.dim),
    ))];
    for seg in wrap(&preview, chunks[1].width.saturating_sub(2) as usize) {
        plines.push(Line::from(Span::styled(seg, Style::default().fg(p.fg))));
    }
    f.render_widget(
        Paragraph::new(Text::from(plines))
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .border_style(Style::default().fg(p.dim)),
            )
            .wrap(Wrap { trim: true }),
        chunks[1],
    );
}

fn kv(p: &Palette, k: &str, v: &str) -> Line<'static> {
    let k = if k.ends_with(':') { k.to_string() } else { format!("{k}:") };
    Line::from(vec![
        Span::styled(format!("  {k} "), Style::default().fg(p.dim)),
        Span::styled(v.to_string(), Style::default().fg(p.fg)),
    ])
}

fn wrap(s: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![s.to_string()];
    }
    let mut out = Vec::new();
    let mut cur = String::new();
    for word in s.split(' ') {
        if cur.len() + word.len() + 1 > width && !cur.is_empty() {
            out.push(cur.clone());
            cur.clear();
        }
        if !cur.is_empty() {
            cur.push(' ');
        }
        cur.push_str(word);
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

// ---------------------------------------------------------------------------
// Detay + verim tablosu
// ---------------------------------------------------------------------------

fn draw_details(f: &mut Frame, app: &App, p: &Palette, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", tr(app.lang, Key::SecDetails)))
        .border_style(Style::default().fg(if app.focus == Focus::Details { p.accent } else { p.dim }));
    let inner = block.inner(area);
    f.render_widget(block, area);
    let Some(m) = app.selected_media() else {
        f.render_widget(
            Paragraph::new(Span::styled(tr(app.lang, Key::NoFiles), Style::default().fg(p.dim)))
                .wrap(Wrap { trim: true }),
            inner,
        );
        return;
    };
    // Ust: detaylar · Alt: bitrate & verim tablosu (oransal)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(58), // detaylar
            Constraint::Percentage(42), // verim tablosu
        ])
        .split(inner);

    let l = app.lang;
    let mut lines = vec![Line::from(Span::styled(
        m.name.clone(),
        Style::default().fg(p.accent).add_modifier(Modifier::BOLD),
    ))];
    for (k, v) in [
        (tr(l, Key::DContainer), m.container.clone()),
        (tr(l, Key::DDuration), util::fmt_dur(m.duration)),
        (tr(l, Key::DSizeLabel), util::fmt_size(m.size)),
    ] {
        lines.push(kv(p, k, &v));
    }
    if let Some(v) = &m.video {
        lines.push(kv(p, tr(l, Key::DVideo), &format!("{} ({})", v.codec, v.profile)));
        lines.push(kv(
            p,
            tr(l, Key::DResolution),
            &format!("{}x{}", v.width, v.height),
        ));
        lines.push(kv(p, tr(l, Key::Fps), &format!("{:.3}", v.fps)));
        lines.push(kv(p, tr(l, Key::DPixFmt), &v.pix_fmt));
        if let Some(b) = v.bitrate {
            lines.push(kv(p, tr(l, Key::DVideoBitrate), &format!("{:.0} kbps", b as f64 / 1000.0)));
        }
    }
    if let Some(a) = &m.audio {
        lines.push(kv(p, tr(l, Key::DAudio), &format!("{}", a.codec)));
        lines.push(kv(
            p,
            tr(l, Key::DAudioRate),
            &format!("{} Hz, {} {}", a.sample_rate, a.channels, tr(l, Key::UnitChannel)),
        ));
        if let Some(b) = a.bitrate {
            lines.push(kv(p, tr(l, Key::DAudioBitrate), &format!("{:.0} kbps", b as f64 / 1000.0)));
        }
    }
    for e in &m.extra {
        lines.push(Line::from(Span::styled(format!("  {e}"), Style::default().fg(p.dim))));
    }
    f.render_widget(
        Paragraph::new(Text::from(lines)).wrap(Wrap { trim: true }),
        chunks[0],
    );
    render_eff_table(f, app, p, chunks[1]);
}

/// Bitrate & verim tablosu (ratatui::Table).
pub fn efficiency_rows(app: &App) -> Vec<Row<'static>> {
    let l = app.lang;
    app.files
        .iter()
        .map(|m| {
            // dar panel: degerleri kisalt (115 kbps -> 115k, 80.6 KB -> 80.6K)
            let src = util::src_kbps(m)
                .map(|k| format!("{k}k"))
                .unwrap_or_else(|| "-".into());
            let (est, mode) = ffstudio_core::profiles::estimate(&app.profile, m, l);
            let est_s = est
                .map(|e| util::fmt_size(e).replace(' ', ""))
                .unwrap_or_else(|| "-".to_string());
            let diff = match est {
                Some(e) if m.size > 0 => {
                    let pct = (e as f64 / m.size as f64 - 1.0) * 100.0;
                    format!("{pct:+.0}%")
                }
                _ => "-".to_string(),
            };
            Row::new(vec![
                util::truncate(&m.name, 14),
                src,
                util::truncate(&mode, 8),
                est_s,
                diff,
            ])
        })
        .collect()
}

fn draw_help_block(f: &mut Frame, p: &Palette, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title(" Yardim ");
    let txt = Text::from(vec![
        Line::from(Span::styled("a  dosya/klasor ekle", Style::default().fg(p.dim))),
        Line::from(Span::styled("p  preset sec", Style::default().fg(p.dim))),
        Line::from(Span::styled("c  donustur", Style::default().fg(p.dim))),
        Line::from(Span::styled("?  tum kisayollar", Style::default().fg(p.dim))),
    ]);
    f.render_widget(Paragraph::new(txt).wrap(Wrap { trim: true }).block(block), area);
}

// ---------------------------------------------------------------------------
// Kuyruk / Log
// ---------------------------------------------------------------------------

fn draw_queue(f: &mut Frame, app: &App, p: &Palette, area: Rect) {
    let (run, que, done, fail) = app.counts();
    let title = format!(
        " {}: {} {} · {} {} · {} {} · {} {} ",
        tr(app.lang, Key::SecQueue),
        run,
        tr(app.lang, Key::StRunning),
        que,
        tr(app.lang, Key::UnitQueue),
        done,
        tr(app.lang, Key::UnitFinished),
        fail,
        tr(app.lang, Key::UnitFailed)
    );
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(if app.focus == Focus::Queue { p.accent } else { p.dim }));
    let inner = block.inner(area);
    f.render_widget(block, area);
    if app.jobs.is_empty() {
        f.render_widget(
            Paragraph::new(Span::styled(
                format!("  {}", tr(app.lang, Key::QueueEmpty)),
                Style::default().fg(p.dim),
            ))
            .wrap(Wrap { trim: true }),
            inner,
        );
        return;
    }
    let inner_h = inner.height as usize;
    let (start, end) = window(app.jobs.len(), 0, inner_h);
    let mut lines: Vec<Line> = Vec::new();
    for j in &app.jobs[start..end] {
        let (label, style) = match &j.state {
            JobState::Queued => (tr(app.lang, Key::Queued).to_string(), Style::default().fg(p.dim)),
            JobState::Running => (
                format!("{:.0}% {}", j.frac * 100.0, j.speed.clone().unwrap_or_default()),
                Style::default().fg(p.accent),
            ),
            JobState::Done => (tr(app.lang, Key::Done).to_string(), Style::default().fg(p.ok)),
            JobState::Skipped(r) => (
                format!("{}: {r}", tr(app.lang, Key::SkippedPrefix)),
                Style::default().fg(p.warn),
            ),
            JobState::Failed(m) => (
                format!("{} {}", tr(app.lang, Key::ErrorPrefix), util::truncate(m, 40)),
                Style::default().fg(p.err),
            ),
        };
        lines.push(Line::from(vec![
            Span::styled(
                format!(" {:<28}", util::truncate(&j.name, 28)),
                Style::default().fg(p.fg),
            ),
            Span::styled(format!("{:<18}", util::truncate(&j.target, 18)), Style::default().fg(p.dim)),
            Span::styled(format!("{:<10}", util::fmt_size(j.src_size)), Style::default().fg(p.dim)),
            Span::styled(label, style),
        ]));
    }
    f.render_widget(Paragraph::new(Text::from(lines)).wrap(Wrap { trim: true }), inner);
}

fn draw_log(f: &mut Frame, app: &App, p: &Palette, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", tr(app.lang, Key::LogHdr)))
        .border_style(Style::default().fg(if app.focus == Focus::Log { p.accent } else { p.dim }));
    let inner = block.inner(area);
    f.render_widget(block, area);
    let h = inner.height as usize;
    let lines: Vec<Line> = app
        .log
        .iter()
        .rev()
        .take(h)
        .rev()
        .map(|(line, is_err)| {
            Line::from(Span::styled(
                line.clone(),
                Style::default().fg(if *is_err { p.err } else { p.dim }),
            ))
        })
        .collect();
    f.render_widget(Paragraph::new(Text::from(lines)).wrap(Wrap { trim: true }), inner);
}

// ---------------------------------------------------------------------------
// Popuplar
// ---------------------------------------------------------------------------

/// Termux depolama izni yok: arayuz yerine NET Turkce yonlendirme ekrani.
///
/// Ham OS hatasi (Permission denied / os error 13) asla gosterilmez;
/// mesaj `storage::health()`ten gelir (adim adim ne yapilacagini soyler).
fn draw_storage_error(f: &mut Frame, app: &App, p: &Palette, area: Rect) {
    let Some(msg) = &app.storage_error else {
        return;
    };
    let blok = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(p.err))
        .style(if p.plain {
            Style::default()
        } else {
            Style::default().bg(p.bg)
        })
        .title(format!(" {} ", tr(app.lang, Key::StorageTitle)));
    let inner = blok.inner(area);
    f.render_widget(blok, area);

    let mut lines: Vec<Line> = msg
        .lines()
        .map(|s| Line::from(Span::styled(s.to_string(), Style::default().fg(p.fg))))
        .collect();
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        tr(app.lang, Key::PressToRetry),
        Style::default().fg(p.warn).add_modifier(Modifier::BOLD),
    )));

    // dikeyde ortala, tasma yok
    let h = (lines.len() as u16).min(inner.height);
    let y = inner.y + inner.height.saturating_sub(h) / 2;
    f.render_widget(
        Paragraph::new(Text::from(lines))
            .alignment(ratatui::layout::Alignment::Center)
            .wrap(Wrap { trim: true }),
        Rect {
            x: inner.x + 1.min(inner.width),
            y,
            width: inner.width.saturating_sub(2),
            height: h,
        },
    );
}

/// Yuzdeye gore ama min/max sinirli kutu: kucuk terminalde de tasmaz.
pub fn pct(area: Rect, yuzde: u16, min: u16, max: u16) -> u16 {
    ((area.width as u32 * yuzde as u32 / 100) as u16).clamp(min, max).min(area.width)
}

/// Yukseklik icin ayni mantik (satir sayisina gore de sinirlanabilir).
pub fn pct_h(area: Rect, yuzde: u16, min: u16, max: u16) -> u16 {
    ((area.height as u32 * yuzde as u32 / 100) as u16).clamp(min, max).min(area.height)
}

fn centered(area: Rect, w: u16, h: u16) -> Rect {
    let w = w.min(area.width);
    let h = h.min(area.height);
    Rect {
        x: area.x + (area.width - w) / 2,
        y: area.y + (area.height - h) / 2,
        width: w,
        height: h,
    }
}

fn draw_preset_popup(f: &mut Frame, app: &App, p: &Palette) {
    let h = (Preset::ALL.len() as u16 + 4).min(pct_h(f.area(), 80, 6, f.area().height));
    let area = centered(f.area(), pct(f.area(), 72, 30, 68), h);
    f.render_widget(Clear, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", tr(app.lang, Key::LabelPreset)))
        .border_style(Style::default().fg(p.accent))
        .style(Style::default().bg(p.bg));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let items: Vec<ListItem> = Preset::ALL
        .iter()
        .enumerate()
        .map(|(i, pr)| {
            let available = app.files.is_empty()
                || app.files.iter().any(|m| pr.applies_to(m.kind));
            let sel = i == app.preset_sel;
            let style = if !available {
                Style::default().fg(Color::DarkGray)
            } else if sel {
                p.selected()
            } else {
                Style::default().fg(p.fg)
            };
            let mark = if app.profile.preset == *pr { "*" } else { " " };
            ListItem::new(Line::from(vec![
                Span::styled(format!("{mark} "), style),
                Span::styled(format!("{:<26}", pr.label(app.lang)), style),
                Span::styled(
                    if available { "" } else { "(uygun degil)" },
                    Style::default().fg(Color::DarkGray),
                ),
            ]))
            .style(style)
        })
        .collect();
    let h = inner.height as usize;
    let (start, end) = window(items.len(), app.preset_sel, h);
    f.render_widget(List::new(items[start..end].to_vec()), inner);
}

fn draw_settings_popup(f: &mut Frame, app: &App, p: &Palette) {
    let area = centered(f.area(), pct(f.area(), 70, 34, 66), pct_h(f.area(), 70, 8, 16));
    f.render_widget(Clear, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", tr(app.lang, Key::SettingsTitle)))
        .border_style(Style::default().fg(p.accent))
        .style(Style::default().bg(p.bg));
    let inner = block.inner(area);
    f.render_widget(block, area);
    let l = app.lang;
    let workers = if app.settings.workers == 0 {
        format!("{} ({})", tr(l, Key::WorkersAuto), app.cpu.auto_workers())
    } else {
        format!("{}", app.settings.workers)
    };
    let rows = [
        (tr(l, Key::SecCpu), workers),
        (tr(l, Key::Language), format!("{}", if app.lang == ffstudio_core::lang::Lang::Tr { "TR" } else { "EN" })),
        (tr(l, Key::SecTheme), app.theme.name(l).to_string()),
        (tr(l, Key::Overwrite), format!("{}", if app.settings.overwrite { "acik" } else { "kapali" })),
    ];
    let mut lines = vec![];
    for (i, (k, v)) in rows.iter().enumerate() {
        let sel = i == app.settings_sel;
        lines.push(Line::from(vec![
            Span::styled(
                format!("{} {:<24}", if sel { ">" } else { " " }, k),
                if sel { p.selected() } else { Style::default().fg(p.fg) },
            ),
            Span::styled((*v).clone(), Style::default().fg(p.accent)),
        ]));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!("{}: {}", tr(l, Key::SecCpu), app.cpu.model),
        Style::default().fg(p.dim),
    )));
    lines.push(Line::from(Span::styled(
        format!("{}: {}", tr(l, Key::Language), config_hint(app)),
        Style::default().fg(p.dim),
    )));
    lines.push(Line::from(Span::styled(
        "  ↑↓ alan · ←→ degistir · Enter uygula · Esc kapat",
        Style::default().fg(p.dim),
    )));
    f.render_widget(Paragraph::new(Text::from(lines)).wrap(Wrap { trim: true }), inner);
}

fn config_hint(app: &App) -> String {
    match crate::config::config_path() {
        Some(p) => {
            let _ = app;
            p.display().to_string()
        }
        None => "-".into(),
    }
}

fn draw_help_popup(f: &mut Frame, app: &App, p: &Palette) {
    let area = centered(f.area(), pct(f.area(), 76, 34, 74), pct_h(f.area(), 86, 8, 22));
    f.render_widget(Clear, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", tr(app.lang, Key::AboutBtn)))
        .border_style(Style::default().fg(p.accent))
        .style(Style::default().bg(p.bg));
    let inner = block.inner(area);
    f.render_widget(block, area);
    let l = app.lang;
    let rows = vec![
        ("a", tr(l, Key::AddFile)),
        ("p", tr(l, Key::LabelPreset)),
        ("c", tr(l, Key::ConvertAll)),
        ("o", tr(l, Key::PickOutFolder)),
        ("r", tr(l, Key::LabelSrcAction)),
        ("w", tr(l, Key::Overwrite)),
        ("s", tr(l, Key::SettingsTitle)),
        ("Tab / Shift+Tab", tr(l, Key::SecFiles)),
        ("↑ ↓ / j k", tr(l, Key::StRunning)),
        ("Enter", tr(l, Key::Done)),
        ("?" , tr(l, Key::AboutBtn)),
        ("q", tr(l, Key::Clear)),
    ];
    let mut lines = vec![Line::from(Span::styled(
        "FF Studio TUI — Termux",
        Style::default().fg(p.accent).add_modifier(Modifier::BOLD),
    ))];
    lines.push(Line::from(""));
    for (k, v) in rows {
        lines.push(Line::from(vec![
            Span::styled(format!("  {:<16}", k), Style::default().fg(p.warn)),
            Span::styled(v.to_string(), Style::default().fg(p.fg)),
        ]));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  Esc ile kapat",
        Style::default().fg(p.dim),
    )));
    f.render_widget(Paragraph::new(Text::from(lines)).wrap(Wrap { trim: true }), inner);
}

fn draw_input_popup(f: &mut Frame, app: &App, p: &Palette) {
    let Some(inp) = &app.input else { return };
    let title = match inp.target {
        crate::app::InputTarget::CustomArgs => tr(app.lang, Key::CustomArgs),
        crate::app::InputTarget::TrimStart => tr(app.lang, Key::TrimStart),
        crate::app::InputTarget::TrimEnd => tr(app.lang, Key::TrimEnd),
        crate::app::InputTarget::MergeName => tr(app.lang, Key::MergeNameLabel),
    };
    let area = centered(f.area(), pct(f.area(), 74, 30, 70), pct_h(f.area(), 30, 4, 5));
    f.render_widget(Clear, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {title} "))
        .border_style(Style::default().fg(p.accent))
        .style(Style::default().bg(p.bg));
    let inner = block.inner(area);
    f.render_widget(block, area);
    let txt = Text::from(vec![
        Line::from(Span::styled(
            format!("> {}", inp.buf),
            Style::default().fg(p.fg).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "  Enter: kaydet · Esc: iptal",
            Style::default().fg(p.dim),
        )),
    ]);
    f.render_widget(Paragraph::new(txt).wrap(Wrap { trim: true }), inner);
}

fn draw_quit_popup(f: &mut Frame, app: &App, p: &Palette) {
    let area = centered(f.area(), pct(f.area(), 60, 26, 50), pct_h(f.area(), 30, 4, 5));
    f.render_widget(Clear, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(p.warn))
        .style(Style::default().bg(p.bg));
    let inner = block.inner(area);
    f.render_widget(block, area);
    let msg = if app.running {
        "Donusum suruyor! Cikilirsa ffmpeg surecleri durdurulur.  e / h"
    } else {
        "Cikilsin mi?  e / h"
    };
    f.render_widget(
        Paragraph::new(Span::styled(msg, Style::default().fg(p.fg))).wrap(Wrap { trim: true }),
        inner,
    );
}

// ---------------------------------------------------------------------------
// Gezici ekrani (tam ekran)
// ---------------------------------------------------------------------------

fn draw_browse(f: &mut Frame, app: &mut App, p: &Palette) {
    let Some(b) = app.browse.as_mut() else { return };
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(12), // yol + mod
            Constraint::Min(3),         // liste (esneyen)
            Constraint::Percentage(10), // ipucu
        ])
        .split(area);

    let mode_label = match b.mode {
        PickMode::Files => "Dosya sec (Space=coklu secim)",
        PickMode::Folder => "Klasor sec (recursive tarama)",
        PickMode::OutDir => "Cikti klasoru sec",
        PickMode::MoveDir => "Tasima klasoru sec",
    };
    let head = Paragraph::new(Line::from(vec![
        Span::styled(" ", Style::default()),
        Span::styled(
            b.cur.display().to_string(),
            Style::default().fg(p.accent).add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!("   [{mode_label}]"), Style::default().fg(p.dim)),
        Span::styled(
            format!("   secili: {}", b.marked.len()),
            Style::default().fg(p.warn),
        ),
    ]))
    .wrap(Wrap { trim: true })
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(head, chunks[0]);

    if let Some(e) = &b.error {
        f.render_widget(
            Paragraph::new(Span::styled(e.clone(), Style::default().fg(p.err)))
                .wrap(Wrap { trim: true })
                .block(Block::default().borders(Borders::ALL).title(" Hata ")),
            chunks[1],
        );
    } else {
        let inner_h = chunks[1].height.saturating_sub(2) as usize;
        let (start, end) = window(b.entries.len(), b.sel, inner_h);
        let items: Vec<ListItem> = b.entries[start..end]
            .iter()
            .enumerate()
            .map(|(i, e)| {
                let idx = start + i;
                let sel = idx == b.sel;
                let marked = b.marked.contains(&e.path);
                let name = e
                    .path
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                let is_media = !e.is_dir && util::is_media_ext(e.path.extension().and_then(|x| x.to_str()).unwrap_or_default());
                let style = if sel {
                    p.selected()
                } else if e.is_dir {
                    Style::default().fg(p.accent)
                } else if is_media {
                    Style::default().fg(p.fg)
                } else {
                    Style::default().fg(p.dim)
                };
                let icon = if e.is_dir { "/" } else if is_media { " " } else { "·" };
                ListItem::new(Line::from(Span::styled(
                    format!("{} [{}] {icon}{name}", if sel { ">" } else { " " }, if marked { "x" } else { " " }),
                    style,
                )))
                .style(style)
            })
            .collect();
        let block = Block::default().borders(Borders::ALL);
        f.render_widget(List::new(items).block(block), chunks[1]);
    }

    let hint = match b.mode {
        PickMode::Files => "↑↓ gezin · Enter: klasore gir / dosyayi sec · Space: isaretle · a: tumunu · d: secimi bitir · Esc: iptal",
        _ => "↑↓ gezin · Enter: klasore gir · d: bu klasoru sec · Esc: iptal",
    };
    let quick = browse::quick_starts()
        .iter()
        .enumerate()
        .map(|(i, (label, _))| format!("{}:{label}", i + 1))
        .collect::<Vec<_>>()
        .join("  ");
    f.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled(hint, Style::default().fg(p.dim))),
            Line::from(Span::styled(
                format!("hizli klasorler: {quick}"),
                Style::default().fg(p.dim),
            )),
        ])
        .wrap(Wrap { trim: true }),
        chunks[2],
    );
}

/// Verim tablosu cizimi (yardimci: test/alternatif kullanim).
pub fn render_eff_table<'a>(f: &mut Frame, app: &App, p: &Palette, area: Rect) {
    let header = Row::new(vec![
        tr(app.lang, Key::ColFile),
        tr(app.lang, Key::ColSrcKbps),
        tr(app.lang, Key::ColTarget),
        tr(app.lang, Key::ColEst),
        tr(app.lang, Key::ColDiff),
    ])
    .style(Style::default().fg(p.accent).add_modifier(Modifier::BOLD));
    let table = Table::new(
        efficiency_rows(app),
        [
            Constraint::Percentage(34),
            Constraint::Percentage(16),
            Constraint::Percentage(20),
            Constraint::Percentage(16),
            Constraint::Percentage(14),
        ],
    )
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(tr(app.lang, Key::SecEff)));
    f.render_widget(table, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// KABUL KRITERI: hangi boyutta olursa olsun hicbir panel alanin
    /// DISINA cikmaz ve paneller ust uste binmez (toplam = tam yukseklik).
    #[test]
    fn layout_hicbir_boyutta_tasmaz() {
        for (w, h) in [
            (20u16, 8u16), (24, 10), (30, 12), (40, 16), (60, 20),
            (80, 24), (100, 30), (120, 40), (160, 60), (200, 80),
            (21, 9), (20, 40), (200, 8), (119, 23),
        ] {
            let area = Rect::new(0, 0, w, h);
            let chunks = main_layout(area);
            let toplam: u16 = chunks.iter().map(|c| c.height).sum();
            assert_eq!(
                toplam, h,
                "{w}x{h}: panellerin toplami alan yuksekligine esit olmali"
            );
            let mut beklenen_y = 0;
            for (i, c) in chunks.iter().enumerate() {
                assert!(c.x >= area.x && c.y >= area.y, "{w}x{h}: panel {i} alan disinda");
                assert!(
                    c.x + c.width <= area.x + area.width,
                    "{w}x{h}: panel {i} saga tasti"
                );
                assert!(
                    c.y + c.height <= area.y + area.height,
                    "{w}x{h}: panel {i} asagi tasti"
                );
                assert_eq!(c.y, beklenen_y, "{w}x{h}: panel {i} bosluk/ust uste binme");
                beklenen_y += c.height;
            }
            // govde her zaman gorunur bir panel kalmali
            assert!(chunks[1].height >= 3, "{w}x{h}: govde icin yer kalmadi");
        }
    }

    /// Kucuk terminal esigi: bu boyutlarin altinda normal arayuz cizilmez.
    #[test]
    fn kucuk_terminal_esigi() {
        assert!(MIN_COLS >= 20 && MIN_ROWS >= 8);
        // esik civari: kabul edilen boyutlarda govde yine de pozitif
        for (w, h) in [(MIN_COLS, MIN_ROWS), (MIN_COLS, MIN_ROWS + 1), (MIN_COLS + 1, MIN_ROWS)] {
            let c = main_layout(Rect::new(0, 0, w, h));
            assert!(c[1].height >= 3, "{w}x{h}");
        }
    }

    /// Popup olculeri (pct) alani asmaz.
    #[test]
    fn popup_olculeri_tasmaz() {
        for (w, h) in [(20u16, 8u16), (40, 12), (80, 24), (120, 40)] {
            let area = Rect::new(0, 0, w, h);
            for yuzde in [30u16, 60, 72, 76] {
                let pw = pct(area, yuzde, 26, 74);
                assert!(pw <= w, "{w}x{h}: popup genisligi tasti");
            }
            for yuzde in [30u16, 70, 80, 86] {
                let ph = pct_h(area, yuzde, 4, 22);
                assert!(ph <= h, "{w}x{h}: popup yuksekligi tasti");
            }
        }
    }

    #[test]
    fn pencere_hesabi_sinirlari() {
        // liste kisaysa tamami
        assert_eq!(window(3, 0, 10), (0, 3));
        // seciliyi ortalar
        assert_eq!(window(100, 50, 10), (45, 55));
        // sonda tasmaz
        assert_eq!(window(100, 99, 10), (90, 100));
        // basta negatife dusmez
        assert_eq!(window(100, 0, 10), (0, 10));
    }

    #[test]
    fn komut_satiri_sarmasi() {
        let w = wrap("ffmpeg -i a.flac -c:a libmp3lame out.mp3", 12);
        assert!(w.len() > 1, "uzun komut sarilmali");
        assert!(w.iter().all(|s| s.len() <= 13));
    }
}
