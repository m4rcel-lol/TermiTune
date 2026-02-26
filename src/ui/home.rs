//! Home page — 3-panel layout that adapts to terminal size
//!
//! Wide  (>=120 cols):  [Browser 38%] [Playlist 38%] [NowPlaying 24%]
//! Normal (>=80 cols):  [Browser 42%] [Right: NowPlaying top + Playlist bottom]
//! Narrow (<80 cols):   Single panel, Tab to switch

use super::widgets::{block, draw_hints, draw_status_bar, progress_line, trunc};
use crate::app::{App, FocusedPanel};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph},
    Frame,
};

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.size();
    let t    = app.theme.current().clone();
    let bg   = t.bg();

    // Outer layout: [top bar 1] [main] [status 2] [hints 1]
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(2),
            Constraint::Length(1),
        ])
        .split(area);

    // ── Top title bar ────────────────────────────────────────────────────────
    let page_hint = match app.focus {
        FocusedPanel::FileBrowser => "BROWSER",
        FocusedPanel::Playlist    => "PLAYLIST",
        FocusedPanel::NowPlaying  => "NOW PLAYING",
    };
    let title_line = Line::from(vec![
        Span::styled(" TermiTune ", Style::default().fg(t.accent()).bg(bg).add_modifier(Modifier::BOLD)),
        Span::styled("| ", Style::default().fg(t.border()).bg(bg)),
        Span::styled(page_hint, Style::default().fg(t.muted()).bg(bg)),
        Span::styled("  [Tab] switch focus  [2] full player  [3] settings  [4] credits",
            Style::default().fg(t.muted()).bg(bg)),
    ]);
    let title_bar = Paragraph::new(title_line)
        .style(Style::default().fg(t.fg()).bg(bg));
    f.render_widget(title_bar, outer[0]);

    // ── Main panels ──────────────────────────────────────────────────────────
    let w = outer[1].width;
    if w >= 120 {
        draw_three_col(f, outer[1], app);
    } else if w >= 72 {
        draw_two_col(f, outer[1], app);
    } else {
        draw_single(f, outer[1], app);
    }

    // ── Status + hints ───────────────────────────────────────────────────────
    draw_status_bar(f, outer[2], app);
    draw_hints(f, outer[3], &[
        ("Tab", "Focus"),("Enter", "Play"),("Space", "Pause"),
        ("n/p", "Skip"),("a", "Add folder"),("+/-", "Vol"),("/","Search"),("q","Quit"),
    ], &t);
}

// ─── Three-column layout (wide terminals) ────────────────────────────────────

fn draw_three_col(f: &mut Frame, area: Rect, app: &mut App) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(38),
            Constraint::Percentage(38),
            Constraint::Percentage(24),
        ])
        .split(area);

    draw_browser_panel(f, cols[0], app);
    draw_playlist_panel(f, cols[1], app);
    draw_now_playing_panel(f, cols[2], app);
}

// ─── Two-column layout (normal terminals) ────────────────────────────────────

fn draw_two_col(f: &mut Frame, area: Rect, app: &mut App) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(44), Constraint::Percentage(56)])
        .split(area);

    draw_browser_panel(f, cols[0], app);

    // Right: now-playing (fixed height) + playlist (rest)
    let np_h = 9u16.min(cols[1].height / 3);
    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(np_h), Constraint::Min(1)])
        .split(cols[1]);

    draw_now_playing_panel(f, right[0], app);
    draw_playlist_panel(f, right[1], app);
}

// ─── Single-panel layout (narrow terminals) ──────────────────────────────────

fn draw_single(f: &mut Frame, area: Rect, app: &mut App) {
    match app.focus {
        FocusedPanel::FileBrowser => draw_browser_panel(f, area, app),
        FocusedPanel::Playlist    => draw_playlist_panel(f, area, app),
        FocusedPanel::NowPlaying  => draw_now_playing_panel(f, area, app),
    }
}

// ─── File browser panel ───────────────────────────────────────────────────────

fn draw_browser_panel(f: &mut Frame, area: Rect, app: &App) {
    let t       = app.theme.current();
    let focused = app.focus == FocusedPanel::FileBrowser;
    let blk     = block("File Browser", focused, t);
    let inner   = blk.inner(area);
    f.render_widget(blk, area);

    if inner.height < 2 { return; }

    let bg = t.bg();

    // Dir path row
    let dir_area  = Rect { height: 1, ..inner };
    let list_area = Rect { y: inner.y + 1, height: inner.height.saturating_sub(1), ..inner };

    let dir_str = trunc(
        &app.browser_dir.to_string_lossy(),
        inner.width.saturating_sub(3) as usize,
    );
    let dir_para = Paragraph::new(Line::from(vec![
        Span::styled(" > ", Style::default().fg(t.accent()).bg(bg)),
        Span::styled(dir_str, Style::default().fg(t.muted()).bg(bg)),
    ])).style(Style::default().bg(bg));
    f.render_widget(dir_para, dir_area);

    let visible = list_area.height as usize;
    let scroll  = if app.browser_sel >= visible { app.browser_sel - visible + 1 } else { 0 };
    let max_name = list_area.width.saturating_sub(4) as usize;

    let items: Vec<ListItem> = app.browser_list
        .iter()
        .enumerate()
        .skip(scroll)
        .take(visible)
        .map(|(i, e)| {
            let selected = i == app.browser_sel;
            let icon = if e.is_dir { "> " } else if e.is_audio { "~ " } else { "  " };
            let name = trunc(&e.name, max_name);
            let text = format!(" {}{}", icon, name);

            let style = if selected && focused {
                Style::default().fg(t.highlight_fg()).bg(t.highlight_bg()).add_modifier(Modifier::BOLD)
            } else if e.is_audio {
                Style::default().fg(t.accent()).bg(bg)
            } else if e.is_dir {
                Style::default().fg(t.subtitle()).bg(bg)
            } else {
                Style::default().fg(t.muted()).bg(bg)
            };
            ListItem::new(text).style(style)
        })
        .collect();

    let list = List::new(items).style(Style::default().bg(bg));
    f.render_widget(list, list_area);
}

// ─── Playlist panel ───────────────────────────────────────────────────────────

fn draw_playlist_panel(f: &mut Frame, area: Rect, app: &App) {
    let t       = app.theme.current();
    let focused = app.focus == FocusedPanel::Playlist;
    let pl      = app.playlist_mgr.current_playlist();
    let n       = pl.tracks.len();
    let s       = if n == 1 { "track" } else { "tracks" };
    let title   = format!("Queue  ({} {})", n, s);
    let blk     = block(&title, focused, t);
    let inner   = blk.inner(area);
    f.render_widget(blk, area);

    if inner.height < 1 { return; }

    let bg = t.bg();

    // Optional search row
    let (search_rows, list_top) = if !app.playlist_mgr.search_query.is_empty() || app.searching {
        let sa = Rect { height: 1, ..inner };
        let q  = if app.searching {
            format!(" / {}|", app.search_buf)
        } else {
            format!(" / {}", app.playlist_mgr.search_query)
        };
        let sp = Paragraph::new(Line::from(vec![
            Span::styled(q, Style::default().fg(t.accent()).bg(bg)),
        ])).style(Style::default().bg(bg));
        f.render_widget(sp, sa);
        (1u16, inner.y + 1)
    } else {
        (0, inner.y)
    };

    let list_area = Rect {
        y:      list_top,
        height: inner.height.saturating_sub(search_rows),
        ..inner
    };

    if list_area.height == 0 { return; }

    let filtered = app.playlist_mgr.filtered_tracks();
    if filtered.is_empty() {
        let msg = if app.playlist_mgr.search_query.is_empty() {
            " Empty — browse files and press [Enter] or [a]"
        } else {
            " No results"
        };
        f.render_widget(
            Paragraph::new(msg).style(Style::default().fg(t.muted()).bg(bg)),
            list_area,
        );
        return;
    }

    let visible     = list_area.height as usize;
    let current_idx = app.playlist_mgr.current_track;
    let sel         = app.playlist_sel;
    let scroll      = if sel >= visible { sel - visible + 1 } else { 0 };
    let dur_w       = 6usize;
    let name_w      = list_area.width.saturating_sub(dur_w as u16 + 4) as usize;

    let items: Vec<ListItem> = filtered
        .iter()
        .skip(scroll)
        .take(visible)
        .map(|(real_idx, track)| {
            let playing  = *real_idx == current_idx;
            let selected = *real_idx == sel;
            let prefix   = if playing { ">>" } else { "  " };
            let name     = trunc(&track.display_name(), name_w);
            let dur      = track.duration_str();
            // Pad name to fill available width
            let pad      = name_w.saturating_sub(name.len());
            let text     = format!(" {} {}{}  {}", prefix, name, " ".repeat(pad), dur);

            let style = if selected && focused {
                Style::default().fg(t.highlight_fg()).bg(t.highlight_bg()).add_modifier(Modifier::BOLD)
            } else if playing {
                Style::default().fg(t.accent()).bg(bg).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(t.fg()).bg(bg)
            };
            ListItem::new(text).style(style)
        })
        .collect();

    let list = List::new(items).style(Style::default().bg(bg));
    f.render_widget(list, list_area);
}

// ─── Now-playing panel ────────────────────────────────────────────────────────

fn draw_now_playing_panel(f: &mut Frame, area: Rect, app: &App) {
    let t       = app.theme.current();
    let focused = app.focus == FocusedPanel::NowPlaying;
    let blk     = block("Now Playing", focused, t);
    let inner   = blk.inner(area);
    f.render_widget(blk, area);

    let bg = t.bg();

    if let Some(track) = app.playlist_mgr.current_track() {
        let state = if app.player.is_paused() { "|| PAUSED" } else { ">> PLAYING" };
        let w     = inner.width.saturating_sub(2) as usize;

        let title  = trunc(&track.title,  w);
        let artist = trunc(&track.artist, w);
        let album  = trunc(&track.album,  w);
        let elapsed = crate::utils::format_duration(app.player.elapsed());
        let total   = crate::utils::format_duration(track.duration);
        let prog    = app.player.progress();

        let mut lines: Vec<Line> = vec![
            Line::from(Span::styled(
                format!(" {}", state),
                Style::default().fg(t.accent()).bg(bg).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled("", Style::default().bg(bg))),
            Line::from(Span::styled(
                format!(" {}", title),
                Style::default().fg(t.title()).bg(bg).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                format!(" {}", artist),
                Style::default().fg(t.subtitle()).bg(bg),
            )),
            Line::from(Span::styled(
                format!(" {}", album),
                Style::default().fg(t.muted()).bg(bg),
            )),
            Line::from(Span::styled("", Style::default().bg(bg))),
        ];

        // Progress bar
        let bar_w = inner.width.saturating_sub(2) as usize;
        lines.push(progress_line(prog, bar_w, t));
        lines.push(Line::from(Span::styled(
            format!(" {} / {}", elapsed, total),
            Style::default().fg(t.muted()).bg(bg),
        )));

        // Volume (only if there's room)
        if inner.height as usize > lines.len() + 2 {
            lines.push(Line::from(Span::styled("", Style::default().bg(bg))));
            let vol_w = inner.width.saturating_sub(12) as usize;
            let vfill = ((app.player.volume_pct() as usize * vol_w) / 100).min(vol_w);
            let vempt = vol_w - vfill;
            lines.push(Line::from(vec![
                Span::styled(
                    format!(" Vol {:3}%  ", app.player.volume_pct()),
                    Style::default().fg(t.muted()).bg(bg),
                ),
                Span::styled("▓".repeat(vfill), Style::default().fg(t.accent()).bg(bg)),
                Span::styled("░".repeat(vempt),  Style::default().fg(t.muted()).bg(bg)),
            ]));
        }

        let para = Paragraph::new(lines).style(Style::default().bg(bg));
        f.render_widget(para, inner);
    } else {
        let para = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                " No track loaded",
                Style::default().fg(t.muted()).bg(bg),
            )),
            Line::from(""),
            Line::from(Span::styled(
                " Browse files and press [Enter]",
                Style::default().fg(t.muted()).bg(bg),
            )),
        ]).style(Style::default().bg(bg));
        f.render_widget(para, inner);
    }
}
