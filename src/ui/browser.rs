use super::widgets::{block, draw_hints, draw_status_bar, trunc};
use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph},
    Frame,
};

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.size();
    let t    = app.theme.current().clone();
    let bg   = t.bg();

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(2), Constraint::Length(1)])
        .split(area);

    let blk   = block("File Browser", true, &t);
    let inner = blk.inner(outer[0]);
    f.render_widget(blk, outer[0]);

    // Dir path
    let dir_area  = ratatui::layout::Rect { height: 1, ..inner };
    let list_area = ratatui::layout::Rect {
        y:      inner.y + 1,
        height: inner.height.saturating_sub(1),
        ..inner
    };

    let dir_str = trunc(&app.browser_dir.to_string_lossy(), inner.width.saturating_sub(4) as usize);
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" > ", Style::default().fg(t.accent()).bg(bg)),
            Span::styled(dir_str, Style::default().fg(t.muted()).bg(bg)),
        ])).style(Style::default().bg(bg)),
        dir_area,
    );

    let visible  = list_area.height as usize;
    let scroll   = if app.browser_sel >= visible { app.browser_sel - visible + 1 } else { 0 };
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
            let style = if selected {
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

    f.render_widget(List::new(items).style(Style::default().bg(bg)), list_area);

    draw_status_bar(f, outer[1], app);
    draw_hints(f, outer[2], &[
        ("j/k","Move"),("Enter","Open"),("Backspace","Up dir"),
        ("a","Add folder"),("1","Home"),("q","Quit"),
    ], &t);
}
