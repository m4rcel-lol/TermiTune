use anyhow::Result;
use lofty::{AudioFile, TaggedFileExt, Tag, ItemKey};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub path:     PathBuf,
    pub title:    String,
    pub artist:   String,
    pub album:    String,
    pub duration: Duration,
    pub track_no: Option<u32>,
    pub year:     Option<u32>,
}

impl Track {
    pub fn from_path(path: PathBuf) -> Result<Self> {
        let filename = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();

        let (title, artist, album, duration, track_no, year) =
            Self::read_tags(&path).unwrap_or_else(|_| {
                (filename.clone(), "Unknown".to_string(), "Unknown".to_string(),
                 Duration::ZERO, None, None)
            });

        Ok(Track { path, title, artist, album, duration, track_no, year })
    }

    fn read_tags(path: &Path) -> Result<(String, String, String, Duration, Option<u32>, Option<u32>)> {
        let tagged = lofty::read_from_path(path)?;
        let props  = tagged.properties();
        let dur    = props.duration();

        let tag: Option<&dyn Tag> = tagged.primary_tag().or_else(|| tagged.first_tag());

        let title  = tag.and_then(|t| t.get_string(&ItemKey::TrackTitle))
            .map(str::to_string)
            .unwrap_or_else(|| {
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Unknown")
                    .to_string()
            });

        let artist   = tag.and_then(|t| t.get_string(&ItemKey::TrackArtist))
            .map(str::to_string)
            .unwrap_or_else(|| "Unknown Artist".to_string());

        let album    = tag.and_then(|t| t.get_string(&ItemKey::AlbumTitle))
            .map(str::to_string)
            .unwrap_or_else(|| "Unknown Album".to_string());

        let track_no = tag.and_then(|t| t.get_string(&ItemKey::TrackNumber))
            .and_then(|s| s.parse().ok());

        let year     = tag.and_then(|t| t.get_string(&ItemKey::Year))
            .and_then(|s| s.parse().ok());

        Ok((title, artist, album, dur, track_no, year))
    }

    pub fn duration_str(&self) -> String {
        let secs  = self.duration.as_secs();
        let mins  = secs / 60;
        let secs  = secs % 60;
        if mins >= 60 {
            format!("{:02}:{:02}:{:02}", mins / 60, mins % 60, secs)
        } else {
            format!("{:02}:{:02}", mins, secs)
        }
    }

    pub fn display_name(&self) -> String {
        if self.artist != "Unknown Artist" {
            format!("{} - {}", self.artist, self.title)
        } else {
            self.title.clone()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LoopMode { Off, Single, Playlist }

impl LoopMode {
    pub fn next(&self) -> Self {
        match self {
            LoopMode::Off      => LoopMode::Single,
            LoopMode::Single   => LoopMode::Playlist,
            LoopMode::Playlist => LoopMode::Off,
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            LoopMode::Off      => "⇥ OFF",
            LoopMode::Single   => "⟳ ONE",
            LoopMode::Playlist => "⟲ ALL",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
    pub name:    String,
    pub tracks:  Vec<Track>,
    pub path:    Option<PathBuf>,
}

impl Playlist {
    pub fn new(name: impl Into<String>) -> Self {
        Playlist { name: name.into(), tracks: vec![], path: None }
    }

    pub fn from_dir(dir: &Path) -> Result<Self> {
        let name = dir.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("Playlist")
            .to_string();

        let mut tracks = Vec::new();
        for entry in WalkDir::new(dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            let ext  = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if matches!(ext.to_lowercase().as_str(), "mp3" | "flac" | "ogg" | "wav" | "aac" | "m4a") {
                if let Ok(track) = Track::from_path(path.to_path_buf()) {
                    tracks.push(track);
                }
            }
        }

        tracks.sort_by(|a, b| {
            a.track_no.unwrap_or(999).cmp(&b.track_no.unwrap_or(999))
                .then(a.title.cmp(&b.title))
        });

        Ok(Playlist { name, tracks, path: None })
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Self> {
        let data = fs::read_to_string(path)?;
        let mut pl: Playlist = serde_json::from_str(&data)?;
        pl.path = Some(path.to_path_buf());
        Ok(pl)
    }

    pub fn total_duration(&self) -> Duration {
        self.tracks.iter().map(|t| t.duration).sum()
    }

    pub fn remove_track(&mut self, idx: usize) {
        if idx < self.tracks.len() {
            self.tracks.remove(idx);
        }
    }

    pub fn sort_by_name(&mut self) {
        self.tracks.sort_by(|a, b| a.title.cmp(&b.title));
    }

    pub fn sort_by_duration(&mut self) {
        self.tracks.sort_by(|a, b| a.duration.cmp(&b.duration));
    }

    pub fn sort_by_artist(&mut self) {
        self.tracks.sort_by(|a, b| a.artist.cmp(&b.artist));
    }
}

pub struct PlaylistManager {
    pub playlists:       Vec<Playlist>,
    pub active_playlist: usize,
    pub current_track:   usize,
    pub loop_mode:       LoopMode,
    pub shuffle:         bool,
    pub shuffle_order:   Vec<usize>,
    pub shuffle_pos:     usize,
    pub search_query:    String,
}

impl PlaylistManager {
    pub fn new() -> Self {
        PlaylistManager {
            playlists:       vec![Playlist::new("Queue")],
            active_playlist: 0,
            current_track:   0,
            loop_mode:       LoopMode::Off,
            shuffle:         false,
            shuffle_order:   vec![],
            shuffle_pos:     0,
            search_query:    String::new(),
        }
    }

    pub fn current_playlist(&self) -> &Playlist {
        &self.playlists[self.active_playlist]
    }

    pub fn current_playlist_mut(&mut self) -> &mut Playlist {
        &mut self.playlists[self.active_playlist]
    }

    pub fn current_track(&self) -> Option<&Track> {
        self.current_playlist().tracks.get(self.current_track)
    }

    pub fn add_playlist(&mut self, pl: Playlist) {
        self.playlists.push(pl);
    }

    pub fn next_track(&mut self) -> Option<usize> {
        let len = self.current_playlist().tracks.len();
        if len == 0 { return None; }

        match self.loop_mode {
            LoopMode::Single => Some(self.current_track),
            LoopMode::Off => {
                if self.shuffle {
                    self.next_shuffle()
                } else if self.current_track + 1 < len {
                    self.current_track += 1;
                    Some(self.current_track)
                } else {
                    None
                }
            }
            LoopMode::Playlist => {
                if self.shuffle {
                    self.next_shuffle()
                } else {
                    self.current_track = (self.current_track + 1) % len;
                    Some(self.current_track)
                }
            }
        }
    }

    pub fn prev_track(&mut self) -> Option<usize> {
        let len = self.current_playlist().tracks.len();
        if len == 0 { return None; }
        if self.current_track > 0 {
            self.current_track -= 1;
        } else {
            self.current_track = len - 1;
        }
        Some(self.current_track)
    }

    pub fn build_shuffle_order(&mut self) {
        use rand::seq::SliceRandom;
        let len = self.current_playlist().tracks.len();
        let mut order: Vec<usize> = (0..len).collect();
        let mut rng = rand::thread_rng();
        order.shuffle(&mut rng);
        // Put current track first
        if let Some(pos) = order.iter().position(|&x| x == self.current_track) {
            order.swap(0, pos);
        }
        self.shuffle_order = order;
        self.shuffle_pos   = 0;
    }

    fn next_shuffle(&mut self) -> Option<usize> {
        if self.shuffle_order.is_empty() {
            self.build_shuffle_order();
        }
        self.shuffle_pos = (self.shuffle_pos + 1) % self.shuffle_order.len();
        self.current_track = self.shuffle_order[self.shuffle_pos];
        Some(self.current_track)
    }

    pub fn toggle_shuffle(&mut self) {
        self.shuffle = !self.shuffle;
        if self.shuffle {
            self.build_shuffle_order();
        }
    }

    pub fn toggle_loop(&mut self) {
        self.loop_mode = self.loop_mode.next();
    }

    pub fn filtered_tracks(&self) -> Vec<(usize, &Track)> {
        let q = self.search_query.to_lowercase();
        self.current_playlist().tracks.iter().enumerate()
            .filter(|(_, t)| {
                q.is_empty()
                    || t.title.to_lowercase().contains(&q)
                    || t.artist.to_lowercase().contains(&q)
                    || t.album.to_lowercase().contains(&q)
            })
            .collect()
    }
}
