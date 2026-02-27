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
    pub border_type: String,
    pub colors:      ThemeColors,
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub def: ThemeDef,
}

impl Theme {
    pub fn fg(&self)           -> Color { parse_color(&self.def.colors.fg) }
    pub fn bg(&self)           -> Color { parse_color(&self.def.colors.bg) }
    pub fn accent(&self)       -> Color { parse_color(&self.def.colors.accent) }
    pub fn progress(&self)     -> Color { parse_color(&self.def.colors.progress_bar) }
    pub fn highlight_fg(&self) -> Color { parse_color(&self.def.colors.highlight_fg) }
    pub fn highlight_bg(&self) -> Color { parse_color(&self.def.colors.highlight_bg) }
    pub fn border(&self)       -> Color { parse_color(&self.def.colors.border) }
    pub fn title(&self)        -> Color { parse_color(&self.def.colors.title) }
    pub fn subtitle(&self)     -> Color { parse_color(&self.def.colors.subtitle) }
    pub fn muted(&self)        -> Color { parse_color(&self.def.colors.muted) }

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
        Style::default().fg(self.title()).bg(self.bg()).add_modifier(Modifier::BOLD)
    }
    pub fn subtitle_style(&self) -> Style {
        Style::default().fg(self.subtitle()).bg(self.bg())
    }
    pub fn border_style(&self) -> Style {
        Style::default().fg(self.border()).bg(self.bg())
    }
    pub fn accent_style(&self) -> Style {
        Style::default().fg(self.accent()).bg(self.bg()).add_modifier(Modifier::BOLD)
    }
    pub fn muted_style(&self) -> Style {
        Style::default().fg(self.muted()).bg(self.bg())
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
    let s = s.trim();
    if s.starts_with('#') && s.len() == 7 {
        let r = u8::from_str_radix(&s[1..3], 16).unwrap_or(200);
        let g = u8::from_str_radix(&s[3..5], 16).unwrap_or(200);
        let b = u8::from_str_radix(&s[5..7], 16).unwrap_or(200);
        return Color::Rgb(r, g, b);
    }
    Color::Reset
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
        for def in [default_theme(), dark_theme(), neon_theme(), gruvbox_theme(), dracula_theme()] {
            let name = def.name.clone();
            self.themes.insert(name, Theme { def });
        }
    }

    fn load_user_themes(&mut self) -> Result<()> {
        let dir = theme_dir();
        if !dir.exists() {
            fs::create_dir_all(&dir)?;
            self.save_builtin_themes()?;
        }
        for entry in fs::read_dir(&dir)?.flatten() {
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
        for (fname, def) in [
            ("default.json",  default_theme()),
            ("dark.json",     dark_theme()),
            ("neon.json",     neon_theme()),
            ("gruvbox.json",  gruvbox_theme()),
            ("dracula.json",  dracula_theme()),
        ] {
            fs::write(dir.join(fname), serde_json::to_string_pretty(&def)?)?;
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
        self.current_name = names[(idx + 1) % names.len()].clone();
    }
}

pub fn theme_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("termitune")
        .join("themes")
}

// ─── Built-in themes ─────────────────────────────────────────────────────────
// Key fix: muted colors are now light enough to read on dark backgrounds
// Rule: muted must be at least #7a brightness on dark themes

fn default_theme() -> ThemeDef {
    ThemeDef {
        name:        "default".into(),
        description: "Nord-inspired dark theme".into(),
        border_type: "rounded".into(),
        colors: ThemeColors {
            fg:           "#d8dee9".into(),
            bg:           "#2e3440".into(),
            accent:       "#88c0d0".into(),
            progress_bar: "#5e81ac".into(),
            visualizer:   vec!["#88c0d0".into(), "#81a1c1".into(), "#5e81ac".into(), "#8fbcbb".into()],
            highlight_fg: "#2e3440".into(),
            highlight_bg: "#88c0d0".into(),
            border:       "#616e88".into(), // was #4c566a — too dark
            title:        "#eceff4".into(),
            subtitle:     "#88c0d0".into(),
            muted:        "#9099a7".into(), // was #4c566a — was invisible, now readable
        },
    }
}

fn dark_theme() -> ThemeDef {
    ThemeDef {
        name:        "dark".into(),
        description: "Catppuccin Mocha dark theme".into(),
        border_type: "plain".into(),
        colors: ThemeColors {
            fg:           "#cdd6f4".into(),
            bg:           "#1e1e2e".into(),
            accent:       "#cba6f7".into(),
            progress_bar: "#89b4fa".into(),
            visualizer:   vec!["#cba6f7".into(), "#89b4fa".into(), "#74c7ec".into(), "#94e2d5".into()],
            highlight_fg: "#1e1e2e".into(),
            highlight_bg: "#cba6f7".into(),
            border:       "#585b70".into(), // was #313244 — too dark
            title:        "#cdd6f4".into(),
            subtitle:     "#cba6f7".into(),
            muted:        "#7f849c".into(), // was #45475a — was invisible, now readable
        },
    }
}

fn neon_theme() -> ThemeDef {
    ThemeDef {
        name:        "neon".into(),
        description: "Cyberpunk neon theme".into(),
        border_type: "double".into(),
        colors: ThemeColors {
            fg:           "#e0e0e0".into(),
            bg:           "#0d0d0d".into(),
            accent:       "#00ff9f".into(),
            progress_bar: "#ff007f".into(),
            visualizer:   vec!["#00ff9f".into(), "#00cfff".into(), "#ff007f".into(), "#ffdd00".into()],
            highlight_fg: "#0d0d0d".into(),
            highlight_bg: "#00ff9f".into(),
            border:       "#2a2a3e".into(),
            title:        "#00ff9f".into(),
            subtitle:     "#00cfff".into(),
            muted:        "#888888".into(), // was #333333 — invisible on black bg
        },
    }
}

fn gruvbox_theme() -> ThemeDef {
    ThemeDef {
        name:        "gruvbox".into(),
        description: "Warm retro Gruvbox theme".into(),
        border_type: "plain".into(),
        colors: ThemeColors {
            fg:           "#ebdbb2".into(),
            bg:           "#282828".into(),
            accent:       "#fabd2f".into(),
            progress_bar: "#d65d0e".into(),
            visualizer:   vec!["#fabd2f".into(), "#fe8019".into(), "#b8bb26".into(), "#83a598".into()],
            highlight_fg: "#282828".into(),
            highlight_bg: "#fabd2f".into(),
            border:       "#7c6f64".into(),
            title:        "#fbf1c7".into(),
            subtitle:     "#fabd2f".into(),
            muted:        "#a89984".into(),
        },
    }
}

fn dracula_theme() -> ThemeDef {
    ThemeDef {
        name:        "dracula".into(),
        description: "Classic Dracula theme".into(),
        border_type: "rounded".into(),
        colors: ThemeColors {
            fg:           "#f8f8f2".into(),
            bg:           "#282a36".into(),
            accent:       "#bd93f9".into(),
            progress_bar: "#ff79c6".into(),
            visualizer:   vec!["#bd93f9".into(), "#ff79c6".into(), "#50fa7b".into(), "#8be9fd".into()],
            highlight_fg: "#282a36".into(),
            highlight_bg: "#bd93f9".into(),
            border:       "#6272a4".into(),
            title:        "#f8f8f2".into(),
            subtitle:     "#bd93f9".into(),
            muted:        "#9098b8".into(),
        },
    }
}
