//! Navigation, input and animation tests for the shared application state.

use super::*;
use crate::{content, ui};

#[test]
fn scrolling_sections_and_footer() {
    for (width, height) in [(120, 40), (40, 40), (32, 20)] {
        let mut app = App::new();
        app.resize(width, height);
        let page = ui::portfolio_page(width as usize, height as usize);
        assert!(page.max_scroll() > 0);
        for index in 0..3 {
            app.select_section(index);
            assert_eq!(app.scroll_offset, page.offsets[index]);
            assert_eq!(app.nav_index(), index);
        }
        app.key('G');
        assert_eq!(app.scroll_offset, page.max_scroll());
        app.key('g');
        app.key('g');
        assert_eq!(app.scroll_offset, 0);
        app.navigate(2);
        assert_eq!(app.scroll_offset, 2);
    }
}

#[test]
fn line_keys_and_hover_hit_testing() {
    let mut app = App::new();
    app.resize(80, 24);
    app.key('j');
    assert_eq!(app.scroll_offset, 1);
    app.key('k');
    assert_eq!(app.scroll_offset, 0);
    let nav_x = ui::nav_ranges(80)[1].0;
    assert!(app.hover(nav_x, 1));
    assert_eq!(app.hovered_nav, Some(1));
    app.click(nav_x, 1);
    assert_eq!(app.nav_index(), 1);
    let page = ui::portfolio_page(80, 24);
    let link = &page.links[0];
    let (x, y) = (8 + link.start, 4 + link.row - app.scroll_offset);
    assert!(app.hover(x, y));
    assert_eq!(app.hovered_link, Some(0));
    app.click(x, y);
    assert_eq!(app.focused_link, Some(0));
    app.clear_hover();
    assert_eq!(app.hovered_link, None);
    assert!(!app.hover(2, 3));
    app.key('G');
    assert!(app.scroll_offset > 0);
    assert!(app.hover(2, 1));
    app.click(2, 1);
    assert_eq!(app.scroll_offset, 0);
}

#[test]
fn mouse_navigation_and_links() {
    let mut app = App::new();
    app.resize(80, 24);
    let x = ui::nav_ranges(80)[2].0;
    app.click(x, 1);
    assert_eq!(app.nav_index(), 2);
    app.focus_link(false);
    assert_eq!(app.focused_nav, Some(0));
}

#[test]
fn external_navigation_is_clickable_and_keyboard_focusable() {
    let mut app = App::new();
    app.resize(100, 24);
    for (index, link) in content::profile().links.iter().enumerate() {
        let x = ui::nav_ranges(100)[index + 3].0;
        assert_eq!(app.click(x, 1), Some(link.url));
    }
    app.select_section(0);
    app.focus_link(false); // logo
    for link in content::profile().links {
        app.focus_link(false);
        assert_eq!(app.activate(), Some(link.url));
    }
    app.focus_link(false);
    assert!(app.focused_link.is_some());
}

#[test]
fn menu_opens_and_cycles_every_visual() {
    let mut app = App::new();
    app.resize(80, 24);
    for _ in 0..3 {
        app.click(2, 1);
    }
    assert_eq!(app.view, View::Visuals);
    let (x, y, _) = ui::visual_ranges(80, 24)[1];
    assert!(app.hover(x, y));
    app.click(x, y);
    assert_eq!(app.view, View::Lichen);
    app.back();
    assert_eq!(app.view, View::Visuals);
    app.menu = 0;
    app.activate();
    assert_eq!(app.view, View::Unicode);
    app.navigate(1);
    app.back();
    assert_eq!(app.view, View::Visuals);
    app.back();
    assert_eq!(app.view, View::Portfolio(0));
}

#[test]
fn crash_is_a_menu_entry_like_the_others() {
    let mut app = App::new();
    app.resize(80, 24);
    app.select_section(3);
    assert_eq!(app.view, View::Visuals);
    let (x, y, _) = *ui::visual_ranges(80, 24).last().expect("menu entry");
    assert!(app.hover(x, y));
    app.click(x, y);
    assert_eq!(app.view, View::Crash);

    let blank = app.crash.cell(0, 0);
    assert!(app.tick(64), "the sweep advances");
    let painted = (0..80).any(|x| (0..25).any(|y| app.crash.cell(x, y) != blank));
    assert!(painted, "the sweep writes into video memory");

    app.key('r');
    assert_eq!(app.crash.cell(0, 0), blank, "reset starts over");
    app.back();
    assert_eq!(app.view, View::Visuals);
}

#[test]
fn every_visual_is_reachable_from_the_menu() {
    for (index, visual) in crate::visuals::VISUALS.iter().enumerate() {
        let mut app = App::new();
        app.resize(80, 24);
        app.select_section(3);
        app.menu = index;
        app.activate();
        assert!(app.view.is_visual(), "{visual:?} did not open");
        app.back();
        assert_eq!(app.view, View::Visuals);
    }
}

#[test]
fn logo_keyboard_gesture_and_fractal_controls() {
    let mut app = App::new();
    app.focus_link(false);
    assert_eq!(app.focused_nav, Some(0));
    for _ in 0..3 {
        app.activate();
    }
    assert_eq!(app.view, View::Visuals);
    app.menu = 2;
    app.activate();
    assert_eq!(app.view, View::Fractal);
    let (x, y, scale) = (app.fractal.x(), app.fractal.y(), app.fractal.scale());
    app.key('l');
    app.key('j');
    app.key('+');
    assert!(app.fractal.x() > x && app.fractal.y() > y && app.fractal.scale() < scale);
    app.key('r');
    assert_eq!(
        (app.fractal.x(), app.fractal.y(), app.fractal.scale()),
        (x, y, scale)
    );
    assert!(app.tick(100));
    assert!(app.fractal.scale() < scale && app.fractal.phase() > 0.0);
    app.back();
    assert_eq!(app.view, View::Visuals);
    assert!(!app.tick(100));
}

#[test]
fn footer_documents_open_inside_the_terminal_and_return() {
    #[cfg(target_arch = "wasm32")]
    return;

    #[cfg(not(target_arch = "wasm32"))]
    for (width, height) in [(100, 24), (32, 16)] {
        let mut app = App::new();
        app.resize(width, height);
        app.key('G');
        let original_scroll = app.scroll_offset;
        let page = ui::portfolio_page(width as usize, height as usize);
        for (kind, document) in [
            (ui::LinkKind::Age, Document::Age),
            (ui::LinkKind::Pgp, Document::Pgp),
            (ui::LinkKind::Me, Document::Me),
        ] {
            let index = page
                .links
                .iter()
                .position(|link| link.kind == kind)
                .unwrap();
            let link = &page.links[index];
            let y = 4 + link.row - original_scroll;
            assert!(app.hover(8 + link.start, y));
            assert_eq!(app.click(8 + link.start, y), None);
            assert_eq!(app.view, View::Document(document));
            app.key('G');
            assert_eq!(
                app.document_scroll,
                ui::document_max_scroll(document, width as usize, height as usize)
            );
            app.back();
            assert_eq!(app.scroll_offset, original_scroll);
            assert!(matches!(app.view, View::Portfolio(_)));
            app.focused_link = Some(index);
            assert_eq!(app.activate(), None);
            assert_eq!(app.view, View::Document(document));
            app.back();
        }
    }
}

#[test]
fn lichen_controls_work_inside_a_session() {
    let mut app = App::new();
    app.view = View::Visuals;
    app.menu = 1;
    app.activate();
    assert_eq!(app.view, View::Lichen);
    for _ in 0..20 {
        app.navigate(1);
    }
    let (width, height) = app.lichen.size();
    assert_eq!(
        app.lichen_pixelation,
        width.max(height).next_power_of_two().trailing_zeros() as u8
    );
    app.key('-');
    assert_eq!(app.lichen_interval_ms, 132);
    app.key('+');
    assert_eq!(app.lichen_interval_ms, 66);
    app.key('+');
    assert!(app.lichen_interval_ms < 66);
    app.key('+');
    app.key('+');
    app.key('+');
    assert_eq!(app.lichen_interval_ms, 4);
    assert!(app.tick(16), "four generations at a 4 ms interval");
    app.paused = true;
    assert!(!app.tick(100), "paused colonies do not advance");
    app.paused = false;
    assert!(app.tick(1000), "catch-up is capped, not skipped");
    app.key('0');
    assert_eq!(app.lichen_interval_ms, 66);
    app.back();
    assert_eq!(app.view, View::Visuals);
}

#[test]
fn unicode_navigates_only_once_opened() {
    let mut app = App::new();
    app.resize(80, 24);
    app.view = View::Unicode;
    app.navigate(3);
    assert!(app.unicode.is_none(), "the catalogue is built on demand");
    app.view = View::Visuals;
    app.menu = 0;
    app.activate();
    assert_eq!(app.view, View::Unicode);
    assert!(app.unicode.is_some());
    app.key('r');
    app.navigate(-2);
}
