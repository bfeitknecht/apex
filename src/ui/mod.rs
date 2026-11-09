//! Ratatui rendering: layout and drawing only, no state.
//!
//! Every screen is a function of an [`App`], which keeps the native terminal,
//! the SSH session and the browser (through Ratzilla) pixel-identical.

mod document;
mod nav;
mod portfolio;
mod theme;
mod visuals;

use ratatui::widgets::Clear;
use ratatui::Frame;

use crate::app::{App, View};

pub use document::{document_lines, document_max_scroll};
pub use nav::nav_ranges;
pub use portfolio::{portfolio_page, wrap_lines, LinkKind, PageLink, PageRow, PortfolioPage};
pub use visuals::{unicode_layout, visual_ranges, UnicodeLayout};

/// Draw the current view.
pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();
    frame.render_widget(Clear, area);
    if app.help {
        visuals::draw_help(frame);
        return;
    }
    match app.view {
        View::Portfolio(_) => portfolio::draw(frame, app),
        View::Document(document) => document::draw(frame, app, document),
        View::Visuals => visuals::draw_menu(frame, app),
        View::Unicode => visuals::draw_unicode(frame, app),
        View::Lichen => visuals::draw_lichen(frame, app),
        View::Fractal => visuals::draw_fractal(frame, app),
        View::Crash => visuals::draw_crash(frame, app),
    }
}
