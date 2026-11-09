//! Footer documents (public keys and the ASCII self-portrait) shown in place.

use ratatui::Frame;

use super::theme::{put, rule, MUTED, WHITE};
use crate::app::{App, Document};

pub fn document_lines(document: Document, width: usize) -> Vec<String> {
    let width = width.saturating_sub(4).max(1);
    let mut output = Vec::new();
    for line in document.text().lines() {
        if line.is_empty() {
            output.push(String::new());
            continue;
        }
        let mut chunk = String::new();
        for ch in line.chars() {
            if chunk.chars().count() == width {
                output.push(std::mem::take(&mut chunk));
            }
            chunk.push(ch);
        }
        output.push(chunk);
    }
    output
}

/// Total scrollable rows for a document in a viewport of `height`.
pub fn document_max_scroll(document: Document, width: usize, height: usize) -> usize {
    document_lines(document, width)
        .len()
        .saturating_sub(height.saturating_sub(6).max(1))
}
/// Draw a document with its scroll offset applied.
pub(super) fn draw(frame: &mut Frame, app: &App, document: Document) {
    let width = frame.area().width as usize;
    let height = frame.area().height as usize;
    put(frame, 2, 1, &format!("== {}", document.title()), WHITE);
    rule(frame, 3, width);
    for (row, line) in document_lines(document, width)
        .iter()
        .skip(app.document_scroll)
        .enumerate()
    {
        if row + 6 >= height {
            break;
        }
        put(frame, 2, row + 6, line, MUTED);
    }
}
