//! Full-screen Now Playing page
//!
//! Layout (tall terminal):
//!   [Track info header — 5 rows]
//!   [Progress bar + time — 3 rows]
//!   [Visualizer — fills remaining height]
//!   [Controls row — 1 row]
//!   [Status bar — 2 rows]
//!   [Hints — 1 row]

use super::widgets::{block, draw_hints, draw_status_bar, progress_line, volume_line, trunc};
use crate::app::App;
use crate::utils::format_duration;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.size();
    let t    = app.theme.current().clone();
    let bg   = t.bg();

    // Outer: [header] [body] [status] [hints]
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(2),
            Constraint::Length(1),
        ])
        .split(area);

    // Body: [info+progress left | visualizer right]  OR stacked on narrow
    let body = outer[0];
    if body.width >= 80 {
        draw_wide(f, body, app);
    } else {
        draw_stacked(f, body, app);
    }

    draw_status_bar(f, outer[1], app);
    draw_hints(f, outer[2], &[
        ("Space","Pause"),("n/p","Skip"),("l","Loop"),("s","Shuffle"),
        ("v","Viz"),("t","Theme"),("+/-","Vol"),("1","Home"),("q","Quit"),
    ], &t);
}

// ─── Wide layout: info panel left, full-height visualizer right ───────────────

fn draw_wide(f: &mut Frame, area: Rect, app: &mut App) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(36), Constraint::Min(1)])
        .split(area);

    draw_info_panel(f, cols[0], app);
    draw_visualizer_panel(f, cols[1], app);
}

// ─── Stacked layout: info on top, visualizer below ───────────────────────────

fn draw_stacked(f: &mut Frame, area: Rect, app: &mut App) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(12), Constraint::Min(1)])
        .split(area);

    draw_info_panel(f, rows[0], app);
    draw_visualizer_panel(f, rows[1], app);
}

// ─── Track info + controls panel ─────────────────────────────────────────────

fn draw_info_panel(f: &mut Frame, area: Rect, app: &App) {
    let t   = app.theme.current();
    let blk = block("Track Info", false, t);
    let inner = blk.inner(area);
    f.render_widget(blk, area);

    let bg = t.bg();
    let w  = inner.width.saturating_sub(2) as usize;

    if let Some(track) = app.playlist_mgr.current_track() {
        let state   = if app.player.is_paused() { "|| PAUSED" } else { ">> PLAYING" };
        let title   = trunc(&track.title,  w);
        let artist  = trunc(&track.artist, w);
        let album   = trunc(&track.album,  w);
        let elapsed = format_duration(app.player.elapsed());
        let total   = format_duration(track.duration);
        let prog    = app.player.progress();
        let loop_s  = app.playlist_mgr.loop_mode.icon();
        let shuf    = if app.playlist_mgr.shuffle { "Shuffle: ON" } else { "Shuffle: OFF" };
        let muted   = if app.player.muted { "  [MUTED]" } else { "" };

        let bar_w  = w;
        let vol_w  = w.saturating_sub(10);

        let mut lines = vec![
            // State
            Line::from(Span::styled(
                format!(" {}", state),
                Style::default().fg(t.accent()).bg(bg).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(" ", Style::default().bg(bg))),
            // Title
            Line::from(Span::styled(
                format!(" {}", title),
                Style::default().fg(t.title()).bg(bg).add_modifier(Modifier::BOLD),
            )),
            // Artist
            Line::from(Span::styled(
                format!(" {}", artist),
                Style::default().fg(t.subtitle()).bg(bg),
            )),
            // Album
            Line::from(Span::styled(
                format!(" {}", album),
                Style::default().fg(t.muted()).bg(bg),
            )),
            Line::from(Span::styled(" ", Style::default().bg(bg))),
            // Progress bar
            progress_line(prog, bar_w, t),
            // Time
            Line::from(Span::styled(
                format!(" {} / {}", elapsed, total),
                Style::default().fg(t.muted()).bg(bg),
            )),
            Line::from(Span::styled(" ", Style::default().bg(bg))),
            // Volume
            Line::from(vec![
                Span::styled(
                    format!(" Vol {:3}%{}  ", app.player.volume_pct(), muted),
                    Style::default().fg(t.muted()).bg(bg),
                ),
                Span::styled(
                    "▓".repeat(((app.player.volume_pct() as usize * vol_w) / 100).min(vol_w)),
                    Style::default().fg(t.accent()).bg(bg),
                ),
                Span::styled(
                    "░".repeat(vol_w.saturating_sub((app.player.volume_pct() as usize * vol_w) / 100)),
                    Style::default().fg(t.muted()).bg(bg),
                ),
            ]),
            Line::from(Span::styled(" ", Style::default().bg(bg))),
            // Loop + shuffle
            Line::from(Span::styled(
                format!(" {}   {}", loop_s, shuf),
                Style::default().fg(t.accent()).bg(bg),
            )),
        ];

        // Year if available
        if let Some(year) = track.year {
            lines.insert(5, Line::from(Span::styled(
                format!(" {}", year),
                Style::default().fg(t.muted()).bg(bg),
            )));
        }

        let para = Paragraph::new(lines).style(Style::default().bg(bg));
        f.render_widget(para, inner);
    } else {
        let para = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(" No track loaded", Style::default().fg(t.muted()).bg(bg))),
            Line::from(""),
            Line::from(Span::styled(" Press [1] to go back and select a file",
                Style::default().fg(t.muted()).bg(bg))),
        ]).style(Style::default().bg(bg));
        f.render_widget(para, inner);
    }
}

// ─── Visualizer panel ─────────────────────────────────────────────────────────

fn draw_visualizer_panel(f: &mut Frame, area: Rect, app: &mut App) {
    let t     = app.theme.current().clone();
    let bg    = t.bg();
    let viz_title = format!("Visualizer  {}  [v] next", app.visualizer.mode.name());
    let blk   = block(&viz_title, true, &t);
    let inner = blk.inner(area);
    f.render_widget(blk, area);

    if inner.width < 4 || inner.height < 2 { return; }

    let w = inner.width  as usize;
    let h = inner.height as usize;

    let capture              = app.player.capture.clone();
    let (base_rows, overlay) = app.visualizer.render(&capture, w, h);

    let colors     = t.visualizer_colors();
    let n_cols     = colors.len().max(1);
    let peak_color = colors[0];

    let empty_row = String::new();
    let lines: Vec<Line> = base_rows
        .iter()
        .enumerate()
        .map(|(row_i, base_row)| {
            let ov_row = overlay.get(row_i).unwrap_or(&empty_row);
            let chars: Vec<Span> = base_row
                .chars()
                .zip(ov_row.chars().chain(std::iter::repeat(' ')))
                .enumerate()
                .map(|(col_i, (base_ch, ov_ch))| {
                    let (ch, color) = if ov_ch != ' ' {
                        (ov_ch, peak_color)
                    } else {
                        (base_ch, colors[col_i % n_cols])
                    };
                    let style = if ch == ' ' {
                        Style::default().bg(bg)
                    } else {
                        Style::default().fg(color).bg(bg)
                    };
                    Span::styled(ch.to_string(), style)
                })
                .collect();
            Line::from(chars)
        })
        .collect();

    let para = Paragraph::new(lines).style(Style::default().bg(bg));
    f.render_widget(para, inner);
}
