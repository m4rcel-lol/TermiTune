use super::widgets::{block, draw_hints};
use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn draw(f: &mut Frame, app: &App) {
    let area = f.size();
    let t    = app.theme.current().clone();
    let bg   = t.bg();

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(area);

    let blk   = block("Settings", true, &t);
    let inner = blk.inner(outer[0]);
    f.render_widget(blk, outer[0]);

    let themes = app.theme.names().join(", ");
    let settings: &[(&str, String)] = &[
        ("Theme",            format!("{} (available: {})", app.theme.current_name, themes)),
        ("Volume",           format!("{}%", app.player.volume_pct())),
        ("Visualizer mode",  app.visualizer.mode.name().to_string()),
        ("Sensitivity",      format!("{:.1}", app.visualizer.sensitivity)),
        ("Loop mode",        app.playlist_mgr.loop_mode.icon().to_string()),
        ("Shuffle",          if app.playlist_mgr.shuffle { "ON".into() } else { "OFF".into() }),
        ("Restore session",  if app.config.restore_session { "ON".into() } else { "OFF".into() }),
        ("Config path",      crate::config::Config::config_path().display().to_string()),
        ("Themes path",      crate::theme::theme_dir().display().to_string()),
    ];

    let mut lines: Vec<Line> = vec![Line::from("")];
    for (i, (key, val)) in settings.iter().enumerate() {
        let selected = i == app.settings_sel;
        let (key_style, val_style) = if selected {
            (
                Style::default().fg(t.highlight_fg()).bg(t.highlight_bg()).add_modifier(Modifier::BOLD),
                Style::default().fg(t.accent()).bg(t.highlight_bg()).add_modifier(Modifier::BOLD),
            )
        } else {
            (
                Style::default().fg(t.subtitle()).bg(bg),
                Style::default().fg(t.fg()).bg(bg),
            )
        };
        lines.push(Line::from(vec![
            Span::styled(format!("  {:22}", key), key_style),
            Span::styled(format!("  {}", val), val_style),
        ]));
        lines.push(Line::from(""));
    }

    f.render_widget(Paragraph::new(lines).style(Style::default().bg(bg)), inner);

    draw_hints(f, outer[1], &[
        ("j/k","Navigate"),("t","Theme"),("v","Viz"),
        ("+/-","Volume"),("Esc","Back"),("q","Quit"),
    ], &t);
}
