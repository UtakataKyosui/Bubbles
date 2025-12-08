use std::io;
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use anyhow::Result;

mod app;
mod ui;

use app::App;
use ui::ui;

#[tokio::main]
async fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new().await?;

    // Initial fetch
    app.refresh_timeline().await;

    let mut last_tick = std::time::Instant::now();
    let tick_rate = std::time::Duration::from_secs(10); // Auto refresh every 10s

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if crossterm::event::poll(std::time::Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if app.input_mode {
                     match key.code {
                        KeyCode::Enter => {
                            app.publish_input().await;
                        }
                        KeyCode::Esc => {
                            app.input_mode = false;
                        }
                        _ => {
                            app.input.input(key);
                        }
                     }
                } else {
                    match key.code {
                        KeyCode::Esc => break,
                        KeyCode::Char('r') => {
                            app.refresh_timeline().await;
                        }
                        KeyCode::Char('i') => {
                            app.input_mode = true;
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            app.scroll_down();
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            app.scroll_up();
                        }
                        _ => {}
                    }
                }
            }
        }
        
        if app.should_quit {
            break;
        }
        
        if last_tick.elapsed() >= tick_rate {
            app.refresh_timeline().await;
            last_tick = std::time::Instant::now();
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
