use crate::app::App;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

// Simple clean ASCII logo, works in any font
const LOGO_WIDE: &[&str] = &[
    "  :::::::::::  ::::::::::  :::::::::   ::::    ::::  ::::::::::: ::::::::::: :::    ::: ::::    ::: ::::::::::  ",
    "     :+:      :+:         :+:    :+:  +:+:+: :+:+:+     :+:         :+:     :+:    :+: :+:+:   :+: :+:         ",
    "    +:+      +:+         +:+    +:+  +:+ +:+:+ +:+     +:+         +:+     +:+    +:+ :+:+:+  +:+ +:+           ",
    "   +#+      +#++:++#    +#++:++#:   +#+  +:+  +#+     +#+         +#+     +#+    +:+ +#+ +:+ +#+ +#++:++#        ",
    "  +#+      +#+         +#+    +#+  +#+       +#+     +#+         +#+     +#+    +#+ +#+  +#+#+# +#+               ",
    " #+#      #+#         #+#    #+#  #+#       #+#     #+#         #+#     #+#    #+# #+#   #+#+# #+#               ",
    "###      ########## ###    ###  ###       ### ###########     ###      ########  ###    #### ##########          ",
];

const LOGO_COMPACT: &[&str] = &[
    " _____ _____ ____  __  __ _____ _____ _   _ _   _ _____",
    "|_   _| ____|  _ \\|  \\/  |_   _|_   _| | | | \\ | | ____|",
    "  | | |  _| | |_) | |\\/| | | |   | | | | | |  \\| |  _|",
    "  | | | |___|  _ <| |  | | | |   | | | |_| | |\\  | |___",
    "  |_| |_____|_| \\_\\_|  |_| |_|   |_|  \\___/|_| \\_|_____|",
];

fn separator(width: usize, t: &crate::theme::Theme, bg: ratatui::style::Color) -> Line<'static> {
    Line::from(Span::styled(
        "─".repeat(width.min(60)),
        Style::default().fg(t.border()).bg(bg),
    ))
}

fn label_value<'a>(
    label: &'a str,
    value: &'a str,
    t: &crate::theme::Theme,
    bg: ratatui::style::Color,
) -> Line<'a> {
    Line::from(vec![
        Span::styled(
            format!("{:>16}  ", label),
            Style::default().fg(t.muted()).bg(bg),
        ),
        Span::styled(value, Style::default().fg(t.accent()).bg(bg).add_modifier(Modifier::BOLD)),
    ])
}

fn pill(text: &str, t: &crate::theme::Theme, bg: ratatui::style::Color) -> Span<'static> {
    Span::styled(
        format!(" {} ", text),
        Style::default()
            .fg(t.highlight_fg())
            .bg(t.highlight_bg())
            .add_modifier(Modifier::BOLD),
    )
}

pub fn draw(f: &mut Frame, app: &App) {
    let t    = app.theme.current();
    let bg   = t.bg();
    let area = f.size();

    // Outer border
    let outer_block = Block::default()
        .borders(Borders::ALL)
        .border_type(t.border_type())
        .border_style(Style::default().fg(t.accent()).bg(bg))
        .style(Style::default().bg(bg));
    let inner = outer_block.inner(area);
    f.render_widget(outer_block, area);

    // Choose logo based on width
    let (logo, logo_h) = if inner.width >= 112 {
        (LOGO_WIDE.as_ref(), LOGO_WIDE.len() as u16)
    } else {
        (LOGO_COMPACT.as_ref(), LOGO_COMPACT.len() as u16)
    };

    // Layout: [top padding] [logo] [divider] [body] [bottom padding]
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(logo_h),
            Constraint::Length(2),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(inner);

    // ── Logo ──────────────────────────────────────────────────────────────────
    // Pick gradient-style coloring: each line gets a slightly different shade
    // by cycling through visualizer colors
    let viz_colors = t.visualizer_colors();
    let n = viz_colors.len().max(1);
    let logo_lines: Vec<Line> = logo
        .iter()
        .enumerate()
        .map(|(i, l)| {
            let color = viz_colors[i % n];
            Line::from(Span::styled(
                l.to_string(),
                Style::default().fg(color).bg(bg).add_modifier(Modifier::BOLD),
            ))
        })
        .collect();

    f.render_widget(
        Paragraph::new(logo_lines)
            .alignment(Alignment::Center)
            .style(Style::default().bg(bg)),
        layout[1],
    );

    // ── Divider with version ──────────────────────────────────────────────────
    let div_w    = inner.width as usize;
    let ver_text = "  v0.1.0  ";
    let side_w   = (div_w.saturating_sub(ver_text.len())) / 2;
    let divider  = Line::from(vec![
        Span::styled("─".repeat(side_w), Style::default().fg(t.border()).bg(bg)),
        Span::styled(ver_text, Style::default().fg(t.accent()).bg(bg).add_modifier(Modifier::BOLD)),
        Span::styled("─".repeat(side_w), Style::default().fg(t.border()).bg(bg)),
    ]);

    f.render_widget(
        Paragraph::new(vec![Line::from(""), divider])
            .style(Style::default().bg(bg)),
        layout[2],
    );

    // ── Body ─────────────────────────────────────────────────────────────────
    let body_w = inner.width as usize;
    let sep    = separator(body_w, t, bg);

    let mut info: Vec<Line> = vec![
        Line::from(""),
        label_value("description", "A beautiful TUI music player for Arch Linux", t, bg),
        Line::from(""),
        sep.clone(),
        Line::from(""),
        label_value("developer",  "m4rcel-lol", t, bg),
        label_value("github",     "github.com/m4rcel-lol/TermiTune", t, bg),
        label_value("license",    "MIT", t, bg),
        Line::from(""),
        sep.clone(),
        Line::from(""),
        // Tech stack as pills
        Line::from(vec![
            Span::styled("         stack  ", Style::default().fg(t.muted()).bg(bg)),
            pill("Rust", t, bg),
            Span::styled(" ", Style::default().bg(bg)),
            pill("ratatui", t, bg),
            Span::styled(" ", Style::default().bg(bg)),
            pill("rodio", t, bg),
            Span::styled(" ", Style::default().bg(bg)),
            pill("rustfft", t, bg),
            Span::styled(" ", Style::default().bg(bg)),
            pill("lofty", t, bg),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("      platform  ", Style::default().fg(t.muted()).bg(bg)),
            pill("Arch Linux", t, bg),
            Span::styled("  native — no Flatpak, no Snap", Style::default().fg(t.muted()).bg(bg)),
        ]),
        Line::from(""),
        sep.clone(),
        Line::from(""),
    ];

    // Keybinding quick-ref
    let keys = [
        ("Space", "Play / Pause"),
        ("n / p",  "Next / Prev"),
        ("l",      "Loop mode"),
        ("s",      "Shuffle"),
        ("v",      "Visualizer"),
        ("t",      "Theme"),
        ("/ ",     "Search"),
        ("q",      "Quit"),
    ];
    for (k, v) in &keys {
        info.push(Line::from(vec![
            Span::styled(format!("{:>16}  ", k), Style::default().fg(t.accent()).bg(bg).add_modifier(Modifier::BOLD)),
            Span::styled(v.to_string(), Style::default().fg(t.fg()).bg(bg)),
        ]));
    }

    info.push(Line::from(""));
    info.push(sep.clone());
    info.push(Line::from(""));
    info.push(Line::from(
        Span::styled(
            "[ Press any key to return ]",
            Style::default().fg(t.accent()).bg(bg).add_modifier(Modifier::BOLD),
        )
    ));

    f.render_widget(
        Paragraph::new(info)
            .alignment(Alignment::Center)
            .style(Style::default().bg(bg)),
        layout[3],
    );
}
