use crate::app::App;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

const LOGO: &str = r#"
  ████████╗███████╗██████╗ ███╗   ███╗██╗████████╗██╗   ██╗███╗   ██╗███████╗
     ██╔══╝██╔════╝██╔══██╗████╗ ████║██║╚══██╔══╝██║   ██║████╗  ██║██╔════╝
     ██║   █████╗  ██████╔╝██╔████╔██║██║   ██║   ██║   ██║██╔██╗ ██║█████╗  
     ██║   ██╔══╝  ██╔══██╗██║╚██╔╝██║██║   ██║   ██║   ██║██║╚██╗██║██╔══╝  
     ██║   ███████╗██║  ██║██║ ╚═╝ ██║██║   ██║   ╚██████╔╝██║ ╚████║███████╗
     ╚═╝   ╚══════╝╚═╝  ╚═╝╚═╝     ╚═╝╚═╝   ╚═╝    ╚═════╝ ╚═╝  ╚═══╝╚══════╝
"#;

pub fn draw(f: &mut Frame, app: &App) {
    let theme = app.theme.current();
    let area  = f.size();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(theme.border_type())
        .border_style(theme.border_style())
        .style(theme.normal());
    let inner = block.inner(area);
    f.render_widget(block, area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),  // Logo
            Constraint::Min(4),     // Info
        ])
        .split(inner);

    // Logo
    let logo_lines: Vec<Line> = LOGO
        .lines()
        .map(|l| Line::from(Span::styled(l.to_string(), theme.accent_style())))
        .collect();
    let logo = Paragraph::new(logo_lines)
        .alignment(Alignment::Center)
        .style(theme.normal());
    f.render_widget(logo, layout[0]);

    // Info
    let info = vec![
        Line::from(""),
        Line::from(Span::styled("Version 0.1.0", theme.subtitle_style())),
        Line::from(""),
        Line::from(Span::styled("A beautiful TUI music player for Arch Linux", theme.muted_style())),
        Line::from(""),
        Line::from(vec![
            Span::styled("Developer:  ", theme.muted_style()),
            Span::styled("m4rcel-lol", theme.accent_style().add_modifier(Modifier::BOLD)),
            Span::styled("  (github.com/m4rcel-lol)", theme.muted_style()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Built with  ", theme.muted_style()),
            Span::styled("♥", Style::default().fg(ratatui::style::Color::Red)),
            Span::styled("  in Rust for Arch Linux", theme.muted_style()),
        ]),
        Line::from(""),
        Line::from(Span::styled("Stack: Rust • ratatui • rodio • rustfft • lofty", theme.muted_style())),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled("[ Press any key to return ]", theme.accent_style())),
    ];

    let info_para = Paragraph::new(info)
        .alignment(Alignment::Center)
        .style(theme.normal());
    f.render_widget(info_para, layout[1]);
}
