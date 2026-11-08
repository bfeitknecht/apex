//! Shared navigation and input state.
use crate::lichen::Lichen;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Portfolio(usize),
    Study,
    Unicode,
    Lichen,
    Fractal,
    Document(Document),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Document {
    Age,
    Pgp,
    Me,
}
impl Document {
    pub fn title(self) -> &'static str {
        match self {
            Self::Age => "age.txt",
            Self::Pgp => "pgp.txt",
            Self::Me => "me",
        }
    }
    pub fn text(self) -> &'static str {
        match self {
            Self::Age => crate::portfolio::AGE_TEXT,
            Self::Pgp => crate::portfolio::PGP_TEXT,
            Self::Me => crate::portfolio::ME_ART,
        }
    }
}
#[cfg(target_arch = "wasm32")]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FontStatus {
    #[default]
    NotRequested,
    Loading,
    Ready,
    Failed,
}

pub struct App {
    pub lichen: Lichen,
    /// Display pixelation level; the underlying simulation is unchanged.
    pub lichen_pixelation: u8,
    /// Milliseconds between simulation generations (independent of display pixelation).
    pub lichen_interval_ms: u64,
    lichen_elapsed_ms: u64,
    pub view: View,
    pub previous: usize,
    pub scroll_offset: usize,
    pub document_scroll: usize,
    pub focused_link: Option<usize>,
    pub hovered_link: Option<usize>,
    pub hovered_nav: Option<usize>,
    pub focused_nav: Option<usize>,
    pub study: usize,
    pub fractal_x: f64,
    pub fractal_y: f64,
    pub fractal_scale: f64,
    pub fractal_phase: f64,
    fractal_direction: f64,
    fractal_elapsed_ms: u64,
    logo_activations: u8,
    logo_last_ms: u64,
    #[cfg(not(target_arch = "wasm32"))]
    pub native_glyph: usize,
    #[cfg(target_arch = "wasm32")]
    pub unicode: Option<crate::unicode_study::UnicodeStudy>,
    #[cfg(target_arch = "wasm32")]
    pub unicode_font_status: FontStatus,
    pub paused: bool,
    pub help: bool,
    pub width: usize,
    pub height: usize,
    pending_g: bool,
}
fn seed() -> u64 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(41)
    }
    #[cfg(target_arch = "wasm32")]
    {
        (js_sys::Date::now() as u64) ^ ((js_sys::Math::random() * u32::MAX as f64) as u64)
    }
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
            study: 0,
            fractal_x: 0.0,
            fractal_y: 0.0,
            fractal_scale: 3.2,
            fractal_phase: 0.0,
            fractal_direction: 1.0,
            fractal_elapsed_ms: 0,
            logo_activations: 0,
            logo_last_ms: 0,
            #[cfg(not(target_arch = "wasm32"))]
            native_glyph: 0,
            #[cfg(target_arch = "wasm32")]
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
    pub fn resize(&mut self, w: u16, h: u16) {
        self.width = w as usize;
        self.height = h as usize;
        if matches!(self.view, View::Portfolio(_)) {
            self.set_scroll(self.scroll_offset);
        } else if let View::Document(document) = self.view {
            self.document_scroll = self.document_scroll.min(crate::ui::document_max_scroll(
                document,
                self.width,
                self.height,
            ));
        }
    }
    /// Advance by elapsed time, allowing multiple generations per display frame.
    /// Limit catch-up work after a suspended tab or stalled terminal.
    pub fn advance_lichen(&mut self, elapsed_ms: u64) -> usize {
        if self.view != View::Lichen || self.paused {
            self.lichen_elapsed_ms = 0;
            return 0;
        }
        const MAX_CATCH_UP: u64 = 8;
        self.lichen_elapsed_ms = self
            .lichen_elapsed_ms
            .saturating_add(elapsed_ms)
            .min(self.lichen_interval_ms * MAX_CATCH_UP);
        let steps = self.lichen_elapsed_ms / self.lichen_interval_ms;
        self.lichen_elapsed_ms %= self.lichen_interval_ms;
        for _ in 0..steps {
            self.lichen.step();
        }
        steps as usize
    }
    /// Continuously rotate a 2D slice through quaternion space while zooming.
    pub fn advance_fractal(&mut self, elapsed_ms: u64) -> bool {
        if self.view != View::Fractal || self.help {
            self.fractal_elapsed_ms = 0;
            return false;
        }
        self.fractal_elapsed_ms = (self.fractal_elapsed_ms + elapsed_ms.min(200)).min(250);
        if self.fractal_elapsed_ms < 50 {
            return false;
        }
        let dt = self.fractal_elapsed_ms as f64 / 1000.0;
        self.fractal_elapsed_ms = 0;
        self.fractal_phase += dt * 0.28;
        self.fractal_scale =
            (self.fractal_scale * (-self.fractal_direction * 0.035 * dt).exp()).clamp(0.4, 3.2);
        if self.fractal_scale <= 0.4 {
            self.fractal_direction = -1.0;
        }
        if self.fractal_scale >= 3.2 {
            self.fractal_direction = 1.0;
        }
        true
    }
    pub fn nav_index(&self) -> usize {
        match self.view {
            View::Portfolio(i) => i,
            View::Study | View::Lichen | View::Unicode | View::Fractal => 3,
            View::Document(_) => {
                crate::ui::portfolio_page(self.width, self.height).active(self.scroll_offset)
            }
        }
    }
    fn set_scroll(&mut self, requested: usize) {
        let page = crate::ui::portfolio_page(self.width, self.height);
        self.scroll_offset = requested.min(page.max_scroll());
        self.view = View::Portfolio(page.active(self.scroll_offset));
    }
    pub fn select_section(&mut self, n: usize) {
        self.help = false;
        self.pending_g = false;
        self.focused_link = None;
        self.focused_nav = None;
        self.hovered_link = None;
        if n >= 3 {
            if let View::Portfolio(i) = self.view {
                self.previous = i;
            }
            self.view = View::Study;
            self.study = 0;
        } else {
            let page = crate::ui::portfolio_page(self.width, self.height);
            self.set_scroll(page.offsets[n]);
        }
    }
    pub fn focus_link(&mut self, reverse: bool) {
        self.clear_hover();
        if self.help {
            return;
        }
        if self.view == View::Study {
            self.study = if reverse {
                (self.study + 2) % 3
            } else {
                (self.study + 1) % 3
            };
            return;
        }
        if !matches!(self.view, View::Portfolio(_)) {
            return;
        }
        let page = crate::ui::portfolio_page(self.width, self.height);
        if page.links.is_empty() {
            return;
        }
        if let Some(nav) = self.focused_nav {
            if nav == 0 {
                if reverse {
                    self.focused_nav = None;
                    self.focused_link = Some(page.links.len() - 1);
                } else {
                    self.focused_nav = Some(3);
                    return;
                }
            } else if (!reverse && nav < 5) || (reverse && nav > 3) {
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
            self.focused_nav = Some(if reverse { 5 } else { 0 });
            return;
        } else if let Some(i) = self.focused_link {
            self.focused_link = Some(if reverse { i - 1 } else { i + 1 });
        }
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
    pub fn change_section(&mut self, d: i32) {
        self.clear_hover();
        if self.view == View::Fractal {
            self.fractal_x += d as f64 * self.fractal_scale * 0.12;
            return;
        }
        if self.view == View::Study {
            self.study = (self.study as i32 + d).clamp(0, 2) as usize;
            return;
        }
        let next = (self.nav_index() as i32 + d).clamp(0, 2) as usize;
        self.select_section(next);
    }
    pub fn back(&mut self) {
        if self.help {
            self.help = false;
        } else if matches!(self.view, View::Lichen | View::Unicode | View::Fractal) {
            self.view = View::Study;
        } else if self.view == View::Study {
            self.view = View::Portfolio(self.previous);
        } else if matches!(self.view, View::Document(_)) {
            self.set_scroll(self.scroll_offset);
        }
    }
    pub fn is_unicode(&self) -> bool {
        self.view == View::Unicode
    }
    pub fn navigate(&mut self, d: i32) {
        if self.help {
            return;
        }
        match self.view {
            View::Lichen => {
                let (w, h) = self.lichen.size();
                let max_level = w.max(h).max(1).next_power_of_two().trailing_zeros() as i32;
                self.lichen_pixelation =
                    (self.lichen_pixelation as i32 + d).clamp(0, max_level) as u8;
            }
            #[cfg(target_arch = "wasm32")]
            View::Unicode => {
                if let Some(study) = self.unicode.as_mut() {
                    for _ in 0..d.unsigned_abs().min(8) {
                        if d > 0 {
                            study.next()
                        } else {
                            study.previous()
                        }
                    }
                }
            }
            #[cfg(not(target_arch = "wasm32"))]
            View::Unicode => {
                self.native_glyph = (self.native_glyph as i32 + d).rem_euclid(8) as usize;
            }
            View::Portfolio(_) => {
                let page = crate::ui::portfolio_page(self.width, self.height);
                let next = (self.scroll_offset as i64 + d as i64).clamp(0, page.max_scroll() as i64)
                    as usize;
                self.focused_link = None;
                self.hovered_link = None;
                self.set_scroll(next);
            }
            View::Study => self.study = (self.study as i32 + d).clamp(0, 2) as usize,
            View::Fractal => self.fractal_y += d as f64 * self.fractal_scale * 0.08,
            View::Document(document) => {
                let max = crate::ui::document_max_scroll(document, self.width, self.height);
                self.document_scroll =
                    (self.document_scroll as i64 + d as i64).clamp(0, max as i64) as usize;
            }
        }
    }
    pub fn scroll(&mut self, d: i32) {
        self.focused_nav = None;
        self.navigate(d);
    }
    pub fn key(&mut self, c: char) {
        self.logo_activations = 0;
        self.clear_hover();
        if c != 'n' && c != 'N' {
            self.focused_nav = None;
        }
        if c == '?' {
            self.help = !self.help;
            self.pending_g = false;
            return;
        }
        if self.help {
            return;
        }
        match c {
            '1'..='3' => self.select_section(c as usize - '1' as usize),
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
                    } else if self.view == View::Study {
                        self.study = 0;
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
                    let max = crate::ui::portfolio_page(self.width, self.height).max_scroll();
                    self.focused_link = None;
                    self.set_scroll(max);
                } else if self.view == View::Study {
                    self.study = 2;
                } else if let View::Document(document) = self.view {
                    self.document_scroll =
                        crate::ui::document_max_scroll(document, self.width, self.height);
                }
            }
            '+' | '=' if self.view == View::Fractal => {
                self.fractal_scale = (self.fractal_scale / 1.5).max(0.4)
            }
            '-' if self.view == View::Fractal => {
                self.fractal_scale = (self.fractal_scale * 1.5).min(3.2)
            }
            'r' | '0' if self.view == View::Fractal => {
                self.fractal_x = 0.0;
                self.fractal_y = 0.0;
                self.fractal_scale = 3.2;
                self.fractal_phase = 0.0;
                self.fractal_direction = 1.0;
                self.fractal_elapsed_ms = 0;
            }
            '-' if self.view == View::Lichen => {
                self.lichen_interval_ms = (self.lichen_interval_ms * 2).min(1056);
            }
            '+' | '=' if self.view == View::Lichen => {
                self.lichen_interval_ms = (self.lichen_interval_ms / 2).max(4);
            }
            '0' if self.view == View::Lichen => self.lichen_interval_ms = 66,
            ' ' if self.view == View::Lichen => self.paused = !self.paused,
            'r' if self.view == View::Lichen => self.lichen.reset(seed()),
            #[cfg(target_arch = "wasm32")]
            'r' if self.view == View::Unicode => {
                if let Some(study) = self.unicode.as_mut() {
                    study.shuffle();
                }
            }
            _ => {}
        }
        self.pending_g = false;
    }
    /// Activate the selected study or return a portfolio link.
    pub fn activate(&mut self) -> Option<&'static str> {
        #[cfg(target_arch = "wasm32")]
        if !self.help && self.view == View::Study && self.study == 0 {
            if self.unicode.is_none() {
                self.unicode = Some(crate::unicode_study::UnicodeStudy::new());
            }
            self.view = View::Unicode;
            return None;
        }
        #[cfg(not(target_arch = "wasm32"))]
        if !self.help && self.view == View::Study && self.study == 0 {
            self.view = View::Unicode;
            return None;
        }
        if !self.help && self.view == View::Study && self.study == 1 {
            self.view = View::Lichen;
            return None;
        }
        if let Some(i) = self.focused_nav {
            if i == 0 {
                self.activate_logo();
                return None;
            }
            return crate::portfolio::EXTERNAL_NAV.get(i - 3).copied();
        }
        if !self.help && self.view == View::Study && self.study == 2 {
            self.view = View::Fractal;
            return None;
        }
        self.focused_link.and_then(|i| self.activate_link(i))
    }
    fn activate_logo(&mut self) {
        #[cfg(target_arch = "wasm32")]
        let now = js_sys::Date::now() as u64;
        #[cfg(not(target_arch = "wasm32"))]
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
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
        let page = crate::ui::portfolio_page(self.width, self.height);
        let link = page.links.get(index)?;
        #[cfg(not(target_arch = "wasm32"))]
        {
            use crate::ui::LinkKind;
            let document = match link.kind {
                LinkKind::Age => Some(Document::Age),
                LinkKind::Pgp => Some(Document::Pgp),
                LinkKind::Me => Some(Document::Me),
                LinkKind::External => None,
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
    /// Update hover styling using the same geometry as click hit-testing.
    pub fn hover(&mut self, x: usize, y: usize) -> bool {
        self.hovered_nav = None;
        self.hovered_link = None;
        if self.help || !matches!(self.view, View::Portfolio(_) | View::Study) {
            return false;
        }
        if self.view == View::Study {
            self.hovered_link = crate::ui::study_ranges(self.width, self.height)
                .iter()
                .position(|&(sx, sy, len)| y == sy && x >= sx && x < sx + len);
            return self.hovered_link.is_some();
        }
        if y == 1 && (2..4).contains(&x) {
            return true;
        }
        if y <= 2 {
            self.hovered_nav = crate::ui::nav_ranges(self.width)
                .iter()
                .position(|&(start, end)| x >= start && x < end);
        } else if y >= 4 && matches!(self.view, View::Portfolio(_)) {
            let page = crate::ui::portfolio_page(self.width, self.height);
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
    /// Return a URL when an item is clicked; callers decide how to open it.
    pub fn click(&mut self, x: usize, y: usize) -> Option<&'static str> {
        if !(y == 1 && (2..4).contains(&x) && matches!(self.view, View::Portfolio(_))) {
            self.logo_activations = 0;
        }
        if self.help {
            self.help = false;
            return None;
        }
        if self.view == View::Lichen || self.is_unicode() {
            self.back();
            return None;
        }
        if matches!(self.view, View::Portfolio(_)) && y == 1 && (2..4).contains(&x) {
            self.activate_logo();
            return None;
        }
        if self.view == View::Study {
            if let Some(i) = crate::ui::study_ranges(self.width, self.height)
                .iter()
                .position(|&(sx, sy, len)| y == sy && x >= sx && x < sx + len)
            {
                self.study = i;
                return self.activate();
            }
            return None;
        }
        if y <= 2 {
            for (i, (start, end)) in crate::ui::nav_ranges(self.width).iter().enumerate() {
                if x >= *start && x < *end {
                    if i >= 3 {
                        self.focused_link = None;
                        self.focused_nav = Some(i);
                        return crate::portfolio::EXTERNAL_NAV.get(i - 3).copied();
                    }
                    self.select_section(i);
                    return None;
                }
            }
        }
        if matches!(self.view, View::Portfolio(_)) && y >= 4 {
            let page = crate::ui::portfolio_page(self.width, self.height);
            let row = self.scroll_offset + y - 4;
            if row >= page.footer_start && self.scroll_offset != page.max_scroll() {
                return None;
            }
            for (i, link) in page.links.iter().enumerate() {
                if link.row == row && x >= 8 + link.start && x < 8 + link.end {
                    self.focused_nav = None;
                    self.focused_link = Some(i);
                    return self.activate_link(i);
                }
            }
        }
        None
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scrolling_sections_and_footer() {
        for (w, h) in [(120, 40), (40, 40), (32, 20)] {
            let mut a = App::new();
            a.resize(w, h);
            let page = crate::ui::portfolio_page(w as usize, h as usize);
            assert!(page.max_scroll() > 0);
            for index in 0..3 {
                a.select_section(index);
                assert_eq!(a.scroll_offset, page.offsets[index]);
                assert_eq!(a.nav_index(), index);
            }
            a.key('G');
            assert_eq!(a.scroll_offset, page.max_scroll());
            a.key('g');
            a.key('g');
            assert_eq!(a.scroll_offset, 0);
            a.navigate(2);
            assert_eq!(a.scroll_offset, 2);
        }
    }
    #[test]
    fn line_keys_and_hover_hit_testing() {
        let mut a = App::new();
        a.resize(80, 24);
        a.key('j');
        assert_eq!(a.scroll_offset, 1);
        a.key('k');
        assert_eq!(a.scroll_offset, 0);
        let nav_x = crate::ui::nav_ranges(80)[1].0;
        assert!(a.hover(nav_x, 1));
        assert_eq!(a.hovered_nav, Some(1));
        a.click(nav_x, 1);
        assert_eq!(a.nav_index(), 1);
        let page = crate::ui::portfolio_page(80, 24);
        let link = &page.links[0];
        let (x, y) = (8 + link.start, 4 + link.row - a.scroll_offset);
        assert!(a.hover(x, y));
        assert_eq!(a.hovered_link, Some(0));
        a.click(x, y);
        assert_eq!(a.focused_link, Some(0));
        a.clear_hover();
        assert_eq!(a.hovered_link, None);
        assert!(!a.hover(2, 3));
        a.key('G');
        assert!(a.scroll_offset > 0);
        assert!(a.hover(2, 1));
        a.click(2, 1);
        assert_eq!(a.scroll_offset, 0);
    }
    #[test]
    fn mouse_navigation_and_links() {
        let mut a = App::new();
        a.resize(80, 24);
        let x = crate::ui::nav_ranges(80)[2].0;
        a.click(x, 1);
        assert_eq!(a.nav_index(), 2);
        a.focus_link(false);
        assert_eq!(a.focused_nav, Some(0));
    }
    #[test]
    fn external_navigation_is_clickable_and_keyboard_focusable() {
        let mut a = App::new();
        a.resize(100, 24);
        for (i, expected) in crate::portfolio::EXTERNAL_NAV.iter().enumerate() {
            let x = crate::ui::nav_ranges(100)[i + 3].0;
            assert_eq!(a.click(x, 1), Some(*expected));
        }
        a.select_section(0);
        a.focus_link(false); // logo
        for expected in crate::portfolio::EXTERNAL_NAV {
            a.focus_link(false);
            assert_eq!(a.activate(), Some(expected));
        }
        a.focus_link(false);
        assert!(a.focused_link.is_some());
    }
    #[test]
    fn visuals_navigation_and_study_activation() {
        let mut a = App::new();
        a.resize(80, 24);
        for _ in 0..3 {
            a.click(2, 1);
        }
        assert_eq!(a.view, View::Study);
        let (x, y, _) = crate::ui::study_ranges(80, 24)[1];
        assert!(a.hover(x, y));
        a.click(x, y);
        assert_eq!(a.view, View::Lichen);
        a.back();
        assert_eq!(a.view, View::Study);
        a.study = 0;
        a.activate();
        assert_eq!(a.view, View::Unicode);
        a.navigate(1);
        a.back();
        assert_eq!(a.view, View::Study);
        a.back();
        assert_eq!(a.view, View::Portfolio(0));
    }
    #[test]
    fn logo_keyboard_gesture_and_fractal_controls() {
        let mut a = App::new();
        a.focus_link(false);
        assert_eq!(a.focused_nav, Some(0));
        for _ in 0..3 {
            a.activate();
        }
        assert_eq!(a.view, View::Study);
        a.study = 2;
        a.activate();
        assert_eq!(a.view, View::Fractal);
        let (x, y, scale) = (a.fractal_x, a.fractal_y, a.fractal_scale);
        a.key('l');
        a.key('j');
        a.key('+');
        assert!(a.fractal_x > x && a.fractal_y > y && a.fractal_scale < scale);
        a.key('r');
        assert_eq!((a.fractal_x, a.fractal_y, a.fractal_scale), (x, y, scale));
        assert!(a.advance_fractal(100));
        assert!(a.fractal_scale < scale && a.fractal_phase > 0.0);
        a.back();
        assert_eq!(a.view, View::Study);
        assert!(!a.advance_fractal(100));
    }
    #[test]
    fn footer_documents_open_inside_the_terminal_and_return() {
        use crate::ui::LinkKind;
        for (width, height) in [(100, 24), (32, 16)] {
            let mut a = App::new();
            a.resize(width, height);
            a.key('G');
            let original_scroll = a.scroll_offset;
            let page = crate::ui::portfolio_page(width as usize, height as usize);
            for (kind, document) in [
                (LinkKind::Age, Document::Age),
                (LinkKind::Pgp, Document::Pgp),
                (LinkKind::Me, Document::Me),
            ] {
                let index = page
                    .links
                    .iter()
                    .position(|link| link.kind == kind)
                    .unwrap();
                let link = &page.links[index];
                let y = 4 + link.row - original_scroll;
                assert!(a.hover(8 + link.start, y));
                assert_eq!(a.click(8 + link.start, y), None);
                assert_eq!(a.view, View::Document(document));
                a.key('G');
                assert_eq!(
                    a.document_scroll,
                    crate::ui::document_max_scroll(document, width as usize, height as usize)
                );
                a.back();
                assert_eq!(a.scroll_offset, original_scroll);
                assert!(matches!(a.view, View::Portfolio(_)));
                a.focused_link = Some(index);
                assert_eq!(a.activate(), None);
                assert_eq!(a.view, View::Document(document));
                a.back();
            }
        }
    }
    #[test]
    fn studies_remain_available_internally() {
        let mut a = App::new();
        a.view = View::Study;
        a.study = 1;
        a.activate();
        assert_eq!(a.view, View::Lichen);
        for _ in 0..20 {
            a.navigate(1);
        }
        let (w, h) = a.lichen.size();
        assert_eq!(
            a.lichen_pixelation,
            w.max(h).next_power_of_two().trailing_zeros() as u8
        );
        a.key('-');
        assert_eq!(a.lichen_interval_ms, 132);
        a.key('+');
        assert_eq!(a.lichen_interval_ms, 66);
        a.key('+');
        assert!(a.lichen_interval_ms < 66);
        a.key('+');
        a.key('+');
        a.key('+');
        assert_eq!(a.lichen_interval_ms, 4);
        assert_eq!(a.advance_lichen(16), 4);
        a.paused = true;
        assert_eq!(a.advance_lichen(100), 0);
        a.paused = false;
        assert_eq!(a.advance_lichen(1000), 8);
        a.key('0');
        assert_eq!(a.lichen_interval_ms, 66);
        a.back();
        assert_eq!(a.view, View::Study);
    }
}
