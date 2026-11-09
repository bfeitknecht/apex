//! Native terminal frontend: crossterm events into the shared [`App`].

use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, MouseEventKind,
};

use crate::{ui, App};

/// Frame interval while nothing else is pending.
const POLL_INTERVAL: Duration = Duration::from_millis(10);

/// Run the portfolio in the current terminal until the visitor quits.
pub fn run() -> io::Result<()> {
    let mut terminal = ratatui::init();
    if let Err(error) = crossterm::execute!(io::stdout(), EnableMouseCapture) {
        ratatui::restore();
        return Err(error);
    }
    let result = (|| {
        let mut app = App::new();
        let mut last_generation = Instant::now();
        loop {
            let size = terminal.size()?;
            app.resize(size.width, size.height);
            // Half blocks give the terminal two vertical pixels per cell.
            app.lichen.resize(
                size.width as usize,
                (size.height as usize).saturating_mul(2),
            );

            let elapsed = last_generation.elapsed();
            if elapsed >= Duration::from_millis(1) {
                let milliseconds = elapsed.as_millis().min(u64::MAX as u128) as u64;
                app.tick(milliseconds);
                last_generation = Instant::now();
            }

            terminal.draw(|frame| ui::draw(frame, &app))?;

            if event::poll(POLL_INTERVAL)? {
                match event::read()? {
                    Event::Key(key) if key.kind == KeyEventKind::Press => {
                        app.clear_hover();
                        if !apply_key(&mut app, key.code) {
                            break;
                        }
                    }
                    Event::Mouse(mouse) => match mouse.kind {
                        MouseEventKind::Moved => {
                            app.hover(mouse.column as usize, mouse.row as usize);
                        }
                        MouseEventKind::ScrollDown => app.navigate(3),
                        MouseEventKind::ScrollUp => app.navigate(-3),
                        MouseEventKind::Down(_) => {
                            let _ = app.click(mouse.column as usize, mouse.row as usize);
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
        Ok(())
    })();

    let _ = crossterm::execute!(io::stdout(), DisableMouseCapture);
    ratatui::restore();
    result
}

/// Apply one terminal key press. Returns `false` when the visitor quits.
fn apply_key(app: &mut App, code: KeyCode) -> bool {
    match code {
        KeyCode::Char('q') => return false,
        KeyCode::Esc => app.back(),
        KeyCode::Enter => {
            let _ = app.activate();
        }
        KeyCode::Tab => app.focus_link(false),
        KeyCode::BackTab => app.focus_link(true),
        KeyCode::Left => app.change_section(-1),
        KeyCode::Right => app.change_section(1),
        KeyCode::Char(character) => app.key(character),
        KeyCode::Up => app.navigate(-1),
        KeyCode::Down => app.navigate(1),
        KeyCode::PageUp => app.scroll(-page_step(app)),
        KeyCode::PageDown => app.scroll(page_step(app)),
        KeyCode::Home => {
            app.key('g');
            app.key('g');
        }
        KeyCode::End => app.key('G'),
        _ => {}
    }
    true
}

fn page_step(app: &App) -> i32 {
    app.height.saturating_sub(4).max(1) as i32
}
