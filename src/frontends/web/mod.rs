//! Browser frontend: Ratzilla draws the shared UI into WebGL2 (with Canvas 2D
//! and DOM fallbacks), and two extra canvases enlarge the colony and the
//! Unicode specimen beyond character cells.

mod input;
mod surfaces;

use std::cell::RefCell;
use std::io;
use std::rc::Rc;

use ratzilla::ratatui::{backend::Backend, Terminal};
use ratzilla::{CanvasBackend, CellSized, DomBackend, WebEventHandler, WebGl2Backend, WebRenderer};

use crate::app::{App, FontStatus};
use crate::visuals::Unicode;
use crate::{ui, View};

use surfaces::{FontCache, GlyphSurface, LichenSurface, TextMirror, Viewport};

/// The application state shared with the event listeners.
type Shared = Rc<RefCell<App>>;

/// Pixels per character cell stay above this so the grid stays legible.
const MIN_CELL_SIZE: usize = 2;

/// Viewport area in pixels that one cell should cover.
const CELL_AREA: f64 = 500_000.0;

/// Open `url` in the way the visitor expects.
fn open(url: &str) {
    let Some(window) = web_sys::window() else {
        return;
    };
    if url.starts_with("mailto:") {
        let _ = window.open_with_url_and_target(url, "_self");
    } else {
        let _ = window.open_with_url_and_target_and_features(url, "_blank", "noopener,noreferrer");
    }
}

/// Install the panic hook, load the UI font, then start the renderer.
pub fn start() {
    console_error_panic_hook::set_once();
    wasm_bindgen_futures::spawn_local(async {
        if let Some(document) = web_sys::window().and_then(|window| window.document()) {
            let _ =
                wasm_bindgen_futures::JsFuture::from(document.fonts().load("16px 'Commit Mono'"))
                    .await;
        }
        if let Err(error) = initialize() {
            web_sys::console::error_1(&error.to_string().into());
        }
    });
}

/// Prefer WebGL2, then Canvas 2D, then the DOM backend.
fn initialize() -> io::Result<()> {
    use ratzilla::backend::webgl2::{FontAtlasConfig, WebGl2BackendOptions};

    let options = WebGl2BackendOptions::new()
        .disable_auto_css_resize()
        .font_atlas_config(FontAtlasConfig::dynamic(
            &["Commit Mono", "monospace"],
            16.0,
        ));
    if let Ok(backend) = WebGl2Backend::new_with_options(options) {
        return run(backend);
    }
    if let Ok(backend) = CanvasBackend::new() {
        return run(backend);
    }
    run(DomBackend::new()?)
}

fn run<B>(backend: B) -> io::Result<()>
where
    B: Backend<Error = io::Error> + WebEventHandler + CellSized + 'static,
{
    let window = web_sys::window().expect("window");
    let document = window.document().expect("document");

    let state: Shared = Rc::new(RefCell::new(App::new()));
    let mut terminal = Terminal::new(backend)?;
    input::install(&mut terminal, &window, &document, state.clone())?;

    let mut colony_surface = LichenSurface::new(&document);
    let mut glyph_surface = GlyphSurface::new(&document);
    let mut mirror = TextMirror::new(&document);
    let fonts = FontCache::default();
    let mut held_glyph: Option<char> = None;
    let mut last_frame = 0.0;

    terminal.draw_web(move |frame| {
        let mut app = state.borrow_mut();
        let area = frame.area();
        app.resize(area.width, area.height);

        let viewport = Viewport::current();
        let cell_size = ((viewport.width * viewport.height / CELL_AREA).sqrt().ceil() as usize)
            .max(MIN_CELL_SIZE);
        let columns = (viewport.width / cell_size as f64).ceil() as usize;
        let rows = (viewport.height / cell_size as f64).ceil() as usize;

        let now = js_sys::Date::now();
        let elapsed = if last_frame == 0.0 {
            0
        } else {
            (now - last_frame).max(0.0) as u64
        };
        last_frame = now;

        if app.view == View::Lichen {
            app.lichen.resize(columns, rows);
        }
        app.tick(elapsed);

        // The Unicode view may fall back to the hidden menu for one frame when
        // there is no specimen to show yet; the real view is restored before
        // the frame ends so the state never sticks.
        let real_view = app.view;
        let font_status = fonts.status(&app);
        let status = settle_specimen(&mut app, font_status, &mut held_glyph);
        app.unicode_font_status = status;

        ui::draw(frame, &app);

        mirror.set_visible(app.view != View::Lichen || app.help);
        mirror.sync(frame, viewport);

        glyph_surface.set_visible(app.is_unicode() && !app.help);
        if let Some(unicode) = app.unicode.as_ref() {
            glyph_surface.paint(unicode, status, area, viewport);
        }
        if let Some(unicode) = app.unicode.as_mut() {
            unicode.pin_glyph(None);
        }

        app.view = real_view;

        colony_surface.set_visible(app.view == View::Lichen && !app.help);
        if app.view == View::Lichen {
            colony_surface.paint(&app.lichen, app.lichen_pixelation, columns, rows, cell_size);
        }
    });
    Ok(())
}

/// Keep the last loaded specimen on screen while its replacement's font loads.
///
/// Without a previous specimen the browser shows the hidden menu for the frame
/// instead of an empty canvas.
fn settle_specimen(app: &mut App, status: FontStatus, held: &mut Option<char>) -> FontStatus {
    if !app.is_unicode() || app.help {
        return status;
    }
    match status {
        FontStatus::Ready => {
            *held = app.unicode.as_ref().map(Unicode::glyph);
            FontStatus::Ready
        }
        FontStatus::Loading => match *held {
            Some(glyph) => {
                if let Some(unicode) = app.unicode.as_mut() {
                    unicode.pin_glyph(Some(glyph));
                }
                FontStatus::Ready
            }
            None => {
                app.view = View::Visuals;
                FontStatus::Loading
            }
        },
        other => other,
    }
}
