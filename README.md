# TermiTune 🎵

```
  ████████╗███████╗██████╗ ███╗   ███╗██╗████████╗██╗   ██╗███╗   ██╗███████╗
     ██╔══╝██╔════╝██╔══██╗████╗ ████║██║╚══██╔══╝██║   ██║████╗  ██║██╔════╝
     ██║   █████╗  ██████╔╝██╔████╔██║██║   ██║   ██║   ██║██╔██╗ ██║█████╗  
     ██║   ██╔══╝  ██╔══██╗██║╚██╔╝██║██║   ██║   ██║   ██║██║╚██╗██║██╔══╝  
     ██║   ███████╗██║  ██║██║ ╚═╝ ██║██║   ██║   ╚██████╔╝██║ ╚████║███████╗
     ╚═╝   ╚══════╝╚═╝  ╚═╝╚═╝     ╚═╝╚═╝   ╚═╝    ╚═════╝ ╚═╝  ╚═══╝╚══════╝
```

> A beautiful, fast, fully customizable TUI music player built natively for Arch Linux.

---

## ✨ Features

- **Real-time audio visualizer** — FFT-powered bars, waveform, and spectrum modes
- **JSON theme system** — 3 built-in themes + custom theme support
- **Full playlist management** — scan folders, shuffle, loop, search, save/load
- **Vim-style navigation** — keyboard-driven with intuitive controls
- **Session restore** — remembers last playlist and position
- **Modular Rust architecture** — fast, memory-safe, 60fps TUI

---

## 📸 UI Preview

```
╭─ File Browser ──────────────────╮╭─ ▶ Now Playing ────────────────────╮
│ 📂 ~/Music                       ││ ▶ PLAYING                           │
│  󰉋 Albums                        ││ ♪  Midnight City                    │
│  󰉋 Singles                       ││    M83 — Hurry Up, We're Dreaming   │
│ ♪ 01-midnight-city.mp3          ││    2011                              │
│ ♪ 02-reunion.mp3                ││ ████████████░░░░░░░░░  2:34 / 4:03  │
│ ♪ 03-soon.mp3                   ││ Volume: 70%  ⟲ ALL                   │
╰──────────────────────────────────╯╰──────────────────────────────────────╯
╭─ Queue (12 tracks) ─────────────────────────────────────────────────────────╮
│ ▶ M83 - Midnight City                                                  4:03 │
│   M83 - Reunion                                                        3:55 │
│   M83 - Soon                                                           4:38 │
│   Boards of Canada - Roygbiv                                           2:27 │
╰─────────────────────────────────────────────────────────────────────────────╯
 ▶ M83 — Midnight City   [████████░░░░░░░]  2:34 / 4:03   vol: 70%   ⟲ ALL

 [Tab] Switch panel  [Enter] Select  [Space] Play/Pause  [n/p] Next/Prev  [q] Quit
```

---

## 🚀 Installation (Arch Linux)

### Prerequisites

```bash
sudo pacman -S rust alsa-lib pkgconf
```

### Build & Run

```bash
git clone https://github.com/m4rcel-lol/termitune
cd termitune
chmod +x build.sh
./build.sh          # builds release binary
./build.sh install  # installs to /usr/local/bin
```

### Or with Make

```bash
make install        # build + install
make run            # debug run
make help           # show all targets
```

---

## ⌨️ Keybindings

| Key           | Action                        |
|---------------|-------------------------------|
| `Space`       | Play / Pause                  |
| `n`           | Next track                    |
| `p`           | Previous track                |
| `l`           | Cycle loop mode (off/one/all) |
| `s`           | Toggle shuffle                |
| `v`           | Cycle visualizer mode         |
| `t`           | Cycle theme                   |
| `/`           | Search playlist               |
| `+` / `-`     | Volume up / down              |
| `m`           | Mute toggle                   |
| `f` / `b`     | Seek forward / backward       |
| `j` / `k`     | Navigate down / up            |
| `h` / `l`     | Browser: go up / enter dir    |
| `Tab`         | Switch panel focus            |
| `Enter`       | Select / play item            |
| `a`           | Add current folder to queue   |
| `d` / `Del`   | Remove track from playlist    |
| `w`           | Save playlist to file         |
| `c`           | Clear playlist                |
| `1`           | Home page                     |
| `2`           | Now Playing page              |
| `3`           | Settings page                 |
| `4`           | Credits page                  |
| `q`           | Quit                          |

---

## 🎨 Theming

Themes live in `~/.config/termitune/themes/` as JSON files.

Built-in themes: `default`, `dark`, `neon`

### Theme format

```json
{
  "name": "my-theme",
  "description": "My custom theme",
  "border_type": "rounded",
  "colors": {
    "fg":           "#d8dee9",
    "bg":           "#2e3440",
    "accent":       "#88c0d0",
    "progress_bar": "#5e81ac",
    "visualizer":   ["#88c0d0", "#81a1c1", "#5e81ac", "#8fbcbb"],
    "highlight_fg": "#2e3440",
    "highlight_bg": "#88c0d0",
    "border":       "#4c566a",
    "title":        "#eceff4",
    "subtitle":     "#88c0d0",
    "muted":        "#4c566a"
  }
}
```

`border_type` options: `"rounded"`, `"plain"`, `"double"`, `"thick"`

Press `t` in-app to cycle through available themes. Reload without restart: press `t` multiple times.

---

## ⚙️ Configuration

Config file: `~/.config/termitune/config.json`

```json
{
  "music_dir": "~/Music",
  "theme": "default",
  "volume": 0.7,
  "visualizer_mode": "bars",
  "visualizer_sensitivity": 1.0,
  "restore_session": true
}
```

---

## 🏗 Architecture

```
src/
 ├── main.rs          — entry, terminal init, run loop (~60fps)
 ├── app.rs           — AppState, event routing, session
 ├── audio.rs         — rodio wrapper + CaptureSource for FFT
 ├── visualizer.rs    — FFT analysis (rustfft), bars/waveform/spectrum
 ├── ui/
 │   ├── mod.rs       — page dispatcher
 │   ├── home.rs      — 3-panel home layout
 │   ├── player.rs    — full player + visualizer page
 │   ├── browser.rs   — full-screen file browser
 │   ├── settings.rs  — settings page
 │   ├── credits.rs   — credits page
 │   └── widgets.rs   — shared widgets (status bar, key hints)
 ├── theme/mod.rs     — theme system (JSON + built-ins)
 ├── playlist/mod.rs  — Track, Playlist, PlaylistManager
 ├── config/mod.rs    — Config (JSON persist)
 └── utils/mod.rs     — format helpers
```

### Why Rust?

- **Memory safety** without GC — zero-cost abstractions at 60fps
- **`ratatui`** is the most mature Rust TUI library
- **`rodio`** wraps CPAL/ALSA natively — no external daemon needed
- **`rustfft`** gives us proper FFT for the visualizer without C bindings
- **`lofty`** reads MP3/FLAC/OGG tags natively
- Compile to a single static-ish binary — no runtime deps beyond ALSA

---

## 📦 Building Manually (PKGBUILD)

A `PKGBUILD` template is included for future AUR submission:

```bash
makepkg -si
```

---

## 📄 License

Nice © m4rcel-lol
