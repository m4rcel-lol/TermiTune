use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

fn default_volume() -> f32 { 0.7 }
fn default_theme() -> String { "default".to_string() }
fn default_viz() -> String { "bars".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub music_dir: Option<PathBuf>,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_volume")]
    pub volume: f32,
    #[serde(default = "default_viz")]
    pub visualizer_mode: String,
    #[serde(default)]
    pub visualizer_sensitivity: f32,
    #[serde(default)]
    pub last_playlist: Option<PathBuf>,
    #[serde(default)]
    pub last_track_index: usize,
    #[serde(default)]
    pub restore_session: bool,
    #[serde(default)]
    pub show_notifications: bool,
    #[serde(default)]
    pub keybindings: Keybindings,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            music_dir: dirs::audio_dir(),
            theme: "default".to_string(),
            volume: 0.7,
            visualizer_mode: "bars".to_string(),
            visualizer_sensitivity: 1.0,
            last_playlist: None,
            last_track_index: 0,
            restore_session: true,
            show_notifications: false,
            keybindings: Keybindings::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keybindings {
    pub play_pause: String,
    pub next: String,
    pub previous: String,
    pub loop_toggle: String,
    pub shuffle: String,
    pub visualizer: String,
    pub theme: String,
    pub search: String,
    pub quit: String,
    pub volume_up: String,
    pub volume_down: String,
    pub mute: String,
    pub seek_forward: String,
    pub seek_backward: String,
}

impl Default for Keybindings {
    fn default() -> Self {
        Self {
            play_pause:    " ".to_string(),
            next:          "n".to_string(),
            previous:      "p".to_string(),
            loop_toggle:   "l".to_string(),
            shuffle:       "s".to_string(),
            visualizer:    "v".to_string(),
            theme:         "t".to_string(),
            search:        "/".to_string(),
            quit:          "q".to_string(),
            volume_up:     "+".to_string(),
            volume_down:   "-".to_string(),
            mute:          "m".to_string(),
            seek_forward:  "f".to_string(),
            seek_backward: "b".to_string(),
        }
    }
}

impl Config {
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("termitune")
            .join("config.json")
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        if path.exists() {
            let data = fs::read_to_string(&path)?;
            let config: Config = serde_json::from_str(&data)?;
            Ok(config)
        } else {
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(&path, json)?;
        Ok(())
    }
}
