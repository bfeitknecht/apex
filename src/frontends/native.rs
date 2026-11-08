use apex::{ui, App};
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, MouseEventKind,
};
use std::{
    io,
    time::{Duration, Instant},
};
pub fn run() -> io::Result<()> {
    let mut terminal = ratatui::init();
    if let Err(e) = crossterm::execute!(io::stdout(), EnableMouseCapture) {
        ratatui::restore();
        return Err(e);
    }
    let result = (|| {
        let mut app = App::new();
        let mut last = Instant::now();

        loop {
            let size = terminal.size()?;
            app.resize(size.width, size.height);
            // Half blocks give the terminal two vertical pixels per character cell.
            app.lichen.resize(
                size.width as usize,
                (size.height as usize).saturating_mul(2),
            );
            let elapsed = last.elapsed();
            if elapsed >= Duration::from_millis(1) {
                let ms = elapsed.as_millis().min(u64::MAX as u128) as u64;
                app.advance_lichen(ms);
                app.advance_fractal(ms);
                last = Instant::now();
            }
            terminal.draw(|f| ui::draw(f, &app))?;
            if event::poll(Duration::from_millis(10))? {
                match event::read()? {
                    Event::Key(k) if k.kind == KeyEventKind::Press => {
                        app.clear_hover();
                        match k.code {
                            KeyCode::Char('q') => break,
                            KeyCode::Esc => app.back(),
                            KeyCode::Enter => {
                                let _ = app.activate();
                            }
                            KeyCode::Tab => app.focus_link(false),
                            KeyCode::BackTab => app.focus_link(true),
                            KeyCode::Left => app.change_section(-1),
                            KeyCode::Right => app.change_section(1),
                            KeyCode::Char(c) => app.key(c),
                            KeyCode::Up => app.navigate(-1),
                            KeyCode::Down => app.navigate(1),
                            KeyCode::PageUp => {
                                app.scroll(-(app.height.saturating_sub(4).max(1) as i32))
                            }
                            KeyCode::PageDown => {
                                app.scroll(app.height.saturating_sub(4).max(1) as i32)
                            }
                            KeyCode::Home => {
                                app.key('g');
                                app.key('g');
                            }
                            KeyCode::End => app.key('G'),
                            _ => {}
                        }
                    }
                    Event::Mouse(m) => match m.kind {
                        MouseEventKind::Moved => {
                            app.hover(m.column as usize, m.row as usize);
                        }
                        MouseEventKind::ScrollDown => app.navigate(3),
                        MouseEventKind::ScrollUp => app.navigate(-3),
                        MouseEventKind::Down(_) => {
                            let _ = app.click(m.column as usize, m.row as usize);
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
