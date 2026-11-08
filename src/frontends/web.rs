use apex::{app::FontStatus, ui, App, View};
use ratzilla::{
    event::MouseEventKind,
    ratatui::{backend::Backend, Terminal},
    CanvasBackend, CellSized, DomBackend, WebEventHandler, WebGl2Backend, WebRenderer,
};
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    io,
    rc::Rc,
};
use wasm_bindgen::{closure::Closure, Clamped, JsCast};
use web_sys::{
    AddEventListenerOptions, CanvasRenderingContext2d, HtmlCanvasElement, ImageData, WheelEvent,
};
fn open(url: &str) {
    if let Some(w) = web_sys::window() {
        if url.starts_with("mailto:") {
            let _ = w.open_with_url_and_target(url, "_self");
        } else {
            let _ = w.open_with_url_and_target_and_features(url, "_blank", "noopener,noreferrer");
        }
    }
}
fn run<B>(backend: B) -> io::Result<()>
where
    B: Backend<Error = io::Error> + WebEventHandler + CellSized + 'static,
{
    let window = web_sys::window().expect("window");
    let document = window.document().expect("document");
    let surface = document
        .get_element_by_id("lichen-pixels")
        .and_then(|e| e.dyn_into::<HtmlCanvasElement>().ok());
    let context = surface
        .as_ref()
        .and_then(|c| c.get_context("2d").ok().flatten())
        .and_then(|v| v.dyn_into::<CanvasRenderingContext2d>().ok());
    let glyph_surface = document
        .get_element_by_id("unicode-glyph")
        .and_then(|e| e.dyn_into::<HtmlCanvasElement>().ok());
    let glyph_context = glyph_surface
        .as_ref()
        .and_then(|c| c.get_context("2d").ok().flatten())
        .and_then(|v| v.dyn_into::<CanvasRenderingContext2d>().ok());
    let font_states = Rc::new(RefCell::new(BTreeMap::<&'static str, FontStatus>::new()));
    let mut glyph_cache: Option<(char, u32, u32, u16, u16, FontStatus)> = None;
    let mut displayed_glyph: Option<char> = None;
    let mut terminal = Terminal::new(backend)?;
    let state = Rc::new(RefCell::new(App::new()));
    // A transparent text mirror keeps WebGL/Canvas rendering while allowing
    // native browser drag-selection and copying of the Ratatui text cells.
    let selection_layer = document
        .get_element_by_id("selectable-text")
        .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok());
    if let Some(layer) = selection_layer.as_ref() {
        let hover_state = state.clone();
        let hover_layer = layer.clone();
        let hover =
            Closure::<dyn FnMut(web_sys::MouseEvent)>::new(move |e: web_sys::MouseEvent| {
                let Some(window) = web_sys::window() else {
                    return;
                };
                let vw = window
                    .inner_width()
                    .ok()
                    .and_then(|v| v.as_f64())
                    .unwrap_or(1.0)
                    .max(1.0);
                let vh = window
                    .inner_height()
                    .ok()
                    .and_then(|v| v.as_f64())
                    .unwrap_or(1.0)
                    .max(1.0);
                let mut app = hover_state.borrow_mut();
                let x = (e.client_x().max(0) as f64 / vw * app.width as f64) as usize;
                let y = (e.client_y().max(0) as f64 / vh * app.height as f64) as usize;
                let active = app.hover(x, y);
                let _ = hover_layer
                    .set_attribute("data-clickable", if active { "true" } else { "false" });
            });
        let _ = layer.add_event_listener_with_callback("mousemove", hover.as_ref().unchecked_ref());
        hover.forget();
        let leave_state = state.clone();
        let leave =
            Closure::<dyn FnMut(web_sys::MouseEvent)>::new(move |_e: web_sys::MouseEvent| {
                leave_state.borrow_mut().clear_hover();
            });
        let _ =
            layer.add_event_listener_with_callback("mouseleave", leave.as_ref().unchecked_ref());
        leave.forget();
        let mouse_start = Rc::new(Cell::new(None::<(i32, i32)>));
        let down_pos = mouse_start.clone();
        let down = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(move |e: web_sys::MouseEvent| {
            down_pos.set(Some((e.client_x(), e.client_y())));
        });
        let _ = layer.add_event_listener_with_callback("mousedown", down.as_ref().unchecked_ref());
        down.forget();
        let selected_state = state.clone();
        let click =
            Closure::<dyn FnMut(web_sys::MouseEvent)>::new(move |e: web_sys::MouseEvent| {
                let Some(window) = web_sys::window() else {
                    return;
                };
                let dragged = mouse_start
                    .take()
                    .map(|(x, y)| (x - e.client_x()).abs() > 4 || (y - e.client_y()).abs() > 4)
                    .unwrap_or(false);
                if dragged {
                    if let Some(doc) = window.document() {
                        if let Ok(Some(el)) = doc
                            .query_selector("canvas:not(#lichen-pixels):not(#unicode-glyph), #grid")
                        {
                            if let Some(html) = el.dyn_ref::<web_sys::HtmlElement>() {
                                let _ = html.focus();
                            }
                        }
                    }
                    return;
                }
                if let Ok(Some(selection)) = window.get_selection() {
                    if !selection.is_collapsed() {
                        let _ = selection.remove_all_ranges();
                    }
                }
                let vw = window
                    .inner_width()
                    .ok()
                    .and_then(|v| v.as_f64())
                    .unwrap_or(1.0)
                    .max(1.0);
                let vh = window
                    .inner_height()
                    .ok()
                    .and_then(|v| v.as_f64())
                    .unwrap_or(1.0)
                    .max(1.0);
                let mut a = selected_state.borrow_mut();
                let x = (e.client_x().max(0) as f64 / vw * a.width as f64) as usize;
                let y = (e.client_y().max(0) as f64 / vh * a.height as f64) as usize;
                if let Some(url) = a.click(x, y) {
                    open(url);
                }
                if let Some(doc) = window.document() {
                    if let Ok(Some(el)) =
                        doc.query_selector("canvas:not(#lichen-pixels):not(#unicode-glyph), #grid")
                    {
                        if let Some(html) = el.dyn_ref::<web_sys::HtmlElement>() {
                            let _ = html.focus();
                        }
                    }
                }
            });
        let _ = layer.add_event_listener_with_callback("click", click.as_ref().unchecked_ref());
        click.forget();
    }
    let mut mirrored_text = String::new();
    // Capture keys at the page level rather than relying on canvas focus: text
    // selection, a click on empty space, and browser focus changes can all blur it.
    let keys = state.clone();
    let keyboard =
        Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(move |e: web_sys::KeyboardEvent| {
            if e.ctrl_key() || e.meta_key() || e.alt_key() {
                return;
            }
            if let Some(target) = e
                .target()
                .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
            {
                let tag = target.tag_name();
                if tag == "INPUT"
                    || tag == "TEXTAREA"
                    || tag == "SELECT"
                    || target.has_attribute("contenteditable")
                {
                    return;
                }
            }
            let key = e.key();
            let mut a = keys.borrow_mut();
            let handled = match key.as_str() {
                "ArrowLeft" => {
                    a.change_section(-1);
                    true
                }
                "ArrowRight" => {
                    a.change_section(1);
                    true
                }
                "ArrowUp" => {
                    a.clear_hover();
                    a.navigate(-1);
                    true
                }
                "ArrowDown" => {
                    a.clear_hover();
                    a.navigate(1);
                    true
                }
                "PageUp" => {
                    let step = a.height.saturating_sub(4).max(1) as i32;
                    a.clear_hover();
                    a.navigate(-step);
                    true
                }
                "PageDown" => {
                    let step = a.height.saturating_sub(4).max(1) as i32;
                    a.clear_hover();
                    a.navigate(step);
                    true
                }
                "Home" => {
                    a.key('g');
                    a.key('g');
                    true
                }
                "End" => {
                    a.key('G');
                    true
                }
                "Escape" => {
                    a.clear_hover();
                    a.back();
                    true
                }
                "Tab" => {
                    a.focus_link(e.shift_key());
                    true
                }
                "Enter" => {
                    a.clear_hover();
                    if let Some(url) = a.activate() {
                        open(url);
                    }
                    true
                }
                _ => {
                    let mut chars = key.chars();
                    match (chars.next(), chars.next()) {
                        (Some(c), None) if "12345hjkl nNgGr-+=0?".contains(c) => {
                            a.key(c);
                            true
                        }
                        _ => false,
                    }
                }
            };
            if handled {
                e.prevent_default();
            }
        });
    window
        .add_event_listener_with_callback("keydown", keyboard.as_ref().unchecked_ref())
        .map_err(|_| io::Error::other("could not register keyboard handler"))?;
    keyboard.forget();
    let mouse = state.clone();
    let _ = terminal.on_mouse_event(move |e| {
        if matches!(e.kind, MouseEventKind::SingleClick(_)) {
            if let Some(url) = mouse.borrow_mut().click(e.col as usize, e.row as usize) {
                open(url)
            }
        }
    });
    let scrolling = state.clone();
    let mut accumulated = 0.0_f64;
    let wheel = Closure::<dyn FnMut(WheelEvent)>::new(move |e: WheelEvent| {
        e.prevent_default();
        let unit = match e.delta_mode() {
            1 => 16.0,
            2 => web_sys::window()
                .and_then(|w| w.inner_height().ok())
                .and_then(|v| v.as_f64())
                .unwrap_or(600.0),
            _ => 1.0,
        };
        let dy = e.delta_y() * unit;
        if accumulated.signum() != dy.signum() {
            accumulated = 0.0
        }
        accumulated += dy;
        if accumulated.abs() >= 24.0 {
            let steps = ((accumulated.abs() / 24.0) as i32).clamp(1, 8);
            let direction = if accumulated > 0.0 { 1 } else { -1 };
            scrolling.borrow_mut().navigate(direction * steps);
            accumulated -= direction as f64 * steps as f64 * 24.0;
            accumulated = accumulated.clamp(-24.0, 24.0);
        }
    });
    let opts = AddEventListenerOptions::new();
    opts.set_passive(false);
    let _ = window.add_event_listener_with_callback_and_add_event_listener_options(
        "wheel",
        wheel.as_ref().unchecked_ref(),
        &opts,
    );
    wheel.forget();
    // Direct touch handling provides navigation and link activation without a keyboard.
    let touch_start_pos = Rc::new(RefCell::new(None::<(i32, i32)>));
    let start = touch_start_pos.clone();
    let down = Closure::<dyn FnMut(web_sys::TouchEvent)>::new(move |e: web_sys::TouchEvent| {
        if let Some(t) = e.touches().item(0) {
            *start.borrow_mut() = Some((t.client_x(), t.client_y()));
        }
    });
    let _ = window.add_event_listener_with_callback("touchstart", down.as_ref().unchecked_ref());
    down.forget();
    let touch_state = state.clone();
    let end = Closure::<dyn FnMut(web_sys::TouchEvent)>::new(move |e: web_sys::TouchEvent| {
        if let (Some((sx, sy)), Some(t)) = (
            touch_start_pos.borrow_mut().take(),
            e.changed_touches().item(0),
        ) {
            let dx = t.client_x() - sx;
            let dy = t.client_y() - sy;
            if dy.abs() > 35 {
                let steps = (dy.abs() / 25).clamp(1, 8);
                touch_state
                    .borrow_mut()
                    .navigate(if dy < 0 { steps } else { -steps });
                e.prevent_default();
            } else if dx.abs() < 20 {
                if let Some(w) = web_sys::window() {
                    let vw = w.inner_width().ok().and_then(|v| v.as_f64()).unwrap_or(1.0);
                    let vh = w
                        .inner_height()
                        .ok()
                        .and_then(|v| v.as_f64())
                        .unwrap_or(1.0);
                    let mut a = touch_state.borrow_mut();
                    let x = (sx as f64 / vw * a.width as f64) as usize;
                    let y = (sy as f64 / vh * a.height as f64) as usize;
                    if let Some(url) = a.click(x, y) {
                        open(url)
                    }
                    e.prevent_default();
                }
            }
        }
    });
    let _ = window.add_event_listener_with_callback_and_add_event_listener_options(
        "touchend",
        end.as_ref().unchecked_ref(),
        &opts,
    );
    end.forget();
    if let Ok(Some(el)) =
        document.query_selector("canvas:not(#lichen-pixels):not(#unicode-glyph), #grid")
    {
        if let Some(html) = el.dyn_ref::<web_sys::HtmlElement>() {
            let _ = html.set_attribute("tabindex", "0");
            let _ = html.focus();
        }
    }
    let mut last = 0.0;
    terminal.draw_web(move |f| {
        let mut a = state.borrow_mut();
        let area = f.area();
        a.resize(area.width, area.height);
        let w = web_sys::window().unwrap();
        let vw = w
            .inner_width()
            .ok()
            .and_then(|v| v.as_f64())
            .unwrap_or(640.0)
            .max(1.0);
        let vh = w
            .inner_height()
            .ok()
            .and_then(|v| v.as_f64())
            .unwrap_or(400.0)
            .max(1.0);
        let scale = ((vw * vh / 500_000.0).sqrt().ceil() as usize).max(2);
        let cols = (vw / scale as f64).ceil() as usize;
        let rows = (vh / scale as f64).ceil() as usize;
        let now = js_sys::Date::now();
        let elapsed = if last == 0.0 {
            0
        } else {
            (now - last).max(0.0) as u64
        };
        last = now;
        if a.view == View::Lichen {
            a.lichen.resize(cols, rows);
        }
        a.advance_lichen(elapsed);
        a.advance_fractal(elapsed);
        let mut font_status = FontStatus::NotRequested;
        if a.is_unicode() {
            if let Some(study) = a.unicode.as_ref() {
                let family = study.font_family();
                let glyph = study.glyph().to_string();
                let current = font_states
                    .borrow()
                    .get(family)
                    .copied()
                    .unwrap_or_default();
                if current == FontStatus::NotRequested {
                    font_states.borrow_mut().insert(family, FontStatus::Loading);
                    let states = font_states.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        let Some(doc) = web_sys::window().and_then(|w| w.document()) else {
                            states.borrow_mut().insert(family, FontStatus::Failed);
                            return;
                        };
                        let loaded = wasm_bindgen_futures::JsFuture::from(
                            doc.fonts()
                                .load_with_text(&format!("100px '{family}'"), &glyph),
                        )
                        .await;
                        let ok = loaded
                            .map(|faces| js_sys::Array::from(&faces).length() > 0)
                            .unwrap_or(false);
                        states.borrow_mut().insert(
                            family,
                            if ok {
                                FontStatus::Ready
                            } else {
                                FontStatus::Failed
                            },
                        );
                    });
                    font_status = FontStatus::Loading;
                } else {
                    font_status = current;
                }
            }
        }
        // Keep the last fully loaded specimen on screen until its replacement is ready.
        let real_view = a.view;
        if a.is_unicode() && !a.help {
            if font_status == FontStatus::Ready {
                displayed_glyph = a.unicode.as_ref().map(|s| s.glyph());
            } else if font_status == FontStatus::Loading {
                if let Some(glyph) = displayed_glyph {
                    if let Some(study) = a.unicode.as_mut() {
                        study.set_display_override(Some(glyph));
                    }
                    font_status = FontStatus::Ready;
                } else {
                    a.view = View::Study;
                }
            }
        }
        a.unicode_font_status = font_status;
        ui::draw(f, &a);
        if let Some(layer) = selection_layer.as_ref() {
            let selectable = a.view != View::Lichen || a.help;
            let _ = layer
                .style()
                .set_property("display", if selectable { "block" } else { "none" });
            if selectable {
                let mut text = String::with_capacity(
                    area.width as usize * area.height as usize + area.height as usize,
                );
                let buf = f.buffer_mut();
                for y in 0..area.height {
                    for x in 0..area.width {
                        text.push_str(buf[(area.x + x, area.y + y)].symbol());
                    }
                    if y + 1 < area.height {
                        text.push('\n');
                    }
                }
                if text != mirrored_text {
                    layer.set_text_content(Some(&text));
                    mirrored_text = text;
                }
                let cell_w = vw / area.width.max(1) as f64;
                let cell_h = vh / area.height.max(1) as f64;
                let _ = layer
                    .style()
                    .set_property("--cell-width", &format!("{cell_w}px"));
                let _ = layer
                    .style()
                    .set_property("--cell-height", &format!("{cell_h}px"));
                let _ = layer
                    .style()
                    .set_property("line-height", &format!("{cell_h}px"));
                let _ = layer
                    .style()
                    .set_property("letter-spacing", &format!("calc({cell_w}px - 1ch)"));
            }
        }
        if let (Some(canvas), Some(ctx)) = (&glyph_surface, &glyph_context) {
            let visible = a.is_unicode() && !a.help;
            let _ = canvas
                .style()
                .set_property("display", if visible { "block" } else { "none" });
            if visible {
                if let Some(study) = a.unicode.as_ref() {
                    let glyph = study.glyph();
                    let key = (
                        glyph,
                        vw as u32,
                        vh as u32,
                        area.width,
                        area.height,
                        font_status,
                    );
                    if glyph_cache != Some(key) {
                        glyph_cache = Some(key);
                        canvas.set_width(vw as u32);
                        canvas.set_height(vh as u32);
                        let _ = canvas.style().set_property("width", &format!("{vw}px"));
                        let _ = canvas.style().set_property("height", &format!("{vh}px"));
                        ctx.clear_rect(0.0, 0.0, vw, vh);
                        if font_status == FontStatus::Ready {
                            let family = study.font_family();
                            let text = glyph.to_string();
                            ctx.set_text_baseline("alphabetic");
                            ctx.set_font(&format!("100px '{family}'"));
                            if let Ok(m) = ctx.measure_text(&text) {
                                let ink_width = (m.actual_bounding_box_left()
                                    + m.actual_bounding_box_right())
                                .max(1.0);
                                let ink_height = (m.actual_bounding_box_ascent()
                                    + m.actual_bounding_box_descent())
                                .max(1.0);
                                let cell_w = vw / area.width.max(1) as f64;
                                let cell_h = vh / area.height.max(1) as f64;
                                let layout = ui::unicode_layout(
                                    study,
                                    area.width as usize,
                                    area.height as usize,
                                );
                                let left = layout.glyph_x as f64 * cell_w;
                                let top = layout.glyph_y as f64 * cell_h;
                                let width = (layout.glyph_width as f64 * cell_w).max(1.0);
                                let height = (layout.glyph_height as f64 * cell_h).max(1.0);
                                let scale = (width * 0.88 / ink_width)
                                    .min(height * 0.88 / ink_height)
                                    .min(15.0);
                                ctx.set_font(&format!("{}px '{family}'", 100.0 * scale));
                                if let Ok(m) = ctx.measure_text(&text) {
                                    let iw = m.actual_bounding_box_left()
                                        + m.actual_bounding_box_right();
                                    let ih = m.actual_bounding_box_ascent()
                                        + m.actual_bounding_box_descent();
                                    let x =
                                        left + (width - iw) / 2.0 + m.actual_bounding_box_left();
                                    let y =
                                        top + (height - ih) / 2.0 + m.actual_bounding_box_ascent();
                                    ctx.set_fill_style_str("#eeeeee");
                                    let _ = ctx.fill_text(&text, x, y);
                                }
                            }
                        }
                    }
                }
            }
        }
        if let Some(study) = a.unicode.as_mut() {
            study.set_display_override(None);
        }
        a.view = real_view;
        if let (Some(canvas), Some(ctx)) = (&surface, &context) {
            let active = a.view == View::Lichen && !a.help;
            let _ = canvas
                .style()
                .set_property("display", if active { "block" } else { "none" });
            if active {
                // Pixelation is a display-only downsample: zooming does not erase or
                // resize the running simulation. Aggregate cells so sparse spores survive.
                let factor = 1usize << a.lichen_pixelation;
                let out_cols = cols.div_ceil(factor);
                let out_rows = rows.div_ceil(factor);
                let pixel_size = scale * factor;
                if canvas.width() != out_cols as u32 || canvas.height() != out_rows as u32 {
                    canvas.set_width(out_cols as u32);
                    canvas.set_height(out_rows as u32);
                }
                let _ = canvas
                    .style()
                    .set_property("width", &format!("{}px", out_cols * pixel_size));
                let _ = canvas
                    .style()
                    .set_property("height", &format!("{}px", out_rows * pixel_size));
                let mut rgba = Vec::with_capacity(out_cols * out_rows * 4);
                for y in 0..out_rows {
                    for x in 0..out_cols {
                        let mut v = 0;
                        for sy in y * factor..((y + 1) * factor).min(rows) {
                            for sx in x * factor..((x + 1) * factor).min(cols) {
                                v = v.max(a.lichen.display_value(sy * cols + sx));
                            }
                        }
                        rgba.extend_from_slice(&[v, v, v, 255]);
                    }
                }
                if let Ok(data) = ImageData::new_with_u8_clamped_array_and_sh(
                    Clamped(&rgba),
                    out_cols as u32,
                    out_rows as u32,
                ) {
                    let _ = ctx.put_image_data(&data, 0.0, 0.0);
                }
            }
        }
    });
    Ok(())
}
pub fn start() {
    console_error_panic_hook::set_once();
    wasm_bindgen_futures::spawn_local(async {
        if let Some(d) = web_sys::window().and_then(|w| w.document()) {
            let _ =
                wasm_bindgen_futures::JsFuture::from(d.fonts().load("16px 'Commit Mono'")).await;
        }
        if let Err(e) = initialize() {
            web_sys::console::error_1(&e.to_string().into());
        }
    });
}
fn initialize() -> io::Result<()> {
    use ratzilla::backend::webgl2::{FontAtlasConfig, WebGl2BackendOptions};
    let options = WebGl2BackendOptions::new()
        .disable_auto_css_resize()
        .font_atlas_config(FontAtlasConfig::dynamic(
            &["Commit Mono", "monospace"],
            16.0,
        ));
    if let Ok(b) = WebGl2Backend::new_with_options(options) {
        return run(b);
    }
    if let Ok(b) = CanvasBackend::new() {
        return run(b);
    }
    run(DomBackend::new()?)
}
