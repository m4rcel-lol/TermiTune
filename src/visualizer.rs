use crate::audio::AudioBuffer;
use num_complex::Complex;
use rustfft::{FftPlanner, FftDirection};

#[derive(Debug, Clone, PartialEq)]
pub enum VisualizerMode {
    Bars,       // Classic FFT bars from bottom
    Mirror,     // Bars mirrored top+bottom from center
    Waveform,   // Oscilloscope waveform
    Spectrum,   // FFT spectrum with peak dots
    Dots,       // Falling dots / rain effect
    Blocks,     // Full-block stepped bars (chunkier look)
}

impl VisualizerMode {
    pub fn next(&self) -> Self {
        match self {
            VisualizerMode::Bars     => VisualizerMode::Mirror,
            VisualizerMode::Mirror   => VisualizerMode::Waveform,
            VisualizerMode::Waveform => VisualizerMode::Spectrum,
            VisualizerMode::Spectrum => VisualizerMode::Dots,
            VisualizerMode::Dots     => VisualizerMode::Blocks,
            VisualizerMode::Blocks   => VisualizerMode::Bars,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            VisualizerMode::Bars     => "Bars",
            VisualizerMode::Mirror   => "Mirror",
            VisualizerMode::Waveform => "Waveform",
            VisualizerMode::Spectrum => "Spectrum",
            VisualizerMode::Dots     => "Dots",
            VisualizerMode::Blocks   => "Blocks",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "mirror"   => VisualizerMode::Mirror,
            "waveform" => VisualizerMode::Waveform,
            "spectrum" => VisualizerMode::Spectrum,
            "dots"     => VisualizerMode::Dots,
            "blocks"   => VisualizerMode::Blocks,
            _          => VisualizerMode::Bars,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            VisualizerMode::Bars     => "bars",
            VisualizerMode::Mirror   => "mirror",
            VisualizerMode::Waveform => "waveform",
            VisualizerMode::Spectrum => "spectrum",
            VisualizerMode::Dots     => "dots",
            VisualizerMode::Blocks   => "blocks",
        }
    }
}

pub struct Visualizer {
    pub mode:        VisualizerMode,
    pub sensitivity: f32,
    fft_size:        usize,
    smoothed:        Vec<f32>,
    peaks:           Vec<f32>,   // for Spectrum peak dots
    peak_hold:       Vec<u8>,    // frames to hold peak
    dots_pos:        Vec<f32>,   // for Dots falling positions
    planner:         FftPlanner<f32>,
}

impl Visualizer {
    pub fn new(mode: VisualizerMode, sensitivity: f32) -> Self {
        let fft_size = 1024;
        Visualizer {
            mode,
            sensitivity,
            fft_size,
            smoothed:   vec![0.0; fft_size / 2],
            peaks:      vec![0.0; fft_size / 2],
            peak_hold:  vec![0;   fft_size / 2],
            dots_pos:   vec![0.0; 256],
            planner:    FftPlanner::new(),
        }
    }

    // ─── Public render entry point ────────────────────────────────────────────

    /// Returns (content_rows, overlay_rows) — overlay is drawn on top with
    /// different color (used for peak dots). Both are width×height grids.
    pub fn render(&mut self, buffer: &AudioBuffer, width: usize, height: usize)
        -> (Vec<String>, Vec<String>)
    {
        if width == 0 || height == 0 {
            return (vec![], vec![]);
        }

        match self.mode {
            VisualizerMode::Bars     => (self.render_bars(buffer, width, height, false), vec![]),
            VisualizerMode::Mirror   => (self.render_mirror(buffer, width, height), vec![]),
            VisualizerMode::Waveform => (self.render_waveform(buffer, width, height), vec![]),
            VisualizerMode::Spectrum => {
                let base  = self.render_bars(buffer, width, height, false);
                let peaks = self.render_peak_overlay(width, height);
                (base, peaks)
            }
            VisualizerMode::Dots     => (self.render_dots(buffer, width, height), vec![]),
            VisualizerMode::Blocks   => (self.render_bars(buffer, width, height, true), vec![]),
        }
    }

    // Keep old API for compatibility
    pub fn render_ascii(&mut self, buffer: &AudioBuffer, width: usize, height: usize) -> Vec<String> {
        self.render(buffer, width, height).0
    }

    // ─── FFT core ─────────────────────────────────────────────────────────────

    fn fft_magnitudes(&mut self, buffer: &AudioBuffer) -> Vec<f32> {
        let raw: Vec<f32> = {
            let buf = buffer.lock();
            buf.iter().copied().collect()
        };

        let n = self.fft_size.min(raw.len());
        if n < 16 { return vec![0.0; self.fft_size / 2]; }

        let mut input: Vec<Complex<f32>> = raw[raw.len() - n..]
            .iter()
            .enumerate()
            .map(|(i, &s)| {
                // Hann window
                let w = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / n as f32).cos());
                Complex::new(s * w, 0.0)
            })
            .collect();

        input.resize(self.fft_size, Complex::new(0.0, 0.0));

        let fft = self.planner.plan_fft(self.fft_size, FftDirection::Forward);
        fft.process(&mut input);

        let half = self.fft_size / 2;
        let mags: Vec<f32> = input[..half]
            .iter()
            .map(|c| (c.re * c.re + c.im * c.im).sqrt() / half as f32)
            .collect();

        let sens = self.sensitivity;
        for (i, (sm, &m)) in self.smoothed.iter_mut().zip(mags.iter()).enumerate() {
            let boosted = m * sens * 10.0;
            // Attack fast, decay slow
            *sm = if boosted > *sm {
                (*sm * 0.4 + boosted * 0.6).min(1.0)
            } else {
                (*sm * 0.82 + boosted * 0.18).min(1.0)
            };

            // Update peaks for Spectrum mode
            if *sm > self.peaks[i] {
                self.peaks[i]     = *sm;
                self.peak_hold[i] = 20; // hold 20 frames
            } else if self.peak_hold[i] > 0 {
                self.peak_hold[i] -= 1;
            } else {
                self.peaks[i] = (self.peaks[i] - 0.02).max(0.0);
            }
        }

        self.smoothed.clone()
    }

    fn map_to_bars(&self, magnitudes: &[f32], num_bars: usize) -> Vec<f32> {
        if num_bars == 0 || magnitudes.is_empty() { return vec![]; }
        let half = magnitudes.len();
        let mut bars = vec![0.0f32; num_bars];
        for i in 0..num_bars {
            // Quadratic log-frequency mapping
            let t0 = (i as f32 / num_bars as f32).powi(2);
            let t1 = ((i + 1) as f32 / num_bars as f32).powi(2);
            let s  = (t0 * half as f32) as usize;
            let e  = ((t1 * half as f32) as usize + 1).min(half);
            let sl = &magnitudes[s..e];
            if !sl.is_empty() {
                bars[i] = sl.iter().cloned().fold(0.0f32, f32::max);
            }
        }
        bars
    }

    // ─── Bars ─────────────────────────────────────────────────────────────────

    fn render_bars(&mut self, buffer: &AudioBuffer, width: usize, height: usize, blocks: bool)
        -> Vec<String>
    {
        let mags = self.fft_magnitudes(buffer);
        let bars = self.map_to_bars(&mags, width);
        let mut rows = vec![String::with_capacity(width); height];

        // Sub-character height using block chars (8 steps per row)
        let sub_chars: &[char] = if blocks {
            &[' ', '▄', '█'] // chunky 2-step
        } else {
            &[' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█']
        };
        let steps = sub_chars.len() - 1;

        for row in 0..height {
            let row_from_top = height - 1 - row;
            for &v in &bars {
                let total_steps  = (v * (height * steps) as f32) as usize;
                let full_rows    = total_steps / steps;
                let partial      = total_steps % steps;
                let ch = if row_from_top < full_rows {
                    sub_chars[steps]
                } else if row_from_top == full_rows {
                    sub_chars[partial]
                } else {
                    ' '
                };
                rows[row].push(ch);
            }
        }
        rows
    }

    // ─── Mirror ───────────────────────────────────────────────────────────────
    // Bars grow from center upward AND downward simultaneously

    fn render_mirror(&mut self, buffer: &AudioBuffer, width: usize, height: usize) -> Vec<String> {
        let mags = self.fft_magnitudes(buffer);
        let bars = self.map_to_bars(&mags, width);
        let mut rows = vec![String::with_capacity(width); height];
        let half_h = height / 2;
        let sub_chars = &[' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
        let steps = 8usize;

        for row in 0..height {
            // Distance from center (0 = center row)
            let dist = if row <= half_h { half_h - row } else { row - half_h };
            for &v in &bars {
                let total_steps = (v * (half_h * steps) as f32) as usize;
                let full_rows   = total_steps / steps;
                let partial     = total_steps % steps;
                let ch = if dist < full_rows {
                    '█'
                } else if dist == full_rows {
                    if row < half_h { sub_chars[partial] }
                    else            { sub_chars[steps - partial.min(steps)] }
                } else if dist == 0 {
                    '─' // center line
                } else {
                    ' '
                };
                rows[row].push(ch);
            }
        }
        rows
    }

    // ─── Waveform ─────────────────────────────────────────────────────────────
    // Smooth oscilloscope — uses braille-style sub-row positioning

    fn render_waveform(&self, buffer: &AudioBuffer, width: usize, height: usize) -> Vec<String> {
        let raw: Vec<f32> = {
            let buf = buffer.lock();
            buf.iter().copied().collect()
        };

        let mut rows = vec![String::with_capacity(width); height];
        if raw.is_empty() {
            for row in &mut rows {
                for _ in 0..width {
                    row.push(if true { ' ' } else { '·' });
                }
            }
            return rows;
        }

        // Sample the waveform at `width` points
        let step = (raw.len() as f32 / width as f32).max(1.0);
        let samples: Vec<f32> = (0..width)
            .map(|i| {
                let idx = (i as f32 * step) as usize;
                // Average a small window for smoothing
                let window = 4usize;
                let start  = idx.min(raw.len().saturating_sub(window));
                let end    = (start + window).min(raw.len());
                let avg: f32 = raw[start..end].iter().sum::<f32>() / (end - start) as f32;
                (avg * self.sensitivity * 1.5).clamp(-1.0, 1.0)
            })
            .collect();

        // Build a 2D grid: for each column, which row does the waveform cross?
        let center = height as f32 / 2.0;
        let positions: Vec<f32> = samples.iter()
            .map(|&s| center - s * center * 0.9)  // map -1..1 → height..0
            .collect();

        // Draw thick waveform — mark 2 rows per sample for thickness
        let mut grid = vec![vec![' '; width]; height];
        for (col, &pos) in positions.iter().enumerate() {
            let row = pos.round() as i32;
            for delta in -1i32..=1 {
                let r = row + delta;
                if r >= 0 && (r as usize) < height {
                    grid[r as usize][col] = if delta == 0 { '█' } else { '▄' };
                }
            }
            // Connect to next sample with interpolation
            if col + 1 < width {
                let next_row = positions[col + 1].round() as i32;
                if (next_row - row).abs() > 1 {
                    let lo = row.min(next_row);
                    let hi = row.max(next_row);
                    for r in lo..=hi {
                        if r >= 0 && (r as usize) < height {
                            if grid[r as usize][col] == ' ' {
                                grid[r as usize][col] = '│';
                            }
                        }
                    }
                }
            }
        }

        for (r, row_data) in grid.into_iter().enumerate() {
            rows[r] = row_data.into_iter().collect();
        }
        rows
    }

    // ─── Spectrum peak overlay ────────────────────────────────────────────────

    fn render_peak_overlay(&self, width: usize, height: usize) -> Vec<String> {
        let peak_bars = self.map_peaks_to_bars(width);
        let mut rows  = vec![String::with_capacity(width); height];

        for row in 0..height {
            let row_from_top = height - 1 - row;
            for &p in &peak_bars {
                let peak_row = ((1.0 - p) * height as f32) as usize;
                let ch = if row_from_top == peak_row && p > 0.02 { '▬' } else { ' ' };
                rows[row].push(ch);
            }
        }
        rows
    }

    fn map_peaks_to_bars(&self, num_bars: usize) -> Vec<f32> {
        if num_bars == 0 { return vec![]; }
        let half = self.peaks.len();
        let mut bars = vec![0.0f32; num_bars];
        for i in 0..num_bars {
            let t0 = (i as f32 / num_bars as f32).powi(2);
            let t1 = ((i + 1) as f32 / num_bars as f32).powi(2);
            let s  = (t0 * half as f32) as usize;
            let e  = ((t1 * half as f32) as usize + 1).min(half);
            let sl = &self.peaks[s..e];
            if !sl.is_empty() {
                bars[i] = sl.iter().cloned().fold(0.0f32, f32::max);
            }
        }
        bars
    }

    // ─── Dots / rain ──────────────────────────────────────────────────────────
    // Each bar column has a "dot" that shoots up on beat then falls

    fn render_dots(&mut self, buffer: &AudioBuffer, width: usize, height: usize) -> Vec<String> {
        let mags = self.fft_magnitudes(buffer);
        let bars = self.map_to_bars(&mags, width);

        // Grow dots_pos to match width
        if self.dots_pos.len() < width {
            self.dots_pos.resize(width, 0.0);
        }

        // Launch dots upward based on bar energy, then fall with gravity
        for (i, &bar) in bars.iter().enumerate() {
            if i >= self.dots_pos.len() { break; }
            if bar > self.dots_pos[i] {
                self.dots_pos[i] = bar; // jump up
            } else {
                self.dots_pos[i] = (self.dots_pos[i] - 0.03).max(0.0); // fall
            }
        }

        let mut rows = vec![String::with_capacity(width); height];
        for row in 0..height {
            let row_from_top = height - 1 - row;
            for (col, &bar) in bars.iter().enumerate() {
                if col >= self.dots_pos.len() { break; }
                let dot_row = ((1.0 - self.dots_pos[col]) * (height - 1) as f32) as usize;
                let bar_row = ((1.0 - bar) * (height - 1) as f32) as usize;

                let ch = if row_from_top == height - 1 - dot_row {
                    '●' // falling dot
                } else if row_from_top <= height - 1 - bar_row {
                    '·' // trail dots
                } else {
                    ' '
                };
                rows[row].push(ch);
            }
        }
        rows
    }
}
