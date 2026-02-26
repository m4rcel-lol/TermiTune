use super::widgets::{draw_key_hints, draw_status_bar, panel_block};
use crate::{app::App, utils::{format_duration, progress_bar, truncate_str, volume_bar}};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn draw(f: &mut Frame, app: &mut App) {
    let theme = app.theme.current().clone();
    let area  = f.size();

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),
            Constraint::Length(2),
            Constraint::Length(1),
        ])
        .split(area);

    let main = layout[0];

    // Split: left = visualizer  |  right = track info
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(55),
            Constraint::Percentage(45),
        ])
        .split(main);

    draw_visualizer_panel(f, cols[0], app);
    draw_track_info_panel(f, cols[1], app);
    draw_status_bar(f, layout[1], app);
    draw_key_hints(f, layout[2], &[
        ("Space", "Play/Pause"), ("n/p", "Next/Prev"),
        ("f/b", "Seek"), ("+/-", "Vol"), ("v", "Visualizer"),
        ("t", "Theme"), ("1", "Home"), ("q", "Quit"),
    ], &theme);
}

fn draw_visualizer_panel(f: &mut Frame, area: Rect, app: &mut App) {
    let theme = app.theme.current().clone();
    let block = panel_block(
        &format!("  Visualizer — {}", app.visualizer.mode.name()),
        true,
        &theme,
    );
    let inner = block.inner(area);
    f.render_widget(block, area);

    let capture = app.player.capture.clone();
    let rows    = app.visualizer.render_ascii(&capture, inner.width as usize, inner.height as usize);
    let colors  = theme.visualizer_colors();
    let n_cols  = colors.len().max(1);

    let lines: Vec<Line> = rows
        .into_iter()
        .enumerate()
        .map(|(row_i, row_text)| {
            // Color each character based on column position
            let chars: Vec<Span> = row_text
                .chars()
                .enumerate()
                .map(|(col_i, ch)| {
                    let color = colors[col_i % n_cols];
                    Span::styled(ch.to_string(), Style::default().fg(color))
                })
                .collect();
            Line::from(chars)
        })
        .collect();

    let para = Paragraph::new(lines).style(theme.normal());
    f.render_widget(para, inner);
}

fn draw_track_info_panel(f: &mut Frame, area: Rect, app: &App) {
    let theme = app.theme.current();
    let block = panel_block("  Track Info", false, theme);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // title
            Constraint::Length(1),  // artist
            Constraint::Length(1),  // album
            Constraint::Length(1),  // year
            Constraint::Length(2),  // spacer
            Constraint::Length(1),  // progress label
            Constraint::Length(1),  // progress bar
            Constraint::Length(2),  // spacer
            Constraint::Length(1),  // volume
            Constraint::Length(1),  // vol bar
            Constraint::Length(2),  // spacer
            Constraint::Length(1),  // loop + shuffle
            Constraint::Min(0),
        ])
        .split(inner);

    if let Some(track) = app.playlist_mgr.current_track() {
        let state_icon = if app.player.is_paused() { "⏸ PAUSED" } else { "▶ PLAYING" };

        // Title (big)
        let title = truncate_str(&track.title, inner.width as usize);
        let title_para = Paragraph::new(vec![
            Line::from(Span::styled(state_icon, theme.accent_style())),
            Line::from(""),
            Line::from(Span::styled(format!("♪  {}", title), theme.title_style().add_modifier(Modifier::BOLD))),
        ]).style(theme.normal());
        f.render_widget(title_para, rows[0]);

        // Artist / album / year
        let artist_para = Paragraph::new(
            Span::styled(format!("   {}", track.artist), theme.subtitle_style())
        );
        f.render_widget(artist_para, rows[1]);

        let album_para = Paragraph::new(
            Span::styled(format!("   {}", track.album), theme.muted_style())
        );
        f.render_widget(album_para, rows[2]);

        if let Some(year) = track.year {
            let year_para = Paragraph::new(
                Span::styled(format!("   {}", year), theme.muted_style())
            );
            f.render_widget(year_para, rows[3]);
        }

        // Progress
        let elapsed = format_duration(app.player.elapsed());
        let total   = format_duration(track.duration);
        let prog    = app.player.progress();
        let bar_w   = rows[6].width.saturating_sub(2) as usize;
        let pbar    = progress_bar(prog, bar_w);

        let prog_label = Paragraph::new(
            Span::styled(format!(" {} / {}", elapsed, total), theme.muted_style())
        );
        f.render_widget(prog_label, rows[5]);

        let prog_bar = Paragraph::new(
            Span::styled(format!(" {}", pbar), Style::default().fg(theme.progress()))
        );
        f.render_widget(prog_bar, rows[6]);

        // Volume
        let vol_w   = rows[9].width.saturating_sub(2) as usize;
        let vbar    = volume_bar(app.player.volume_pct(), vol_w);
        let mute_icon = if app.player.muted { " 🔇" } else { "" };

        let vol_label = Paragraph::new(
            Span::styled(format!(" Volume: {}%{}", app.player.volume_pct(), mute_icon), theme.muted_style())
        );
        f.render_widget(vol_label, rows[8]);

        let vol_bar_para = Paragraph::new(
            Span::styled(format!(" {}", vbar), Style::default().fg(theme.accent()))
        );
        f.render_widget(vol_bar_para, rows[9]);

        // Loop + shuffle
        let loop_str  = app.playlist_mgr.loop_mode.icon();
        let shuf_str  = if app.playlist_mgr.shuffle { " ⇀ Shuffle ON" } else { "" };
        let flags_para = Paragraph::new(
            Span::styled(format!(" {} {}", loop_str, shuf_str), theme.accent_style())
        );
        f.render_widget(flags_para, rows[11]);
    } else {
        let para = Paragraph::new(
            " No track loaded\n\n Press [1] to browse files"
        ).style(theme.muted_style());
        f.render_widget(para, inner);
    }
}
