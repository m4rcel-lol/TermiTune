use super::widgets::{draw_key_hints, draw_status_bar, panel_block};
use crate::{
    app::{App, FocusedPanel},
    utils::{format_duration, truncate_str},
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, ListState, Paragraph, Block, Borders},
    Frame,
};

pub fn draw(f: &mut Frame, app: &mut App) {
    let theme = app.theme.current().clone();
    let area  = f.size();

    // Layout: main area + status + hints
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),
            Constraint::Length(2),
            Constraint::Length(1),
        ])
        .split(area);

    // Main: left (browser) | right (playlist + now-playing)
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(45),
            Constraint::Percentage(55),
        ])
        .split(outer[0]);

    // Right column: now-playing (top) + playlist (bottom)
    let right_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),
            Constraint::Min(5),
        ])
        .split(cols[1]);

    draw_file_browser(f, cols[0], app);
    draw_now_playing_mini(f, right_rows[0], app);
    draw_playlist_panel(f, right_rows[1], app);
    draw_status_bar(f, outer[1], app);
    draw_key_hints(f, outer[2], &[
        ("Tab", "Switch panel"), ("Enter", "Select"), ("Space", "Play/Pause"),
        ("n/p", "Next/Prev"), ("/", "Search"), ("2", "Full player"),
        ("q", "Quit"),
    ], &theme);
}

fn draw_file_browser(f: &mut Frame, area: Rect, app: &App) {
    let theme   = app.theme.current();
    let focused = app.focus == FocusedPanel::FileBrowser;

    let block = panel_block("  File Browser", focused, theme);
    let inner = block.inner(area);
    f.render_widget(block, area);

    // Dir path header
    let header_area = Rect { y: inner.y, height: 1, ..inner };
    let list_area   = Rect { y: inner.y + 1, height: inner.height.saturating_sub(1), ..inner };

    let dir_str = truncate_str(
        app.browser_dir.to_string_lossy().as_ref(),
        inner.width as usize,
    );
    let header = Paragraph::new(format!(" 󰉋  {}", dir_str))
        .style(theme.muted_style());
    f.render_widget(header, header_area);

    let visible = list_area.height as usize;
    let scroll  = {
        let sel = app.browser_sel;
        if sel >= visible { sel - visible + 1 } else { 0 }
    };

    let items: Vec<ListItem> = app.browser_list
        .iter()
        .skip(scroll)
        .take(visible)
        .enumerate()
        .map(|(i, e)| {
            let real_idx = i + scroll;
            let icon = if e.is_dir { "󰉋 " } else if e.is_audio { "♪ " } else { "  " };
            let name = truncate_str(&e.name, list_area.width.saturating_sub(3) as usize);
            let text = format!("{}{}", icon, name);
            let style = if real_idx == app.browser_sel {
                theme.highlighted()
            } else if e.is_audio {
                Style::default().fg(theme.accent())
            } else if e.is_dir {
                Style::default().fg(theme.subtitle())
            } else {
                theme.normal()
            };
            ListItem::new(text).style(style)
        })
        .collect();

    let list = List::new(items).style(theme.normal());
    f.render_widget(list, list_area);
}

fn draw_now_playing_mini(f: &mut Frame, area: Rect, app: &App) {
    let theme   = app.theme.current();
    let focused = app.focus == FocusedPanel::NowPlaying;
    let block   = panel_block("  Now Playing", focused, theme);
    let inner   = block.inner(area);
    f.render_widget(block, area);

    if let Some(track) = app.playlist_mgr.current_track() {
        let state = if app.player.is_paused() { "⏸ Paused" } else { "▶ Playing" };
        let title  = truncate_str(&track.title,  inner.width as usize);
        let artist = truncate_str(&track.artist, inner.width as usize);
        let dur    = format_duration(app.player.elapsed());
        let total  = format_duration(track.duration);
        let progress = app.player.progress();

        // Progress bar
        let bar_w  = inner.width.saturating_sub(2) as usize;
        let filled = (progress * bar_w as f32) as usize;
        let empty  = bar_w.saturating_sub(filled);
        let bar    = format!("{}{}", "█".repeat(filled), "░".repeat(empty));
        let time   = format!("{} / {}", dur, total);

        let lines: Vec<Line> = vec![
            Line::from(vec![
                Span::styled(format!(" {}", state), theme.accent_style()),
            ]),
            Line::from(vec![
                Span::styled(format!(" ♪  {}", title), theme.title_style()),
            ]),
            Line::from(vec![
                Span::styled(format!("    {} — {}", artist, track.album), theme.muted_style()),
            ]),
            Line::from(vec![
                Span::styled(format!(" {}", bar), Style::default().fg(theme.progress())),
            ]),
            Line::from(vec![
                Span::styled(format!(" {}", time), theme.muted_style()),
            ]),
        ];

        let para = Paragraph::new(lines).style(theme.normal());
        f.render_widget(para, inner);
    } else {
        let para = Paragraph::new("  No track loaded\n  Browse files to add music")
            .style(theme.muted_style());
        f.render_widget(para, inner);
    }
}

fn draw_playlist_panel(f: &mut Frame, area: Rect, app: &App) {
    let theme   = app.theme.current();
    let focused = app.focus == FocusedPanel::Playlist;
    let pl      = app.playlist_mgr.current_playlist();
    let title   = format!("  {} ({} tracks)", pl.name, pl.tracks.len());
    let block   = panel_block(&title, focused, theme);
    let inner   = block.inner(area);
    f.render_widget(block, area);

    let filtered: Vec<(usize, _)> = app.playlist_mgr.filtered_tracks();

    if filtered.is_empty() {
        let msg = if app.playlist_mgr.search_query.is_empty() {
            "  Empty playlist\n  Press [a] in browser to add a folder"
        } else {
            "  No results"
        };
        let para = Paragraph::new(msg).style(theme.muted_style());
        f.render_widget(para, inner);
        return;
    }

    // Search bar
    let (header_h, list_top) = if !app.playlist_mgr.search_query.is_empty() || app.searching {
        let header_area = Rect { height: 1, ..inner };
        let query = if app.searching {
            format!(" 🔍 {}▋", app.search_buf)
        } else {
            format!(" 🔍 {}", app.playlist_mgr.search_query)
        };
        let search_para = Paragraph::new(query).style(theme.accent_style());
        f.render_widget(search_para, header_area);
        (1u16, inner.y + 1)
    } else {
        (0u16, inner.y)
    };

    let list_area = Rect {
        y:      list_top,
        height: inner.height.saturating_sub(header_h),
        ..inner
    };

    let visible     = list_area.height as usize;
    let current_idx = app.playlist_mgr.current_track;
    let sel         = app.playlist_sel;
    let scroll      = if sel >= visible { sel - visible + 1 } else { 0 };

    let items: Vec<ListItem> = filtered
        .iter()
        .skip(scroll)
        .take(visible)
        .map(|(real_idx, track)| {
            let playing = *real_idx == current_idx;
            let selected = *real_idx == sel;
            let icon = if playing { "▶ " } else { "  " };
            let name = truncate_str(
                &track.display_name(),
                list_area.width.saturating_sub(8) as usize,
            );
            let dur_str = track.duration_str();
            let text = format!("{}{:<width$} {}", icon, name,
                dur_str, width = list_area.width.saturating_sub(8 + dur_str.len() as u16) as usize);

            let style = if selected && focused {
                theme.highlighted()
            } else if playing {
                Style::default().fg(theme.accent()).add_modifier(Modifier::BOLD)
            } else {
                theme.normal()
            };
            ListItem::new(text).style(style)
        })
        .collect();

    let list = List::new(items).style(theme.normal());
    f.render_widget(list, list_area);
}
