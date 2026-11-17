//! Shared application state: navigation, scrolling and animation timing.
//!
//! An [`App`] knows nothing about terminals or pixels. The native terminal, the
//! SSH session and the browser each decode their own input into these methods,
//! call [`App::tick`] once per frame and hand the result to [`crate::ui::draw`].

mod input;

#[cfg(test)]
mod tests;

use crate::content;
use crate::ui;
use crate::visuals::{self, Crash, Fractal, Lichen, Unicode};

pub use crate::content::Document;

/// Screens the application can show.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum View {
    /// The portfolio, with the index of the section currently in view.
    Portfolio(usize),
    /// The hidden menu that lists the visuals.
    Visuals,
    /// The current Unicode specimen.
    Unicode,
    /// The growing lichen colony.
    Lichen,
    /// The quaternion Julia explorer.
    Fractal,
    /// The text-mode framebuffer sweep.
    Crash,
    /// A footer document, read inside the terminal.
    Document(Document),
}

impl View {
    /// Whether this view draws a [`crate::visuals`] visual.
    pub fn is_visual(self) -> bool {
        matches!(
            self,
            Self::Unicode | Self::Lichen | Self::Fractal | Self::Crash
        )
    }
}

/// How far the browser has got with loading the specimen's font.
#[cfg(target_arch = "wasm32")]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FontStatus {
    #[default]
    NotRequested,
    Loading,
    Ready,
    Failed,
}

/// Everything the portfolio remembers between frames.
pub struct App {
    pub lichen: Lichen,
    pub fractal: Fractal,
    pub crash: Crash,
    /// Pixelation of the display. The simulation keeps its own resolution.
    pub lichen_pixelation: u8,
    /// Milliseconds between simulation generations.
    pub lichen_interval_ms: u64,
    lichen_elapsed_ms: u64,
    pub view: View,
    /// Section to return to after the hidden menu closes.
    pub previous: usize,
    pub scroll_offset: usize,
    pub document_scroll: usize,
    pub focused_link: Option<usize>,
    pub hovered_link: Option<usize>,
    pub hovered_nav: Option<usize>,
    pub focused_nav: Option<usize>,
    /// Index into the hidden menu.
    pub menu: usize,
    logo_activations: u8,
    logo_last_ms: u64,
    /// Built on first use. Parsing the catalogue at startup takes too long.
    pub unicode: Option<Unicode>,
    #[cfg(target_arch = "wasm32")]
    pub unicode_font_status: FontStatus,
    pub paused: bool,
    pub help: bool,
    pub width: usize,
    pub height: usize,
    pending_g: bool,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            lichen: Lichen::new(seed()),
            fractal: Fractal::new(),
            crash: Crash::new(),
            lichen_pixelation: 0,
            lichen_interval_ms: 66,
            lichen_elapsed_ms: 0,
            view: View::Portfolio(0),
            previous: 0,
            scroll_offset: 0,
            document_scroll: 0,
            focused_link: None,
            hovered_link: None,
            hovered_nav: None,
            focused_nav: None,
            menu: 0,
            logo_activations: 0,
            logo_last_ms: 0,
            unicode: None,
            #[cfg(target_arch = "wasm32")]
            unicode_font_status: FontStatus::NotRequested,
            paused: false,
            help: false,
            width: 80,
            height: 24,
            pending_g: false,
        }
    }

    /// Record the terminal size and keep the current view consistent with it.
    pub fn resize(&mut self, width: u16, height: u16) {
        self.width = width as usize;
        self.height = height as usize;
        if matches!(self.view, View::Portfolio(_)) {
            self.set_scroll(self.scroll_offset);
        } else if let View::Document(document) = self.view {
            self.document_scroll = self.document_scroll.min(ui::document_max_scroll(
                document,
                self.width,
                self.height,
            ));
        }
    }

    /// Advance the animation of the current view. Returns `true` when a new frame
    /// is due.
    ///
    /// Every frontend calls this function once per frame, so the three surfaces
    /// keep the same pacing. Each visual caps its own work, so a suspended browser
    /// tab or a stalled session does not delay the next frame.
    pub fn tick(&mut self, elapsed_ms: u64) -> bool {
        let mut dirty = false;
        if self.view == View::Lichen && !self.paused {
            dirty |= self.advance_lichen(elapsed_ms);
        } else {
            self.lichen_elapsed_ms = 0;
        }
        if self.view == View::Fractal && !self.help {
            dirty |= self.advance_fractal(elapsed_ms);
        } else {
            self.fractal.reset_clock();
        }
        if self.view == View::Crash && !self.help {
            dirty |= self.crash.advance(elapsed_ms);
        }
        dirty
    }

    /// Run the colony forward, allowing catch-up generations.
    fn advance_lichen(&mut self, elapsed_ms: u64) -> bool {
        const MAX_CATCH_UP: u64 = 8;
        self.lichen_elapsed_ms = self
            .lichen_elapsed_ms
            .saturating_add(elapsed_ms)
            .min(self.lichen_interval_ms * MAX_CATCH_UP);
        let generations = self.lichen_elapsed_ms / self.lichen_interval_ms;
        self.lichen_elapsed_ms %= self.lichen_interval_ms;
        for _ in 0..generations {
            self.lichen.step();
        }
        generations > 0
    }

    /// Rotate and zoom the explorer. Returns `true` when a new frame is due.
    fn advance_fractal(&mut self, elapsed_ms: u64) -> bool {
        self.fractal.advance(elapsed_ms)
    }

    /// Index of the portfolio section in view, or of the hidden menu.
    pub fn nav_index(&self) -> usize {
        match self.view {
            View::Portfolio(index) => index,
            // Past the three sections: no top-bar entry is active in a visual.
            View::Visuals | View::Unicode | View::Lichen | View::Fractal | View::Crash => 3,
            View::Document(_) => {
                ui::portfolio_page(self.width, self.height).active(self.scroll_offset)
            }
        }
    }

    /// Scroll the portfolio to `requested`, clamped to the page.
    fn set_scroll(&mut self, requested: usize) {
        let page = ui::portfolio_page(self.width, self.height);
        self.scroll_offset = requested.min(page.max_scroll());
        self.view = View::Portfolio(page.active(self.scroll_offset));
    }

    /// Jump to a portfolio section, or open the hidden menu for index 3.
    pub fn select_section(&mut self, index: usize) {
        self.help = false;
        self.pending_g = false;
        self.focused_link = None;
        self.focused_nav = None;
        self.hovered_link = None;
        if index >= 3 {
            if let View::Portfolio(section) = self.view {
                self.previous = section;
            }
            self.view = View::Visuals;
            self.menu = 0;
        } else {
            let page = ui::portfolio_page(self.width, self.height);
            self.set_scroll(page.offsets[index]);
        }
    }

    /// Move through the portfolio links and the external navigation entries.
    pub fn focus_link(&mut self, reverse: bool) {
        self.clear_hover();
        if self.help {
            return;
        }
        if self.view == View::Visuals {
            let count = visuals::VISUALS.len();
            self.menu = if reverse {
                (self.menu + count - 1) % count
            } else {
                (self.menu + 1) % count
            };
            return;
        }
        if !matches!(self.view, View::Portfolio(_)) {
            return;
        }
        let page = ui::portfolio_page(self.width, self.height);
        if page.links.is_empty() {
            return;
        }
        // Slots 0-2 are the sections, 3.. are the destinations in `content`.
        let last_nav = 2 + content::profile().links.len();
        if let Some(nav) = self.focused_nav {
            if nav == 0 {
                if reverse {
                    self.focused_nav = None;
                    self.focused_link = Some(page.links.len() - 1);
                } else {
                    self.focused_nav = Some(3);
                    return;
                }
            } else if (!reverse && nav < last_nav) || (reverse && nav > 3) {
                self.focused_nav = Some(if reverse { nav - 1 } else { nav + 1 });
                return;
            } else {
                self.focused_nav = None;
                if reverse {
                    self.focused_nav = Some(0);
                    return;
                }
                self.focused_link = Some(0);
            }
        } else if self.focused_link.is_none() {
            if !reverse {
                self.focused_nav = Some(0);
                return;
            }
        } else if (!reverse && self.focused_link == Some(page.links.len() - 1))
            || (reverse && self.focused_link == Some(0))
        {
            self.focused_link = None;
            self.focused_nav = Some(if reverse { last_nav } else { 0 });
            return;
        } else if let Some(index) = self.focused_link {
            self.focused_link = Some(if reverse { index - 1 } else { index + 1 });
        }
        // Scroll the newly focused link into view, keeping the footer pinned.
        let next = self.focused_link.unwrap_or(page.links.len() - 1);
        let row = page.links[next].row;
        let requested = if row >= page.footer_start {
            page.max_scroll()
        } else if row < self.scroll_offset {
            row
        } else if row >= self.scroll_offset + page.viewport {
            row + 1 - page.viewport
        } else {
            self.scroll_offset
        };
        self.set_scroll(requested);
        self.focused_link = Some(next);
    }

    /// Move to the previous or next portfolio section.
    pub fn change_section(&mut self, delta: i32) {
        self.clear_hover();
        if self.view == View::Fractal {
            self.fractal.pan(delta as f64, 0.0);
            return;
        }
        if self.view == View::Visuals {
            let last = visuals::VISUALS.len().saturating_sub(1) as i32;
            self.menu = (self.menu as i32 + delta).clamp(0, last) as usize;
            return;
        }
        let next = (self.nav_index() as i32 + delta).clamp(0, 2) as usize;
        self.select_section(next);
    }

    /// Leave the current view: close help, then a visual, then a document.
    pub fn back(&mut self) {
        if self.help {
            self.help = false;
        } else if self.view.is_visual() {
            self.view = View::Visuals;
        } else if self.view == View::Visuals {
            self.view = View::Portfolio(self.previous);
        } else if matches!(self.view, View::Document(_)) {
            self.set_scroll(self.scroll_offset);
        }
    }

    pub fn is_unicode(&self) -> bool {
        self.view == View::Unicode
    }

    /// Scroll or move within the current view.
    pub fn navigate(&mut self, delta: i32) {
        if self.help {
            return;
        }
        match self.view {
            View::Lichen => {
                let (width, height) = self.lichen.size();
                let max_level = width
                    .max(height)
                    .max(1)
                    .next_power_of_two()
                    .trailing_zeros() as i32;
                self.lichen_pixelation =
                    (self.lichen_pixelation as i32 + delta).clamp(0, max_level) as u8;
            }
            View::Unicode => {
                if let Some(unicode) = self.unicode.as_mut() {
                    for _ in 0..delta.unsigned_abs().min(8) {
                        if delta > 0 {
                            unicode.next();
                        } else {
                            unicode.previous();
                        }
                    }
                }
            }
            View::Portfolio(_) => {
                let page = ui::portfolio_page(self.width, self.height);
                let next =
                    (self.scroll_offset as i64 + delta as i64).clamp(0, page.max_scroll() as i64);
                self.focused_link = None;
                self.hovered_link = None;
                self.set_scroll(next as usize);
            }
            View::Visuals => {
                let last = visuals::VISUALS.len().saturating_sub(1) as i32;
                self.menu = (self.menu as i32 + delta).clamp(0, last) as usize;
            }
            View::Fractal => self.fractal.pan(0.0, delta as f64),
            View::Document(document) => {
                let max = ui::document_max_scroll(document, self.width, self.height);
                self.document_scroll =
                    (self.document_scroll as i64 + delta as i64).clamp(0, max as i64) as usize;
            }
            View::Crash => {}
        }
    }

    /// Scroll without keeping a navigation entry focused.
    pub fn scroll(&mut self, delta: i32) {
        self.focused_nav = None;
        self.navigate(delta);
    }
}

/// A per-session seed for the visuals.
fn seed() -> u64 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_nanos() as u64)
            .unwrap_or(41)
    }
    #[cfg(target_arch = "wasm32")]
    {
        (js_sys::Date::now() as u64) ^ ((js_sys::Math::random() * u32::MAX as f64) as u64)
    }
}

/// Wall-clock milliseconds, used to detect repeated logo activations.
fn now_ms() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Date::now() as u64
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }
}
