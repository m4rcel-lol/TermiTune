use super::widgets::{draw_key_hints, draw_status_bar, panel_block};
use crate::{app::App, utils::truncate_str};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::Style,
    text::Span,
    widgets::{List, ListItem, Paragraph},
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

    let block = panel_block("  File Browser", true, &theme);
    let inner = block.inner(layout[0]);
    f.render_widget(block, layout[0]);

    let header_area = ratatui::layout::Rect { height: 1, ..inner };
    let list_area   = ratatui::layout::Rect {
        y:      inner.y + 1,
        height: inner.height.saturating_sub(1),
        ..inner
    };

    let dir_str = app.browser_dir.to_string_lossy();
    let header  = Paragraph::new(Span::styled(
        format!(" 📂  {}", dir_str), theme.muted_style(),
    ));
    f.render_widget(header, header_area);

    let visible = list_area.height as usize;
    let scroll  = if app.browser_sel >= visible { app.browser_sel - visible + 1 } else { 0 };

    let items: Vec<ListItem> = app.browser_list
        .iter()
        .skip(scroll)
        .take(visible)
        .enumerate()
        .map(|(i, e)| {
            let real_idx = i + scroll;
            let icon = if e.is_dir { "󰉋 " } else if e.is_audio { "♪ " } else { "  " };
            let name = truncate_str(&e.name, list_area.width.saturating_sub(4) as usize);
            let text = format!(" {}{}", icon, name);
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

    draw_status_bar(f, layout[1], app);
    draw_key_hints(f, layout[2], &[
        ("↑↓/jk", "Navigate"), ("Enter/l", "Open"), ("Backspace/h", "Up"),
        ("a", "Add folder"), ("1", "Home"), ("q", "Quit"),
    ], &theme);
}
