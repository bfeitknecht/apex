//! Terminal rendering of the hidden menu and the four visuals.
//!
//! The visuals live in [`crate::visuals`]. This module only decides which cells
//! to paint. The browser frontend reuses [`unicode_layout`] to place its canvas
//! glyph, and it draws lichen as pixels instead of half blocks.

use ratatui::style::Color;
use ratatui::Frame;

use super::theme::{put, put_selected, rule, MUTED, WHITE};
use crate::app::App;
#[cfg(target_arch = "wasm32")]
use crate::app::FontStatus;
use crate::visuals::{crash, Unicode, VISUALS};

/// Start column, row and width of every entry in the hidden menu.
pub fn visual_ranges(width: usize, height: usize) -> Vec<(usize, usize, usize)> {
    let labels: Vec<&str> = VISUALS.iter().map(|visual| visual.label()).collect();
    let total: usize = labels.iter().map(|label| label.len()).sum::<usize>() + 2 * labels.len();
    if width >= total {
        let mut x = (width - total) / 2;
        labels
            .iter()
            .map(|label| {
                let range = (x, height / 2, label.len());
                x += label.len() + 2;
                range
            })
            .collect()
    } else {
        let top = height.saturating_sub(labels.len() + 1) / 2;
        labels
            .iter()
            .enumerate()
            .map(|(index, label)| {
                (
                    width.saturating_sub(label.len()) / 2,
                    top + index,
                    label.len(),
                )
            })
            .collect()
    }
}

/// Metadata lines under the specimen, and how many of them are the heading.
pub(super) fn unicode_metadata(unicode: &Unicode, width: usize) -> (Vec<String>, usize) {
    let width = width.clamp(1, 100);
    let code = unicode.glyph() as u32;
    let mut lines = Vec::new();
    fn push_wrapped(lines: &mut Vec<String>, text: &str, width: usize) {
        let mut line = String::new();
        for word in text.split_whitespace() {
            if !line.is_empty() && line.chars().count() + 1 + word.chars().count() > width {
                lines.push(std::mem::take(&mut line));
            }
            if !line.is_empty() {
                line.push(' ');
            }
            for ch in word.chars() {
                if line.chars().count() >= width {
                    lines.push(std::mem::take(&mut line));
                }
                line.push(ch);
            }
        }
        if !line.is_empty() {
            lines.push(line);
        }
    }
    let family = match unicode.font_family() {
        "NotoStudy" => "NOTO SANS".to_string(),
        "NotoSymbols" => "NOTO SYMBOLS".to_string(),
        "NotoSymbols2" => "NOTO SYMBOLS 2".to_string(),
        "NotoMath" => "NOTO MATH".to_string(),
        "BabelStoneHan" => "BABELSTONE HAN".to_string(),
        family if family.starts_with("NotoSans") => {
            let suffix = family.strip_prefix("NotoSans").unwrap_or(family);
            let mut label = String::from("NOTO SANS ");
            for (i, c) in suffix.chars().enumerate() {
                if i > 0 && c.is_ascii_uppercase() {
                    label.push(' ');
                }
                label.push(c.to_ascii_uppercase());
            }
            label
        }
        _ => "GNU UNIFONT".to_string(),
    };
    let heading = format!("U+{code:04X} {} ({family})", unicode.name());
    push_wrapped(&mut lines, &heading, width);
    let heading_lines = lines.len();
    if let Some(block) = unicode.block() {
        lines.push(String::new());
        let bar = |value: u32, lo: u32, hi: u32| {
            let mut b = vec![b'-'; width];
            let pos = ((value.saturating_sub(lo) as u64 * (width - 1) as u64)
                / (hi.saturating_sub(lo).max(1) as u64)) as usize;
            b[pos.min(width - 1)] = b'+';
            String::from_utf8(b).unwrap_or_default()
        };
        lines.push(bar(code, 0, 0x10ffff));
        push_wrapped(&mut lines, "UNICODE U+0000..U+10FFFF", width);
        lines.push(String::new());
        lines.push(bar(code, block.start(), block.end()));
        push_wrapped(
            &mut lines,
            &format!("BLOCK U+{:04X}..U+{:04X}", block.start(), block.end()),
            width,
        );
    }
    (lines, heading_lines)
}
pub(super) fn metadata_foreground(line: usize, heading_lines: usize) -> Color {
    if line < heading_lines {
        WHITE
    } else {
        MUTED
    }
}
pub struct UnicodeLayout {
    pub metadata_x: usize,
    pub metadata_y: usize,
    pub metadata_width: usize,
    pub glyph_x: usize,
    pub glyph_y: usize,
    pub glyph_width: usize,
    pub glyph_height: usize,
}
/// Where the enlarged specimen and its metadata go for a given viewport.
pub fn unicode_layout(unicode: &Unicode, width: usize, height: usize) -> UnicodeLayout {
    let metadata_width = width.saturating_sub(4);
    let (lines, _) = unicode_metadata(unicode, metadata_width);
    let y = height.saturating_sub(lines.len() + 1).max(6);
    UnicodeLayout {
        metadata_x: 2,
        metadata_y: y,
        metadata_width,
        glyph_x: 2,
        glyph_y: 1,
        glyph_width: width.saturating_sub(4),
        glyph_height: y.saturating_sub(2),
    }
}

/// The hidden menu: the visuals, centred, in `VISUALS` order.
pub(super) fn draw_menu(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let visuals = VISUALS;
    for (index, &(x, y, _)) in visual_ranges(area.width as usize, area.height as usize)
        .iter()
        .enumerate()
    {
        put_selected(
            frame,
            x,
            y,
            visuals[index].label(),
            app.menu == index || app.hovered_link == Some(index),
        );
    }
}

/// The Crash sweep: one CP437 character per cell, sixteen EGA colours.
pub(super) fn draw_crash(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let (width, height) = (area.width as usize, area.height as usize);
    let colour = |index: u8| {
        let (r, g, b) = crash::PALETTE[index as usize];
        Color::Rgb(r, g, b)
    };
    let buffer = frame.buffer_mut();
    for y in 0..height {
        for x in 0..width {
            let (glyph, foreground, background) = app.crash.cell(x, y);
            buffer[(area.x + x as u16, area.y + y as u16)]
                .set_char(glyph)
                .set_fg(colour(foreground))
                .set_bg(colour(background));
        }
    }
}

/// The quaternion Julia explorer, one character per cell.
pub(super) fn draw_fractal(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let width = area.width as usize;
    for (row, line) in app
        .fractal
        .rows(width, area.height as usize)
        .iter()
        .enumerate()
    {
        put(frame, 0, row, line, MUTED);
    }
}

/// The colony as half blocks: two vertically stacked pixels per character cell.
pub(super) fn draw_lichen(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let (width, height) = (area.width as usize, area.height as usize);
    let (grid_width, grid_height) = app.lichen.size();
    let factor = 1usize << app.lichen_pixelation;

    // Pool the simulation into display pixels, so a sparse spore survives the
    // downsample and does not fall between samples.
    let pooled_width = grid_width.div_ceil(factor);
    let pooled_height = grid_height.div_ceil(factor);
    let mut pooled = vec![0u8; pooled_width * pooled_height];
    for y in 0..grid_height {
        for x in 0..grid_width {
            let at = (y / factor) * pooled_width + x / factor;
            pooled[at] = pooled[at].max(app.lichen.display_value(y * grid_width + x));
        }
    }

    let buffer = frame.buffer_mut();
    for y in 0..height {
        for x in 0..width {
            let column = (x * grid_width / width.max(1)) / factor;
            let top = ((y * 2) * grid_height / (height * 2).max(1)) / factor;
            let bottom = ((y * 2 + 1) * grid_height / (height * 2).max(1)) / factor;
            let grey = |index: usize| {
                let level = pooled[index];
                Color::Rgb(level, level, level)
            };
            buffer[(area.x + x as u16, area.y + y as u16)]
                .set_char('▀')
                .set_fg(grey(top * pooled_width + column))
                .set_bg(grey(bottom * pooled_width + column));
        }
    }
}

/// The Unicode specimen: metadata everywhere, plus the glyph in the terminal.
pub(super) fn draw_unicode(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let (width, height) = (area.width as usize, area.height as usize);
    let Some(unicode) = app.unicode.as_ref() else {
        put(frame, 1, 1, "unicode catalogue not loaded", MUTED);
        return;
    };
    let layout = unicode_layout(unicode, width, height);
    let (lines, heading_lines) = unicode_metadata(unicode, layout.metadata_width);
    for (index, line) in lines.iter().enumerate() {
        put(
            frame,
            layout.metadata_x,
            layout.metadata_y + index,
            line,
            metadata_foreground(index, heading_lines),
        );
    }
    // A terminal cannot scale a glyph. It shows the glyph once at the top of the
    // specimen area and leaves the rest empty.
    #[cfg(not(target_arch = "wasm32"))]
    put(
        frame,
        layout.glyph_x + 2,
        layout.glyph_y + 1,
        &unicode.glyph().to_string(),
        WHITE,
    );
    #[cfg(target_arch = "wasm32")]
    if app.unicode_font_status == FontStatus::Failed {
        put(frame, 1, 1, "font unavailable", MUTED);
    }
}

/// The keyboard reference, reachable with `?` from every view.
pub(super) fn draw_help(frame: &mut Frame) {
    let width = frame.area().width as usize;
    const KEYS: [&str; 9] = [
        "h / l           previous / next section",
        "j / k           scroll down / up",
        "1 - 3           jump to section",
        "g g / shift g   top / bottom",
        "n / shift n     next / previous link",
        "enter           open selected link",
        "esc             back / close help",
        "?               toggle this help",
        "q               quit terminal",
    ];
    put(frame, 1, 1, "apex / keyboard", WHITE);
    rule(frame, 3, width);
    for (index, line) in KEYS.iter().enumerate() {
        put(
            frame,
            1,
            5 + index * 2,
            line,
            if index == 0 { WHITE } else { MUTED },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrapped_unicode_heading_keeps_its_foreground() {
        assert_eq!(metadata_foreground(0, 2), WHITE);
        assert_eq!(metadata_foreground(1, 2), WHITE);
        assert_eq!(metadata_foreground(2, 2), MUTED);
    }

    #[test]
    fn menu_entries_are_centred_and_ordered() {
        let ranges = visual_ranges(80, 24);
        assert_eq!(ranges.len(), VISUALS.len());
        assert!(ranges.windows(2).all(|pair| pair[0].0 < pair[1].0));
        assert!(ranges.iter().all(|&(_, row, _)| row == 12));
    }

    #[test]
    fn narrow_terminals_stack_the_menu_entries() {
        let ranges = visual_ranges(10, 24);
        assert!(ranges.windows(2).all(|pair| pair[0].1 < pair[1].1));
    }
}
