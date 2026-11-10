//! Monochrome palette and the buffer primitives every screen shares.
//!
//! The palette is small. A terminal supplies its own foreground and background,
//! so only the accent greys are fixed.

use ratatui::style::{Color, Modifier, Style};
use ratatui::Frame;

pub const WHITE: Color = Color::Rgb(235, 235, 235);
pub const MUTED: Color = Color::Rgb(130, 130, 130);
pub const DIM: Color = Color::Rgb(75, 75, 75);
// Terminals supply their own foreground/background, including over SSH. Browsers
// use a dark canvas which CSS adapts to the system color scheme.
pub fn text_style(fg: Color) -> Style {
    #[cfg(target_arch = "wasm32")]
    {
        Style::default().fg(fg).bg(Color::Black)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Style::default()
            .fg(if fg == WHITE { Color::Reset } else { fg })
            .bg(Color::Reset)
    }
}
pub fn put(f: &mut Frame, x: usize, y: usize, s: &str, fg: Color) {
    let a = f.area();
    if x >= a.width as usize || y >= a.height as usize {
        return;
    }
    f.buffer_mut().set_stringn(
        a.x + x as u16,
        a.y + y as u16,
        s,
        a.width as usize - x,
        text_style(fg),
    );
}
pub fn put_selected(f: &mut Frame, x: usize, y: usize, s: &str, selected: bool) {
    let a = f.area();
    if x >= a.width as usize || y >= a.height as usize {
        return;
    }
    let mut style = text_style(if selected { WHITE } else { MUTED });
    if selected {
        style = style.add_modifier(Modifier::BOLD);
    }
    f.buffer_mut().set_stringn(
        a.x + x as u16,
        a.y + y as u16,
        s,
        a.width as usize - x,
        style,
    );
}
pub fn rule(f: &mut Frame, y: usize, w: usize) {
    put(f, 1, y, &"-".repeat(w.saturating_sub(4)), DIM);
}
