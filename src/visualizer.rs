use crate::audio::AudioBuffer;
use num_complex::Complex;
use rustfft::{FftPlanner, FftDirection};

#[derive(Debug, Clone, PartialEq)]
pub enum VisualizerMode { Bars, Waveform, Spectrum }

impl VisualizerMode {
    pub fn next(&self) -> Self {
        match self {
            VisualizerMode::Bars     => VisualizerMode::Waveform,
            VisualizerMode::Waveform => VisualizerMode::Spectrum,
            VisualizerMode::Spectrum => VisualizerMode::Bars,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            VisualizerMode::Bars     => "Bars",
            VisualizerMode::Waveform => "Waveform",
            VisualizerMode::Spectrum => "Spectrum",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "waveform" => VisualizerMode::Waveform,
            "spectrum" => VisualizerMode::Spectrum,
            _          => VisualizerMode::Bars,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            VisualizerMode::Bars     => "bars",
            VisualizerMode::Waveform => "waveform",
            VisualizerMode::Spectrum => "spectrum",
        }
    }
}

pub struct Visualizer {
    pub mode:        VisualizerMode,
    pub sensitivity: f32,
    fft_size:        usize,
    smoothed:        Vec<f32>,
    planner:         FftPlanner<f32>,
}

impl Visualizer {
    pub fn new(mode: VisualizerMode, sensitivity: f32) -> Self {
        let fft_size = 1024;
        Visualizer {
            mode,
            sensitivity,
            fft_size,
            smoothed: vec![0.0; fft_size / 2],
            planner:  FftPlanner::new(),
        }
    }

    /// Returns normalized bar heights (0.0–1.0) for `num_bars` columns
    pub fn compute_bars(&mut self, buffer: &AudioBuffer, num_bars: usize) -> Vec<f32> {
        match &self.mode {
            VisualizerMode::Bars | VisualizerMode::Spectrum => {
                let magnitudes = self.fft_magnitudes(buffer);
                self.map_to_bars(&magnitudes, num_bars)
            }
            VisualizerMode::Waveform => {
                self.waveform(buffer, num_bars)
            }
        }
    }

    fn fft_magnitudes(&mut self, buffer: &AudioBuffer) -> Vec<f32> {
        let raw: Vec<f32> = {
            let buf = buffer.lock();
            buf.iter().copied().collect()
        };

        let n = self.fft_size.min(raw.len());
        if n < 16 { return vec![0.0; self.fft_size / 2]; }

        // Apply Hanning window
        let mut input: Vec<Complex<f32>> = raw[raw.len() - n..]
            .iter()
            .enumerate()
            .map(|(i, &s)| {
                let window = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / n as f32).cos());
                Complex::new(s * window, 0.0)
            })
            .collect();

        // Pad to fft_size if needed
        input.resize(self.fft_size, Complex::new(0.0, 0.0));

        let fft = self.planner.plan_fft(self.fft_size, FftDirection::Forward);
        fft.process(&mut input);

        // Magnitude of first half (real spectrum)
        let half  = self.fft_size / 2;
        let mags: Vec<f32> = input[..half]
            .iter()
            .map(|c| (c.re * c.re + c.im * c.im).sqrt() / half as f32)
            .collect();

        // Smooth with previous frame
        let sens = self.sensitivity;
        for (sm, &m) in self.smoothed.iter_mut().zip(mags.iter()) {
            let boosted = m * sens * 8.0;
            *sm = (*sm * 0.7 + boosted * 0.3).min(1.0);
        }

        self.smoothed.clone()
    }

    fn map_to_bars(&self, magnitudes: &[f32], num_bars: usize) -> Vec<f32> {
        if num_bars == 0 || magnitudes.is_empty() { return vec![]; }

        let half = magnitudes.len();
        // Log-frequency mapping for more musical feel
        let mut bars = vec![0.0f32; num_bars];
        for i in 0..num_bars {
            let t_start = (i as f32 / num_bars as f32).powi(2);
            let t_end   = ((i + 1) as f32 / num_bars as f32).powi(2);
            let start   = (t_start * half as f32) as usize;
            let end     = ((t_end * half as f32) as usize + 1).min(half);
            let slice   = &magnitudes[start..end];
            if !slice.is_empty() {
                bars[i] = slice.iter().cloned().fold(0.0f32, f32::max);
            }
        }
        bars
    }

    fn waveform(&self, buffer: &AudioBuffer, num_points: usize) -> Vec<f32> {
        let raw: Vec<f32> = {
            let buf = buffer.lock();
            buf.iter().copied().collect()
        };

        if raw.is_empty() { return vec![0.5; num_points]; }

        let step = (raw.len() as f32 / num_points as f32).max(1.0) as usize;
        (0..num_points)
            .map(|i| {
                let idx = (i * step).min(raw.len() - 1);
                (raw[idx] * self.sensitivity + 1.0) / 2.0  // map -1..1 → 0..1
            })
            .collect()
    }

    pub fn render_ascii(&mut self, buffer: &AudioBuffer, width: usize, height: usize) -> Vec<String> {
        let values = self.compute_bars(buffer, width);
        let mut rows = vec![String::new(); height];

        let bar_chars = [' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

        match &self.mode {
            VisualizerMode::Waveform => {
                // Render waveform as a centered line
                for row in 0..height {
                    let mut line = String::new();
                    for &v in &values {
                        let y = (v * height as f32) as usize;
                        let row_inv = height - 1 - row;
                        line.push(if row_inv == y { '─' } else if row_inv == height / 2 { '·' } else { ' ' });
                    }
                    rows[row] = line;
                }
            }
            _ => {
                // Render bar chart from bottom
                for row in 0..height {
                    let mut line = String::new();
                    for &v in &values {
                        let filled_cells = (v * height as f32 * 8.0) as usize;
                        let full_rows    = filled_cells / 8;
                        let partial      = filled_cells % 8;
                        let row_from_top = height - 1 - row;

                        let ch = if row_from_top < full_rows {
                            '█'
                        } else if row_from_top == full_rows {
                            bar_chars[partial]
                        } else {
                            ' '
                        };
                        line.push(ch);
                    }
                    rows[row] = line;
                }
            }
        }
        rows
    }
}
