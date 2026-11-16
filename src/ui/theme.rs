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
/// Make the `width` cells at `(x, y)` an OSC 8 hyperlink to `url`.
///
/// Call it after the cells are drawn. Ratatui has no hyperlink style, so the
/// first cell takes the whole link: the escape sequence, the text of all the
/// cells and the closing sequence. `ForcedWidth` tells the diff that this one
/// cell covers `width` columns, so the backend skips the others. The style of
/// the first cell applies to the whole link. Trailing blanks and the `\`
/// row markers stay outside the link.
///
/// The browser draws through Ratzilla and opens links on its own. It must not
/// get escape bytes, so this exists only off wasm.
#[cfg(not(target_arch = "wasm32"))]
pub fn hyperlink(f: &mut Frame, x: usize, y: usize, width: usize, url: &str) {
    use ratatui::buffer::CellDiffOption;
    use std::num::NonZeroU16;

    let a = f.area();
    if url.is_empty() || x >= a.width as usize || y >= a.height as usize {
        return;
    }
    // A control byte in the URL would end the sequence early.
    let url: String = url.chars().filter(|c| !c.is_control()).collect();
    let width = width.min(a.width as usize - x);
    let (x, y) = (a.x + x as u16, a.y + y as u16);
    let buffer = f.buffer_mut();
    let mut symbols: Vec<String> = (0..width as u16)
        .map(|dx| buffer[(x + dx, y)].symbol().to_string())
        .collect();
    while symbols
        .last()
        .is_some_and(|s| s.trim().is_empty() || s == "\\")
    {
        symbols.pop();
    }
    let Some(width) = NonZeroU16::new(symbols.len() as u16) else {
        return;
    };
    let text = symbols.concat();
    buffer[(x, y)]
        .set_symbol(&format!("\x1b]8;;{url}\x1b\\{text}\x1b]8;;\x1b\\"))
        .set_diff_option(CellDiffOption::ForcedWidth(width));
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
