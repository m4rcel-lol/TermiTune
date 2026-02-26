use super::widgets::{draw_key_hints, panel_block};
use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn draw(f: &mut Frame, app: &App) {
    let theme = app.theme.current().clone();
    let area  = f.size();

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),
            Constraint::Length(1),
        ])
        .split(area);

    let block = panel_block("  Settings", true, &theme);
    let inner = block.inner(layout[0]);
    f.render_widget(block, layout[0]);

    let themes_list = app.theme.names().join(", ");
    let settings = vec![
        ("Theme",         format!("{} (available: {})", app.theme.current_name, themes_list)),
        ("Volume",        format!("{}%", app.player.volume_pct())),
        ("Visualizer",    app.visualizer.mode.name().to_string()),
        ("Sensitivity",   format!("{:.1}", app.visualizer.sensitivity)),
        ("Loop Mode",     app.playlist_mgr.loop_mode.icon().to_string()),
        ("Shuffle",       if app.playlist_mgr.shuffle { "ON".to_string() } else { "OFF".to_string() }),
        ("Config file",   crate::config::Config::config_path().display().to_string()),
        ("Themes dir",    crate::theme::theme_dir().display().to_string()),
    ];

    let lines: Vec<Line> = settings
        .iter()
        .enumerate()
        .flat_map(|(i, (key, val))| {
            let selected = i == app.settings_sel;
            let key_style = if selected {
                theme.highlighted()
            } else {
                theme.subtitle_style()
            };
            let val_style = if selected {
                theme.accent_style().add_modifier(Modifier::BOLD)
            } else {
                theme.normal()
            };
            vec![
                Line::from(vec![
                    Span::styled(format!("  {:20} ", key), key_style),
                    Span::styled(val.clone(), val_style),
                ]),
            ]
        })
        .collect();

    let para = Paragraph::new(lines).style(theme.normal());
    f.render_widget(para, inner);

    draw_key_hints(f, layout[1], &[
        ("↑↓/jk", "Navigate"), ("t", "Change theme"), ("v", "Change viz"),
        ("+/-", "Volume"), ("Esc", "Back"), ("q", "Quit"),
    ], &theme);
}
