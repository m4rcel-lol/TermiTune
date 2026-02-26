use crate::{app::App, theme::Theme, utils::format_duration};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

// ─── Reusable block factory ─────────────────────────────────────────────────

pub fn block<'a>(title: &'a str, focused: bool, t: &Theme) -> Block<'a> {
    let (title_style, border_style) = if focused {
        (
            Style::default().fg(t.accent()).bg(t.bg()).add_modifier(Modifier::BOLD),
            Style::default().fg(t.accent()).bg(t.bg()),
        )
    } else {
        (
            Style::default().fg(t.title()).bg(t.bg()),
            Style::default().fg(t.border()).bg(t.bg()),
        )
    };
    Block::default()
        .title(Span::styled(format!(" {} ", title), title_style))
        .borders(Borders::ALL)
        .border_type(t.border_type())
        .border_style(border_style)
        .style(Style::default().fg(t.fg()).bg(t.bg()))
}

pub fn block_plain<'a>(t: &Theme) -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(t.border_type())
        .border_style(Style::default().fg(t.border()).bg(t.bg()))
        .style(Style::default().fg(t.fg()).bg(t.bg()))
}

// ─── Two-tone progress bar ──────────────────────────────────────────────────

pub fn progress_line(progress: f32, width: usize, t: &Theme) -> Line<'static> {
    let w      = width.max(1);
    let filled = ((progress.clamp(0.0, 1.0) * w as f32) as usize).min(w);
    let empty  = w - filled;
    Line::from(vec![
        Span::styled("█".repeat(filled), Style::default().fg(t.progress()).bg(t.bg())),
        Span::styled("░".repeat(empty),  Style::default().fg(t.muted()).bg(t.bg())),
    ])
}

pub fn volume_line(pct: u8, width: usize, t: &Theme) -> Line<'static> {
    let w      = width.max(1);
    let filled = ((pct as usize * w) / 100).min(w);
    let empty  = w - filled;
    Line::from(vec![
        Span::styled("▓".repeat(filled), Style::default().fg(t.accent()).bg(t.bg())),
        Span::styled("░".repeat(empty),  Style::default().fg(t.muted()).bg(t.bg())),
    ])
}

// ─── Bottom status bar ───────────────────────────────────────────────────────

pub fn draw_status_bar(f: &mut Frame, area: Rect, app: &App) {
    let t  = app.theme.current();
    let bg = t.bg();

    let track = app.playlist_mgr.current_track()
        .map(|tr| format!("{}  -  {}", tr.title, tr.artist))
        .unwrap_or_else(|| "No track loaded".to_string());

    let state   = if app.player.is_paused() { "||" } else { ">>" };
    let elapsed = format_duration(app.player.elapsed());
    let total   = format_duration(app.player.duration);
    let vol     = format!("vol {}%", app.player.volume_pct());
    let muted   = if app.player.muted { " [M]" } else { "" };
    let loop_s  = app.playlist_mgr.loop_mode.icon();
    let shuf    = if app.playlist_mgr.shuffle { " [S]" } else { "" };

    let left  = format!(" {} {}", state, track);
    let right = format!("  {} / {}  {}{}  {}{}  ", elapsed, total, vol, muted, loop_s, shuf);
    let bar_w = (area.width as usize)
        .saturating_sub(left.len() + right.len() + 4)
        .max(4);
    let progress = app.player.progress();
    let filled = ((progress * bar_w as f32) as usize).min(bar_w);
    let empty  = bar_w - filled;

    let left_text = if let Some(msg) = app.status() {
        format!(" ** {} ", msg)
    } else {
        left
    };

    let line = Line::from(vec![
        Span::styled(left_text,            Style::default().fg(t.fg()).bg(bg)),
        Span::styled("  [",               Style::default().fg(t.muted()).bg(bg)),
        Span::styled("█".repeat(filled),  Style::default().fg(t.progress()).bg(bg)),
        Span::styled("░".repeat(empty),   Style::default().fg(t.muted()).bg(bg)),
        Span::styled("]",                 Style::default().fg(t.muted()).bg(bg)),
        Span::styled(right,               Style::default().fg(t.muted()).bg(bg)),
    ]);

    let para = Paragraph::new(line)
        .style(Style::default().fg(t.fg()).bg(bg))
        .block(Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(t.border()).bg(bg))
            .style(Style::default().bg(bg)));
    f.render_widget(para, area);
}

// ─── Key hints bar ──────────────────────────────────────────────────────────

pub fn draw_hints(f: &mut Frame, area: Rect, hints: &[(&str, &str)], t: &Theme) {
    let bg   = t.bg();
    let mut spans: Vec<Span> = Vec::new();
    for (i, (key, action)) in hints.iter().enumerate() {
        if i > 0 { spans.push(Span::styled("  ", Style::default().bg(bg))); }
        spans.push(Span::styled(
            format!("[{}]", key),
            Style::default().fg(t.accent()).bg(bg).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            format!(" {}", action),
            Style::default().fg(t.muted()).bg(bg),
        ));
    }
    let para = Paragraph::new(Line::from(spans))
        .style(Style::default().fg(t.fg()).bg(bg));
    f.render_widget(para, area);
}

// ─── Helper: truncate to terminal columns ───────────────────────────────────

pub fn trunc(s: &str, max: usize) -> String {
    if max == 0 { return String::new(); }
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max { return s.to_string(); }
    let mut out: String = chars[..max.saturating_sub(1)].iter().collect();
    out.push('~');
    out
}
