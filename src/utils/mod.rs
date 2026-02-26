use std::time::Duration;

pub fn format_duration(d: Duration) -> String {
    let secs = d.as_secs();
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    if h > 0 {
        format!("{:02}:{:02}:{:02}", h, m, s)
    } else {
        format!("{:02}:{:02}", m, s)
    }
}

pub fn truncate_str(s: &str, max_width: usize) -> String {
    if s.len() <= max_width {
        return s.to_string();
    }
    if max_width <= 3 {
        return "...".chars().take(max_width).collect();
    }
    format!("{}…", &s[..max_width - 1])
}

pub fn center_str(s: &str, width: usize) -> String {
    let len = s.len();
    if len >= width { return s.to_string(); }
    let pad = (width - len) / 2;
    format!("{:>width$}", s, width = len + pad)
}

/// Render a progress bar into `width` characters
pub fn progress_bar(progress: f32, width: usize) -> String {
    if width == 0 { return String::new(); }
    let filled = (progress.clamp(0.0, 1.0) * width as f32) as usize;
    let empty  = width - filled;
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

/// Render volume bar
pub fn volume_bar(pct: u8, width: usize) -> String {
    let filled = (pct as usize * width) / 100;
    let empty  = width.saturating_sub(filled);
    format!("{}{}", "▓".repeat(filled), "░".repeat(empty))
}
