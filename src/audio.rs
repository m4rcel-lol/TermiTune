use anyhow::Result;
use parking_lot::Mutex;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::{
    collections::VecDeque,
    fs::File,
    io::BufReader,
    path::Path,
    sync::Arc,
    time::Duration,
};

/// Shared ring buffer for audio samples (used by visualizer)
pub type AudioBuffer = Arc<Mutex<VecDeque<f32>>>;

const CAPTURE_BUF_SIZE: usize = 4096;

pub struct AudioPlayer {
    _stream:       OutputStream,
    stream_handle: OutputStreamHandle,
    sink:          Sink,
    pub volume:    f32,
    pub muted:     bool,
    /// Approximate elapsed playback time in the current track
    elapsed:       Duration,
    /// Total duration of current track
    pub duration:  Duration,
    pub capture:   AudioBuffer,
    last_tick:     std::time::Instant,
}

impl AudioPlayer {
    pub fn new() -> Result<Self> {
        let (stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;
        let capture: AudioBuffer = Arc::new(Mutex::new(VecDeque::with_capacity(CAPTURE_BUF_SIZE)));

        Ok(AudioPlayer {
            _stream: stream,
            stream_handle,
            sink,
            volume: 0.7,
            muted:  false,
            elapsed: Duration::ZERO,
            duration: Duration::ZERO,
            capture,
            last_tick: std::time::Instant::now(),
        })
    }

    pub fn play(&mut self, path: &Path, duration: Duration) -> Result<()> {
        self.sink.stop();
        // Small delay to ensure clean transition
        self.sink = Sink::try_new(&self.stream_handle)?;
        self.sink.set_volume(if self.muted { 0.0 } else { self.volume });

        let file   = File::open(path)?;
        let reader = BufReader::new(file);
        let source = Decoder::new(reader)?.convert_samples::<f32>();

        let capture     = Arc::clone(&self.capture);
        let cap_source  = CaptureSource::new(source, capture);

        self.sink.append(cap_source);
        self.sink.play();
        self.elapsed  = Duration::ZERO;
        self.duration = duration;
        self.last_tick = std::time::Instant::now();

        Ok(())
    }

    pub fn toggle_pause(&mut self) {
        if self.sink.is_paused() {
            self.sink.play();
        } else {
            self.sink.pause();
        }
    }

    pub fn is_paused(&self) -> bool { self.sink.is_paused() }

    pub fn is_empty(&self) -> bool { self.sink.empty() }

    pub fn stop(&mut self) {
        self.sink.stop();
        self.elapsed = Duration::ZERO;
    }

    pub fn set_volume(&mut self, v: f32) {
        self.volume = v.clamp(0.0, 1.5);
        if !self.muted {
            self.sink.set_volume(self.volume);
        }
    }

    pub fn volume_up(&mut self) {
        self.set_volume(self.volume + 0.05);
    }

    pub fn volume_down(&mut self) {
        self.set_volume((self.volume - 0.05).max(0.0));
    }

    pub fn toggle_mute(&mut self) {
        self.muted = !self.muted;
        self.sink.set_volume(if self.muted { 0.0 } else { self.volume });
    }

    /// Returns elapsed time as a f32 fraction 0.0 – 1.0
    pub fn progress(&self) -> f32 {
        if self.duration.is_zero() { return 0.0; }
        (self.elapsed.as_secs_f32() / self.duration.as_secs_f32()).min(1.0)
    }

    pub fn elapsed(&self) -> Duration { self.elapsed }

    /// Called every frame; advances internal clock
    pub fn tick(&mut self) {
        if !self.sink.is_paused() && !self.sink.empty() {
            let now     = std::time::Instant::now();
            let delta   = now.duration_since(self.last_tick);
            self.elapsed = (self.elapsed + delta).min(self.duration);
            self.last_tick = now;
        } else {
            self.last_tick = std::time::Instant::now();
        }
    }

    pub fn volume_pct(&self) -> u8 { ((self.volume / 1.5) * 100.0) as u8 }
}

// ─── CaptureSource ─────────────────────────────────────────────────────────

use rodio::Source;

struct CaptureSource<S: Source<Item = f32>> {
    inner:   S,
    capture: AudioBuffer,
}

impl<S: Source<Item = f32>> CaptureSource<S> {
    fn new(inner: S, capture: AudioBuffer) -> Self {
        CaptureSource { inner, capture }
    }
}

impl<S: Source<Item = f32>> Iterator for CaptureSource<S> {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let sample = self.inner.next()?;
        let mut buf = self.capture.lock();
        if buf.len() >= CAPTURE_BUF_SIZE {
            buf.pop_front();
        }
        buf.push_back(sample);
        Some(sample)
    }
}

impl<S: Source<Item = f32>> Source for CaptureSource<S> {
    fn current_frame_len(&self)        -> Option<usize> { self.inner.current_frame_len() }
    fn channels(&self)                 -> u16           { self.inner.channels() }
    fn sample_rate(&self)              -> u32           { self.inner.sample_rate() }
    fn total_duration(&self)           -> Option<Duration> { self.inner.total_duration() }
}
