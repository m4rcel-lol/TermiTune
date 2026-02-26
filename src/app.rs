use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use crate::{
    audio::AudioPlayer,
    config::Config,
    playlist::{Playlist, PlaylistManager},
    theme::ThemeManager,
    utils::format_duration,
    visualizer::{Visualizer, VisualizerMode},
};

#[derive(Debug, Clone, PartialEq)]
pub enum AppPage {
    Home,
    NowPlaying,
    FileBrowser,
    Settings,
    Credits,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FocusedPanel {
    Playlist,
    FileBrowser,
    NowPlaying,
}

pub struct App {
    pub config:       Config,
    pub theme:        ThemeManager,
    pub playlist_mgr: PlaylistManager,
    pub player:       AudioPlayer,
    pub visualizer:   Visualizer,

    pub page:         AppPage,
    pub focus:        FocusedPanel,
    pub should_quit:  bool,

    // File browser state
    pub browser_dir:  PathBuf,
    pub browser_list: Vec<BrowserEntry>,
    pub browser_sel:  usize,
    pub browser_scroll: usize,

    // Playlist view state
    pub playlist_sel:    usize,
    pub playlist_scroll: usize,

    // Search
    pub searching:       bool,
    pub search_buf:      String,

    // Settings
    pub settings_sel:    usize,

    // Status message
    pub status_msg:      Option<(String, std::time::Instant)>,

    // Mini player
    pub mini_player:     bool,

    // Tick counter
    pub tick:            u64,
}

#[derive(Debug, Clone)]
pub struct BrowserEntry {
    pub path:    PathBuf,
    pub name:    String,
    pub is_dir:  bool,
    pub is_audio: bool,
}

impl App {
    pub fn new() -> Result<Self> {
        let config = Config::load()?;

        let mut theme = ThemeManager::new();
        theme.set_theme(&config.theme);

        let viz_mode = VisualizerMode::from_str(&config.visualizer_mode);
        let vis_sens = if config.visualizer_sensitivity == 0.0 { 1.0 } else { config.visualizer_sensitivity };
        let visualizer = Visualizer::new(viz_mode, vis_sens);

        let mut player = AudioPlayer::new()?;
        player.set_volume(config.volume);

        let browser_dir = config.music_dir.clone()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from("/")));

        let mut app = App {
            config,
            theme,
            playlist_mgr: PlaylistManager::new(),
            player,
            visualizer,
            page:            AppPage::Home,
            focus:           FocusedPanel::FileBrowser,
            should_quit:     false,
            browser_dir:     browser_dir.clone(),
            browser_list:    vec![],
            browser_sel:     0,
            browser_scroll:  0,
            playlist_sel:    0,
            playlist_scroll: 0,
            searching:       false,
            search_buf:      String::new(),
            settings_sel:    0,
            status_msg:      None,
            mini_player:     false,
            tick:            0,
        };

        app.refresh_browser();
        app.restore_session();
        Ok(app)
    }

    fn restore_session(&mut self) {
        if !self.config.restore_session { return; }
        if let Some(pl_path) = self.config.last_playlist.clone() {
            if pl_path.exists() {
                if let Ok(pl) = Playlist::load(&pl_path) {
                    self.playlist_mgr.playlists[0] = pl;
                    self.playlist_mgr.current_track = self.config.last_track_index;
                }
            }
        }
    }

    pub fn save_session(&self) -> Result<()> {
        let mut config = self.config.clone();
        config.volume              = self.player.volume;
        config.visualizer_mode     = self.visualizer.mode.to_str().to_string();
        config.visualizer_sensitivity = self.visualizer.sensitivity;
        config.theme               = self.theme.current_name.clone();
        config.last_track_index    = self.playlist_mgr.current_track;
        config.save()
    }

    pub fn tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);
        self.player.tick();

        // Clear status messages after 3 seconds
        if let Some((_, t)) = &self.status_msg {
            if t.elapsed() > Duration::from_secs(3) {
                self.status_msg = None;
            }
        }

        // Auto-advance on track end
        if self.player.is_empty() && !self.player.is_paused()
            && self.player.duration > Duration::ZERO
        {
            let _ = self.advance_track();
        }
    }

    fn advance_track(&mut self) -> Result<()> {
        if let Some(idx) = self.playlist_mgr.next_track() {
            self.play_track(idx)?;
        }
        Ok(())
    }

    pub fn play_track(&mut self, idx: usize) -> Result<()> {
        let (path, duration) = {
            let pl = self.playlist_mgr.current_playlist();
            if idx >= pl.tracks.len() { return Ok(()); }
            let t = &pl.tracks[idx];
            (t.path.clone(), t.duration)
        };

        self.player.play(&path, duration)?;
        self.playlist_mgr.current_track = idx;
        self.playlist_sel               = idx;

        let name = self.playlist_mgr.current_track()
            .map(|t| t.display_name())
            .unwrap_or_default();
        self.set_status(format!("▶  {}", name));
        Ok(())
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        // Global quit
        if key.code == KeyCode::Char('q') && !self.searching {
            return Ok(true);
        }

        if self.searching {
            return self.handle_search_key(key);
        }

        match self.page {
            AppPage::Home       => self.handle_home_key(key)?,
            AppPage::NowPlaying => self.handle_player_key(key)?,
            AppPage::FileBrowser => self.handle_browser_key(key)?,
            AppPage::Settings   => self.handle_settings_key(key)?,
            AppPage::Credits    => {
                // Any key returns home
                self.page = AppPage::Home;
            }
        }

        Ok(false)
    }

    fn handle_home_key(&mut self, key: KeyEvent) -> Result<()> {
        // Tab switches focus
        if key.code == KeyCode::Tab {
            self.focus = match self.focus {
                FocusedPanel::FileBrowser => FocusedPanel::Playlist,
                FocusedPanel::Playlist    => FocusedPanel::NowPlaying,
                FocusedPanel::NowPlaying  => FocusedPanel::FileBrowser,
            };
            return Ok(());
        }

        self.handle_playback_keys(&key)?;

        match &self.focus {
            FocusedPanel::FileBrowser => self.handle_browser_key(key)?,
            FocusedPanel::Playlist    => self.handle_playlist_keys(key)?,
            FocusedPanel::NowPlaying  => {}
        }
        Ok(())
    }

    fn handle_playback_keys(&mut self, key: &KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char(' ') => { self.player.toggle_pause(); }
            KeyCode::Char('n') => {
                if let Some(idx) = self.playlist_mgr.next_track() {
                    self.play_track(idx)?;
                }
            }
            KeyCode::Char('p') => {
                if let Some(idx) = self.playlist_mgr.prev_track() {
                    self.play_track(idx)?;
                }
            }
            KeyCode::Char('l') => {
                self.playlist_mgr.toggle_loop();
                let mode = self.playlist_mgr.loop_mode.icon();
                self.set_status(format!("Loop: {}", mode));
            }
            KeyCode::Char('s') => {
                self.playlist_mgr.toggle_shuffle();
                let on = self.playlist_mgr.shuffle;
                self.set_status(format!("Shuffle: {}", if on { "ON" } else { "OFF" }));
            }
            KeyCode::Char('v') => {
                self.visualizer.mode = self.visualizer.mode.next();
                self.set_status(format!("Visualizer: {}", self.visualizer.mode.name()));
            }
            KeyCode::Char('t') => {
                self.theme.next_theme();
                self.set_status(format!("Theme: {}", self.theme.current_name));
            }
            KeyCode::Char('+') | KeyCode::Char('=') => { self.player.volume_up(); }
            KeyCode::Char('-') => { self.player.volume_down(); }
            KeyCode::Char('m') => {
                self.player.toggle_mute();
                self.set_status(format!("Mute: {}", if self.player.muted { "ON" } else { "OFF" }));
            }
            KeyCode::Char('1') => { self.page = AppPage::Home; }
            KeyCode::Char('2') => { self.page = AppPage::NowPlaying; }
            KeyCode::Char('3') => { self.page = AppPage::Settings; }
            KeyCode::Char('4') => { self.page = AppPage::Credits; }
            KeyCode::Char('/') => {
                self.searching   = true;
                self.search_buf  = String::new();
                self.playlist_mgr.search_query = String::new();
            }
            KeyCode::Esc => {
                self.page = AppPage::Home;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_player_key(&mut self, key: KeyEvent) -> Result<()> {
        self.handle_playback_keys(&key)?;
        Ok(())
    }

    fn handle_browser_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.browser_sel > 0 {
                    self.browser_sel -= 1;
                    if self.browser_sel < self.browser_scroll {
                        self.browser_scroll = self.browser_sel;
                    }
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.browser_sel + 1 < self.browser_list.len() {
                    self.browser_sel += 1;
                }
            }
            KeyCode::Enter | KeyCode::Char('l') => {
                self.browser_enter()?;
            }
            KeyCode::Backspace | KeyCode::Char('h') => {
                if let Some(parent) = self.browser_dir.parent().map(|p| p.to_path_buf()) {
                    self.browser_dir = parent;
                    self.browser_sel = 0;
                    self.refresh_browser();
                }
            }
            KeyCode::Char('a') => {
                // Add all audio files in dir to playlist
                self.add_dir_to_playlist()?;
            }
            _ => { self.handle_playback_keys(&key)?; }
        }
        Ok(())
    }

    fn browser_enter(&mut self) -> Result<()> {
        if self.browser_list.is_empty() { return Ok(()); }
        let entry = self.browser_list[self.browser_sel].clone();
        if entry.is_dir {
            self.browser_dir = entry.path;
            self.browser_sel = 0;
            self.browser_scroll = 0;
            self.refresh_browser();
        } else if entry.is_audio {
            if let Ok(track) = crate::playlist::Track::from_path(entry.path.clone()) {
                let idx = self.playlist_mgr.current_playlist().tracks.len();
                self.playlist_mgr.current_playlist_mut().tracks.push(track);
                self.play_track(idx)?;
                self.page = AppPage::NowPlaying;
            }
        }
        Ok(())
    }

    fn add_dir_to_playlist(&mut self) -> Result<()> {
        let pl = Playlist::from_dir(&self.browser_dir.clone())?;
        let count = pl.tracks.len();
        self.playlist_mgr.current_playlist_mut().tracks.extend(pl.tracks);
        self.set_status(format!("Added {} tracks from {}", count,
            self.browser_dir.file_name().and_then(|s| s.to_str()).unwrap_or("dir")));
        Ok(())
    }

    fn handle_playlist_keys(&mut self, key: KeyEvent) -> Result<()> {
        let len = self.playlist_mgr.current_playlist().tracks.len();
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.playlist_sel > 0 { self.playlist_sel -= 1; }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.playlist_sel + 1 < len { self.playlist_sel += 1; }
            }
            KeyCode::Enter => {
                let idx = self.playlist_sel;
                self.play_track(idx)?;
                self.page = AppPage::NowPlaying;
            }
            KeyCode::Char('d') | KeyCode::Delete => {
                let idx = self.playlist_sel;
                self.playlist_mgr.current_playlist_mut().remove_track(idx);
                if self.playlist_sel > 0 { self.playlist_sel -= 1; }
            }
            KeyCode::Char('c') => {
                self.playlist_mgr.current_playlist_mut().tracks.clear();
                self.playlist_sel = 0;
                self.set_status("Playlist cleared".to_string());
            }
            KeyCode::Char('w') => {
                self.save_playlist()?;
            }
            KeyCode::Char('o') | KeyCode::Char('r') => {
                // Load playlist dialog — navigate to browser
                self.focus = FocusedPanel::FileBrowser;
            }
            _ => { self.handle_playback_keys(&key)?; }
        }
        Ok(())
    }

    fn handle_search_key(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc | KeyCode::Enter => {
                self.searching = false;
                self.playlist_mgr.search_query = self.search_buf.clone();
                self.search_buf = String::new();
            }
            KeyCode::Backspace => {
                self.search_buf.pop();
                self.playlist_mgr.search_query = self.search_buf.clone();
            }
            KeyCode::Char(c) => {
                self.search_buf.push(c);
                self.playlist_mgr.search_query = self.search_buf.clone();
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_settings_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => { self.page = AppPage::Home; }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.settings_sel > 0 { self.settings_sel -= 1; }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.settings_sel = (self.settings_sel + 1).min(5);
            }
            _ => {}
        }
        Ok(())
    }

    pub fn handle_mouse(&mut self, _event: MouseEvent) {}

    pub fn handle_resize(&mut self, _w: u16, _h: u16) {}

    pub fn refresh_browser(&mut self) {
        self.browser_list = read_dir_entries(&self.browser_dir);
    }

    fn save_playlist(&mut self) -> Result<()> {
        let dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("termitune")
            .join("playlists");
        std::fs::create_dir_all(&dir)?;
        let path = dir.join(format!("{}.json",
            self.playlist_mgr.current_playlist().name));
        self.playlist_mgr.current_playlist().save(&path)?;
        self.set_status(format!("Saved playlist to {}", path.display()));
        Ok(())
    }

    fn set_status(&mut self, msg: String) {
        self.status_msg = Some((msg, std::time::Instant::now()));
    }

    pub fn status(&self) -> Option<&str> {
        self.status_msg.as_ref().map(|(m, _)| m.as_str())
    }
}

fn read_dir_entries(dir: &Path) -> Vec<BrowserEntry> {
    let mut entries: Vec<BrowserEntry> = Vec::new();

    // Add parent navigation
    if dir.parent().is_some() {
        entries.push(BrowserEntry {
            path:     dir.parent().unwrap().to_path_buf(),
            name:     "..".to_string(),
            is_dir:   true,
            is_audio: false,
        });
    }

    let Ok(read) = std::fs::read_dir(dir) else { return entries; };

    let audio_exts = ["mp3", "flac", "ogg", "wav", "aac", "m4a"];
    let mut dirs_list = Vec::new();
    let mut files_list = Vec::new();

    for entry in read.flatten() {
        let path     = entry.path();
        let name     = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') { continue; }
        let is_dir   = path.is_dir();
        let is_audio = path.extension()
            .and_then(|e| e.to_str())
            .map(|e| audio_exts.contains(&e.to_lowercase().as_str()))
            .unwrap_or(false);

        let e = BrowserEntry { path, name, is_dir, is_audio };
        if is_dir { dirs_list.push(e); } else { files_list.push(e); }
    }

    dirs_list.sort_by(|a, b| a.name.cmp(&b.name));
    files_list.sort_by(|a, b| a.name.cmp(&b.name));
    entries.extend(dirs_list);
    entries.extend(files_list);
    entries
}
