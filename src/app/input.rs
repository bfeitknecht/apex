//! Input handling: character keys, pointer hit testing and link activation.
//!
//! Keys that do not depend on the surface arrive here as characters. Each
//! frontend decodes its own device events first: the terminal maps crossterm
//! key codes, SSH parses the byte stream, the browser reads DOM events.

use super::{now_ms, seed, App, View};
use crate::visuals::{Visual, VISUALS};
use crate::{content, ui};

impl App {
    /// Apply a single character key.
    pub fn key(&mut self, key: char) {
        self.logo_activations = 0;
        self.clear_hover();
        if key != 'n' && key != 'N' {
            self.focused_nav = None;
        }
        if key == '?' {
            self.help = !self.help;
            self.pending_g = false;
            return;
        }
        if self.help {
            return;
        }
        match key {
            '1'..='3' => self.select_section((key as u8 - b'1') as usize),
            'h' => self.change_section(-1),
            'l' => self.change_section(1),
            'j' => self.navigate(1),
            'k' => self.navigate(-1),
            'n' => self.focus_link(false),
            'N' => self.focus_link(true),
            'g' => {
                if self.pending_g {
                    if matches!(self.view, View::Portfolio(_)) {
                        self.focused_link = None;
                        self.set_scroll(0);
                    } else if self.view == View::Visuals {
                        self.menu = 0;
                    } else if matches!(self.view, View::Document(_)) {
                        self.document_scroll = 0;
                    }
                    self.pending_g = false;
                } else {
                    self.pending_g = true;
                }
                return;
            }
            'G' => {
                if matches!(self.view, View::Portfolio(_)) {
                    let max = ui::portfolio_page(self.width, self.height).max_scroll();
                    self.focused_link = None;
                    self.set_scroll(max);
                } else if self.view == View::Visuals {
                    self.menu = VISUALS.len() - 1;
                } else if let View::Document(document) = self.view {
                    self.document_scroll =
                        ui::document_max_scroll(document, self.width, self.height);
                }
            }
            '+' | '=' if self.view == View::Fractal => self.fractal.zoom(1.5),
            '-' if self.view == View::Fractal => self.fractal.zoom(1.0 / 1.5),
            'r' | '0' if self.view == View::Fractal => self.fractal.reset(),
            'r' | '0' if self.view == View::Crash => self.crash.reset(),
            '-' if self.view == View::Lichen => {
                self.lichen_interval_ms = (self.lichen_interval_ms * 2).min(1056);
            }
            '+' | '=' if self.view == View::Lichen => {
                self.lichen_interval_ms = (self.lichen_interval_ms / 2).max(4);
            }
            '0' if self.view == View::Lichen => self.lichen_interval_ms = 66,
            ' ' if self.view == View::Lichen => self.paused = !self.paused,
            'r' if self.view == View::Lichen => self.lichen.reset(seed()),
            'r' if self.view == View::Unicode => {
                if let Some(unicode) = self.unicode.as_mut() {
                    unicode.shuffle();
                }
            }
            _ => {}
        }
        self.pending_g = false;
    }

    /// Activate the focused item: open a visual or a document, or report a URL.
    ///
    /// A returned URL belongs to a browser tab; terminals display it instead, and
    /// footer documents open inside the terminal in both cases.
    pub fn activate(&mut self) -> Option<&'static str> {
        if !self.help && self.view == View::Visuals {
            let visual = VISUALS[self.menu.min(VISUALS.len() - 1)];
            self.open(visual);
            return None;
        }
        if let Some(index) = self.focused_nav {
            if index == 0 {
                self.activate_logo();
                return None;
            }
            return index
                .checked_sub(3)
                .and_then(|slot| content::profile().links.get(slot))
                .map(|link| link.url);
        }
        self.focused_link
            .and_then(|index| self.activate_link(index))
    }

    /// Open the visual the hidden menu is pointing at.
    pub fn open(&mut self, visual: Visual) {
        self.help = false;
        self.clear_hover();
        self.view = match visual {
            Visual::Unicode => {
                if self.unicode.is_none() {
                    self.unicode = Some(crate::visuals::Unicode::new(seed()));
                }
                View::Unicode
            }
            Visual::Lichen => View::Lichen,
            Visual::Fractal => View::Fractal,
            Visual::Crash => View::Crash,
        };
    }

    /// The logo opens the hidden menu on the third activation within 1.2s.
    fn activate_logo(&mut self) {
        let now = now_ms();
        self.logo_activations = if now.saturating_sub(self.logo_last_ms) <= 1200 {
            self.logo_activations + 1
        } else {
            1
        };
        self.logo_last_ms = now;
        if self.logo_activations >= 3 {
            self.logo_activations = 0;
            self.select_section(3);
        } else {
            self.select_section(0);
            self.focused_nav = Some(0);
        }
    }

    fn activate_link(&mut self, index: usize) -> Option<&'static str> {
        if self.help || !matches!(self.view, View::Portfolio(_)) {
            return None;
        }
        let page = ui::portfolio_page(self.width, self.height);
        let link = page.links.get(index)?;
        #[cfg(not(target_arch = "wasm32"))]
        {
            let document = match link.kind {
                ui::LinkKind::Age => Some(super::Document::Age),
                ui::LinkKind::Pgp => Some(super::Document::Pgp),
                ui::LinkKind::Me => Some(super::Document::Me),
                ui::LinkKind::External => None,
            };
            if let Some(document) = document {
                self.document_scroll = 0;
                self.view = View::Document(document);
                self.clear_hover();
                return None;
            }
        }
        (!link.url.is_empty()).then_some(link.url)
    }

    /// Update hover styling using the same geometry as click hit testing.
    pub fn hover(&mut self, x: usize, y: usize) -> bool {
        self.hovered_nav = None;
        self.hovered_link = None;
        if self.help || !matches!(self.view, View::Portfolio(_) | View::Visuals) {
            return false;
        }
        if self.view == View::Visuals {
            self.hovered_link = ui::visual_ranges(self.width, self.height)
                .iter()
                .position(|&(start, row, len)| y == row && x >= start && x < start + len);
            return self.hovered_link.is_some();
        }
        if y == 1 && (2..4).contains(&x) {
            return true;
        }
        if y <= 2 {
            self.hovered_nav = ui::nav_ranges(self.width)
                .iter()
                .position(|&(start, end)| x >= start && x < end);
        } else if y >= 4 {
            let page = ui::portfolio_page(self.width, self.height);
            let row = self.scroll_offset + y - 4;
            if row < page.footer_start || self.scroll_offset == page.max_scroll() {
                self.hovered_link = page
                    .links
                    .iter()
                    .position(|link| link.row == row && x >= 8 + link.start && x < 8 + link.end);
            }
        }
        self.hovered_nav.is_some() || self.hovered_link.is_some()
    }

    pub fn clear_hover(&mut self) {
        self.hovered_nav = None;
        self.hovered_link = None;
    }

    /// Handle a pointer press. Returns a URL when the target is external.
    pub fn click(&mut self, x: usize, y: usize) -> Option<&'static str> {
        if !(y == 1 && (2..4).contains(&x) && matches!(self.view, View::Portfolio(_))) {
            self.logo_activations = 0;
        }
        if self.help {
            self.help = false;
            return None;
        }
        if self.view.is_visual() {
            self.back();
            return None;
        }
        if matches!(self.view, View::Portfolio(_)) && y == 1 && (2..4).contains(&x) {
            self.activate_logo();
            return None;
        }
        if self.view == View::Visuals {
            if let Some(index) = ui::visual_ranges(self.width, self.height)
                .iter()
                .position(|&(start, row, len)| y == row && x >= start && x < start + len)
            {
                self.menu = index;
                return self.activate();
            }
            return None;
        }
        if y <= 2 {
            for (index, (start, end)) in ui::nav_ranges(self.width).iter().enumerate() {
                if x >= *start && x < *end {
                    if index >= 3 {
                        self.focused_link = None;
                        self.focused_nav = Some(index);
                        return index
                            .checked_sub(3)
                            .and_then(|slot| content::profile().links.get(slot))
                            .map(|link| link.url);
                    }
                    self.select_section(index);
                    return None;
                }
            }
        }
        if matches!(self.view, View::Portfolio(_)) && y >= 4 {
            let page = ui::portfolio_page(self.width, self.height);
            let row = self.scroll_offset + y - 4;
            if row >= page.footer_start && self.scroll_offset != page.max_scroll() {
                return None;
            }
            for (index, link) in page.links.iter().enumerate() {
                if link.row == row && x >= 8 + link.start && x < 8 + link.end {
                    self.focused_nav = None;
                    self.focused_link = Some(index);
                    return self.activate_link(index);
                }
            }
        }
        None
    }
}
