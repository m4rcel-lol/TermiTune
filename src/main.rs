mod app;
mod audio;
mod config;
mod playlist;
mod theme;
mod ui;
mod utils;
mod visualizer;

use anyhow::Result;
use app::App;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::LevelFilter;
use ratatui::{backend::CrosstermBackend, Terminal};
use simplelog::{Config as LogConfig, WriteLogger};
use std::{
    fs,
    io,
    panic,
    time::{Duration, Instant},
};

fn setup_logging() -> Result<()> {
    let log_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("termitune");
    fs::create_dir_all(&log_dir)?;
    let log_file = log_dir.join("termitune.log");
    WriteLogger::init(
        LevelFilter::Info,
        LogConfig::default(),
        fs::File::create(log_file)?,
    )?;
    Ok(())
}

fn main() -> Result<()> {
    // Setup logging
    let _ = setup_logging();

    // Setup panic hook to restore terminal on crash
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
        original_hook(info);
    }));

    // Initialize terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    // Create & run app
    let mut app = App::new()?;
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Error: {e}");
    }

    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<()> {
    let tick_rate = Duration::from_millis(16); // ~60fps
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or(Duration::ZERO);

        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) => {
                    if app.handle_key(key)? {
                        break;
                    }
                }
                Event::Mouse(mouse) => {
                    app.handle_mouse(mouse);
                }
                Event::Resize(w, h) => {
                    app.handle_resize(w, h);
                }
                _ => {}
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.tick();
            last_tick = Instant::now();
        }

        if app.should_quit {
            break;
        }
    }

    app.save_session()?;
    Ok(())
}
