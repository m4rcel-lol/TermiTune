use crate::{app::App, theme::Theme, utils::{format_duration, progress_bar, volume_bar}};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

/// Renders the playback status bar at the bottom of any page
pub fn draw_status_bar(f: &mut Frame, area: Rect, app: &App) {
    let theme = app.theme.current();

    let track_info = app.playlist_mgr.current_track()
        .map(|t| format!("♪  {} — {}", t.title, t.artist))
        .unwrap_or_else(|| "No track loaded".to_string());

    let elapsed  = format_duration(app.player.elapsed());
    let total    = app.player.duration;
    let dur_str  = format!("{} / {}", elapsed, format_duration(total));
    let progress = app.player.progress();
    let vol_str  = format!("vol: {}%", app.player.volume_pct());
    let muted    = if app.player.muted { " 🔇" } else { "" };
    let loop_str = app.playlist_mgr.loop_mode.icon();
    let shuf_str = if app.playlist_mgr.shuffle { " ⇀" } else { "" };

    let state = if app.player.is_paused() { "⏸" } else { "▶" };

    let left = format!(" {} {}{}", state, track_info, muted);
    let right = format!("{} {} {}{}  ", dur_str, vol_str, loop_str, shuf_str);

    let bar_width = area.width.saturating_sub(left.len() as u16 + right.len() as u16 + 4) as usize;
    let pbar      = progress_bar(progress, bar_width.max(4));

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(20),
            Constraint::Length(bar_width as u16 + 4),
            Constraint::Length(right.len() as u16 + 2),
        ])
        .split(area);

    let left_para = Paragraph::new(left)
        .style(theme.normal())
        .block(Block::default()
            .borders(Borders::TOP)
            .border_style(theme.border_style())
            .border_type(theme.border_type()));
    f.render_widget(left_para, chunks[0]);

    let prog_para = Paragraph::new(format!(" [{}] ", pbar))
        .style(Style::default().fg(theme.progress()));
    f.render_widget(prog_para, chunks[1]);

    let right_para = Paragraph::new(right)
        .style(theme.muted_style());
    f.render_widget(right_para, chunks[2]);

    // Status message overlay
    if let Some(msg) = app.status() {
        let msg_area = Rect {
            x:      area.x + 2,
            y:      area.y,
            width:  (msg.len() as u16 + 4).min(area.width),
            height: 1,
        };
        let msg_para = Paragraph::new(format!(" {} ", msg))
            .style(theme.accent_style());
        f.render_widget(msg_para, msg_area);
    }
}

/// Key hint bar at the very bottom
pub fn draw_key_hints(f: &mut Frame, area: Rect, hints: &[(&str, &str)], theme: &Theme) {
    let mut spans: Vec<Span> = Vec::new();
    for (i, (key, action)) in hints.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw("  "));
        }
        spans.push(Span::styled(format!("[{}]", key), theme.accent_style()));
        spans.push(Span::styled(format!(" {}", action), theme.muted_style()));
    }
    let para = Paragraph::new(Line::from(spans))
        .style(theme.normal());
    f.render_widget(para, area);
}

/// Draws a title block for panels
pub fn panel_block<'a>(title: &'a str, focused: bool, theme: &Theme) -> Block<'a> {
    let border_style = if focused {
        theme.accent_style()
    } else {
        theme.border_style()
    };
    Block::default()
        .title(Span::styled(format!(" {} ", title), if focused {
            theme.accent_style()
        } else {
            theme.title_style()
        }))
        .borders(Borders::ALL)
        .border_type(theme.border_type())
        .border_style(border_style)
        .style(theme.normal())
}
