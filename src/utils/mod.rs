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
    if s.len() <= max_width { return s.to_string(); }
    if max_width <= 1 { return "~".to_string(); }
    format!("{}~", &s[..max_width - 1])
}

pub fn progress_bar_parts(progress: f32, width: usize) -> (String, String) {
    if width == 0 { return (String::new(), String::new()); }
    let filled = (progress.clamp(0.0, 1.0) * width as f32) as usize;
    let empty  = width.saturating_sub(filled);
    ("█".repeat(filled), "░".repeat(empty))
}

pub fn progress_bar(progress: f32, width: usize) -> String {
    let (f, e) = progress_bar_parts(progress, width);
    format!("{}{}", f, e)
}

pub fn volume_bar_parts(pct: u8, width: usize) -> (String, String) {
    let filled = (pct as usize * width) / 100;
    let empty  = width.saturating_sub(filled);
    ("▓".repeat(filled), "░".repeat(empty))
}

pub fn volume_bar(pct: u8, width: usize) -> String {
    let (f, e) = volume_bar_parts(pct, width);
    format!("{}{}", f, e)
}

pub fn pluralize(n: usize, word: &str) -> String {
    if n == 1 { format!("{} {}", n, word) } else { format!("{} {}s", n, word) }
}
