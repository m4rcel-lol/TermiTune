use anyhow::Result;
use ratatui::style::{Color, Modifier, Style};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    pub fg:           String,
    pub bg:           String,
    pub accent:       String,
    pub progress_bar: String,
    pub visualizer:   Vec<String>,
    pub highlight_fg: String,
    pub highlight_bg: String,
    pub border:       String,
    pub title:        String,
    pub subtitle:     String,
    pub muted:        String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeDef {
    pub name:        String,
    pub description: String,
    pub border_type: String, // "rounded", "plain", "double", "thick"
    pub colors:      ThemeColors,
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub def: ThemeDef,
}

impl Theme {
    pub fn fg(&self) -> Color { parse_color(&self.def.colors.fg) }
    pub fn bg(&self) -> Color { parse_color(&self.def.colors.bg) }
    pub fn accent(&self) -> Color { parse_color(&self.def.colors.accent) }
    pub fn progress(&self) -> Color { parse_color(&self.def.colors.progress_bar) }
    pub fn highlight_fg(&self) -> Color { parse_color(&self.def.colors.highlight_fg) }
    pub fn highlight_bg(&self) -> Color { parse_color(&self.def.colors.highlight_bg) }
    pub fn border(&self) -> Color { parse_color(&self.def.colors.border) }
    pub fn title(&self) -> Color { parse_color(&self.def.colors.title) }
    pub fn subtitle(&self) -> Color { parse_color(&self.def.colors.subtitle) }
    pub fn muted(&self) -> Color { parse_color(&self.def.colors.muted) }

    pub fn visualizer_colors(&self) -> Vec<Color> {
        self.def.colors.visualizer.iter().map(|c| parse_color(c)).collect()
    }

    pub fn normal(&self) -> Style {
        Style::default().fg(self.fg()).bg(self.bg())
    }

    pub fn highlighted(&self) -> Style {
        Style::default()
            .fg(self.highlight_fg())
            .bg(self.highlight_bg())
            .add_modifier(Modifier::BOLD)
    }

    pub fn title_style(&self) -> Style {
        Style::default().fg(self.title()).add_modifier(Modifier::BOLD)
    }

    pub fn subtitle_style(&self) -> Style {
        Style::default().fg(self.subtitle())
    }

    pub fn border_style(&self) -> Style {
        Style::default().fg(self.border())
    }

    pub fn accent_style(&self) -> Style {
        Style::default().fg(self.accent()).add_modifier(Modifier::BOLD)
    }

    pub fn muted_style(&self) -> Style {
        Style::default().fg(self.muted())
    }

    pub fn border_type(&self) -> ratatui::widgets::BorderType {
        match self.def.border_type.as_str() {
            "rounded" => ratatui::widgets::BorderType::Rounded,
            "double"  => ratatui::widgets::BorderType::Double,
            "thick"   => ratatui::widgets::BorderType::Thick,
            _         => ratatui::widgets::BorderType::Plain,
        }
    }
}

fn parse_color(s: &str) -> Color {
    if s.starts_with('#') && s.len() == 7 {
        let r = u8::from_str_radix(&s[1..3], 16).unwrap_or(255);
        let g = u8::from_str_radix(&s[3..5], 16).unwrap_or(255);
        let b = u8::from_str_radix(&s[5..7], 16).unwrap_or(255);
        return Color::Rgb(r, g, b);
    }
    match s {
        "Black"         => Color::Black,
        "Red"           => Color::Red,
        "Green"         => Color::Green,
        "Yellow"        => Color::Yellow,
        "Blue"          => Color::Blue,
        "Magenta"       => Color::Magenta,
        "Cyan"          => Color::Cyan,
        "White"         => Color::White,
        "Reset"         => Color::Reset,
        _               => Color::Reset,
    }
}

pub struct ThemeManager {
    pub themes:       HashMap<String, Theme>,
    pub current_name: String,
}

impl ThemeManager {
    pub fn new() -> Self {
        let mut mgr = ThemeManager {
            themes:       HashMap::new(),
            current_name: "default".to_string(),
        };
        mgr.load_builtin_themes();
        let _ = mgr.load_user_themes();
        mgr
    }

    fn load_builtin_themes(&mut self) {
        self.themes.insert("default".to_string(), Theme { def: default_theme() });
        self.themes.insert("dark".to_string(),    Theme { def: dark_theme() });
        self.themes.insert("neon".to_string(),    Theme { def: neon_theme() });
    }

    fn load_user_themes(&mut self) -> Result<()> {
        let themes_dir = theme_dir();
        if !themes_dir.exists() {
            fs::create_dir_all(&themes_dir)?;
            self.save_builtin_themes()?;
        }
        for entry in fs::read_dir(&themes_dir)?.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                if let Ok(data) = fs::read_to_string(&path) {
                    if let Ok(def) = serde_json::from_str::<ThemeDef>(&data) {
                        let name = def.name.clone();
                        self.themes.insert(name, Theme { def });
                    }
                }
            }
        }
        Ok(())
    }

    pub fn save_builtin_themes(&self) -> Result<()> {
        let dir = theme_dir();
        fs::create_dir_all(&dir)?;
        let themes = [
            ("default.json", default_theme()),
            ("dark.json",    dark_theme()),
            ("neon.json",    neon_theme()),
        ];
        for (fname, def) in &themes {
            let json = serde_json::to_string_pretty(def)?;
            fs::write(dir.join(fname), json)?;
        }
        Ok(())
    }

    pub fn reload(&mut self) {
        self.themes.clear();
        self.load_builtin_themes();
        let _ = self.load_user_themes();
    }

    pub fn current(&self) -> &Theme {
        self.themes.get(&self.current_name)
            .or_else(|| self.themes.get("default"))
            .unwrap()
    }

    pub fn set_theme(&mut self, name: &str) -> bool {
        if self.themes.contains_key(name) {
            self.current_name = name.to_string();
            true
        } else {
            false
        }
    }

    pub fn names(&self) -> Vec<String> {
        let mut v: Vec<_> = self.themes.keys().cloned().collect();
        v.sort();
        v
    }

    pub fn next_theme(&mut self) {
        let names = self.names();
        let idx   = names.iter().position(|n| n == &self.current_name).unwrap_or(0);
        let next  = (idx + 1) % names.len();
        self.current_name = names[next].clone();
    }
}

pub fn theme_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("termitune")
        .join("themes")
}

// ─── Built-in theme definitions ────────────────────────────────────────────

fn default_theme() -> ThemeDef {
    ThemeDef {
        name:        "default".to_string(),
        description: "Clean and minimal default theme".to_string(),
        border_type: "rounded".to_string(),
        colors: ThemeColors {
            fg:           "#d8dee9".to_string(),
            bg:           "#2e3440".to_string(),
            accent:       "#88c0d0".to_string(),
            progress_bar: "#5e81ac".to_string(),
            visualizer:   vec![
                "#88c0d0".to_string(),
                "#81a1c1".to_string(),
                "#5e81ac".to_string(),
                "#8fbcbb".to_string(),
            ],
            highlight_fg: "#2e3440".to_string(),
            highlight_bg: "#88c0d0".to_string(),
            border:       "#4c566a".to_string(),
            title:        "#eceff4".to_string(),
            subtitle:     "#88c0d0".to_string(),
            muted:        "#4c566a".to_string(),
        },
    }
}

fn dark_theme() -> ThemeDef {
    ThemeDef {
        name:        "dark".to_string(),
        description: "Deep dark theme with subtle accents".to_string(),
        border_type: "plain".to_string(),
        colors: ThemeColors {
            fg:           "#cdd6f4".to_string(),
            bg:           "#1e1e2e".to_string(),
            accent:       "#cba6f7".to_string(),
            progress_bar: "#89b4fa".to_string(),
            visualizer:   vec![
                "#cba6f7".to_string(),
                "#89b4fa".to_string(),
                "#74c7ec".to_string(),
                "#94e2d5".to_string(),
            ],
            highlight_fg: "#1e1e2e".to_string(),
            highlight_bg: "#cba6f7".to_string(),
            border:       "#313244".to_string(),
            title:        "#cdd6f4".to_string(),
            subtitle:     "#cba6f7".to_string(),
            muted:        "#45475a".to_string(),
        },
    }
}

fn neon_theme() -> ThemeDef {
    ThemeDef {
        name:        "neon".to_string(),
        description: "Vibrant neon cyberpunk theme".to_string(),
        border_type: "double".to_string(),
        colors: ThemeColors {
            fg:           "#e0e0e0".to_string(),
            bg:           "#0d0d0d".to_string(),
            accent:       "#00ff9f".to_string(),
            progress_bar: "#ff007f".to_string(),
            visualizer:   vec![
                "#00ff9f".to_string(),
                "#00cfff".to_string(),
                "#ff007f".to_string(),
                "#ffdd00".to_string(),
            ],
            highlight_fg: "#0d0d0d".to_string(),
            highlight_bg: "#00ff9f".to_string(),
            border:       "#1a1a2e".to_string(),
            title:        "#00ff9f".to_string(),
            subtitle:     "#00cfff".to_string(),
            muted:        "#333333".to_string(),
        },
    }
}
