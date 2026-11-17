//! Browser input: DOM listeners decoded into the shared [`App`].
//!
//! The code captures keys at the window level, not on the canvas. A text
//! selection, a click on empty space or a browser focus change would otherwise
//! remove the focus from the canvas.

use std::cell::{Cell, RefCell};
use std::io;
use std::rc::Rc;

use ratzilla::event::MouseEventKind;
use ratzilla::ratatui::{backend::Backend, Terminal};
use ratzilla::{CellSized, WebEventHandler, WebRenderer};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{AddEventListenerOptions, Document, Window};

use super::surfaces::Viewport;
use super::{open, Shared};

/// Pointer travel in pixels above which a press counts as a text-selection drag.
const DRAG_THRESHOLD: i32 = 4;

/// The animation-frame callback of a glide. It re-schedules itself.
type FrameSlot = Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>>;

/// Finger travel in pixels below which a touch is a tap.
const TAP_SLOP: i32 = 10;

/// Glide speed (pixels per millisecond) below which momentum stops.
const GLIDE_MIN: f64 = 0.05;

/// Velocity kept per 16 ms frame of glide.
const GLIDE_DECAY: f64 = 0.95;

/// Wheel delta in pixels that turns into one scroll step.
const WHEEL_STEP: f64 = 24.0;

/// Keys that the application consumes. The browser keeps all other keys.
const APPLICATION_KEYS: &str = "12345hjkl nNgGr-+=0?";

/// Register every listener the portfolio needs.
pub(super) fn install<B>(
    terminal: &mut Terminal<B>,
    window: &Window,
    document: &Document,
    state: Shared,
) -> io::Result<()>
where
    B: Backend<Error = io::Error> + WebEventHandler + CellSized + 'static,
{
    install_text_layer(document, state.clone());
    install_keyboard(window, state.clone())?;
    install_mouse(terminal, state.clone());
    install_wheel(window, state.clone());
    install_touch(window, state);
    focus_canvas(document);
    Ok(())
}

/// Hover, click and drag on the transparent text layer.
fn install_text_layer(document: &Document, state: Shared) {
    let Some(layer) = document
        .get_element_by_id("selectable-text")
        .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok())
    else {
        return;
    };

    let hover_state = state.clone();
    let hover_layer = layer.clone();
    let hover =
        Closure::<dyn FnMut(web_sys::MouseEvent)>::new(move |event: web_sys::MouseEvent| {
            let viewport = Viewport::current();
            let Ok(mut app) = hover_state.try_borrow_mut() else {
                return;
            };
            let x = (event.client_x().max(0) as f64 / viewport.width * app.width as f64) as usize;
            let y = (event.client_y().max(0) as f64 / viewport.height * app.height as f64) as usize;
            let active = app.hover(x, y);
            let _ =
                hover_layer.set_attribute("data-clickable", if active { "true" } else { "false" });
        });
    let _ = layer.add_event_listener_with_callback("mousemove", hover.as_ref().unchecked_ref());
    hover.forget();

    let leave_state = state.clone();
    let leave = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(move |_| {
        if let Ok(mut app) = leave_state.try_borrow_mut() {
            app.clear_hover();
        }
    });
    let _ = layer.add_event_listener_with_callback("mouseleave", leave.as_ref().unchecked_ref());
    leave.forget();

    let press = Rc::new(Cell::new(None::<(i32, i32)>));
    let press_slot = press.clone();
    let down = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(move |event: web_sys::MouseEvent| {
        press_slot.set(Some((event.client_x(), event.client_y())));
    });
    let _ = layer.add_event_listener_with_callback("mousedown", down.as_ref().unchecked_ref());
    down.forget();

    let click_state = state.clone();
    let click =
        Closure::<dyn FnMut(web_sys::MouseEvent)>::new(move |event: web_sys::MouseEvent| {
            let Some(window) = web_sys::window() else {
                return;
            };
            let dragged = press
                .take()
                .map(|(x, y)| {
                    (x - event.client_x()).abs() > DRAG_THRESHOLD
                        || (y - event.client_y()).abs() > DRAG_THRESHOLD
                })
                .unwrap_or(false);
            if dragged {
                // Keep the browser's own text selection.
                focus_canvas_of(&window);
                return;
            }
            if let Ok(Some(selection)) = window.get_selection() {
                if !selection.is_collapsed() {
                    let _ = selection.remove_all_ranges();
                }
            }
            let viewport = Viewport::current();
            let mut app = click_state.borrow_mut();
            let x = (event.client_x().max(0) as f64 / viewport.width * app.width as f64) as usize;
            let y = (event.client_y().max(0) as f64 / viewport.height * app.height as f64) as usize;
            let url = app.click(x, y);
            drop(app);
            if let Some(url) = url {
                open(url);
            }
            focus_canvas_of(&window);
        });
    let _ = layer.add_event_listener_with_callback("click", click.as_ref().unchecked_ref());
    click.forget();
}

fn install_keyboard(window: &Window, state: Shared) -> io::Result<()> {
    let keyboard =
        Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(move |event: web_sys::KeyboardEvent| {
            if event.ctrl_key() || event.meta_key() || event.alt_key() {
                return;
            }
            if let Some(target) = event
                .target()
                .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
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
            let key = event.key();
            let mut app = state.borrow_mut();
            let mut url = None;
            let handled = match key.as_str() {
                "ArrowLeft" => {
                    app.change_section(-1);
                    true
                }
                "ArrowRight" => {
                    app.change_section(1);
                    true
                }
                "ArrowUp" => {
                    app.clear_hover();
                    app.navigate(-1);
                    true
                }
                "ArrowDown" => {
                    app.clear_hover();
                    app.navigate(1);
                    true
                }
                "PageUp" => {
                    let step = page_step(&app);
                    app.clear_hover();
                    app.navigate(-step);
                    true
                }
                "PageDown" => {
                    let step = page_step(&app);
                    app.clear_hover();
                    app.navigate(step);
                    true
                }
                "Home" => {
                    app.key('g');
                    app.key('g');
                    true
                }
                "End" => {
                    app.key('G');
                    true
                }
                "Escape" => {
                    app.clear_hover();
                    app.back();
                    true
                }
                "Tab" => {
                    app.focus_link(event.shift_key());
                    true
                }
                "Enter" => {
                    app.clear_hover();
                    url = app.activate();
                    true
                }
                _ => {
                    let mut characters = key.chars();
                    match (characters.next(), characters.next()) {
                        (Some(character), None) if APPLICATION_KEYS.contains(character) => {
                            app.key(character);
                            true
                        }
                        _ => false,
                    }
                }
            };
            drop(app);
            if handled {
                event.prevent_default();
            }
            if let Some(url) = url {
                open(url);
            }
        });
    window
        .add_event_listener_with_callback("keydown", keyboard.as_ref().unchecked_ref())
        .map_err(|_| io::Error::other("could not register the keyboard handler"))?;
    keyboard.forget();
    Ok(())
}

fn install_mouse<B>(terminal: &mut Terminal<B>, state: Shared)
where
    B: Backend<Error = io::Error> + WebEventHandler + CellSized + 'static,
{
    let _ = terminal.on_mouse_event(move |event| {
        if matches!(event.kind, MouseEventKind::SingleClick(_)) {
            let url = state
                .borrow_mut()
                .click(event.col as usize, event.row as usize);
            if let Some(url) = url {
                open(url);
            }
        }
    });
}

fn install_wheel(window: &Window, state: Shared) {
    let mut accumulated = 0.0_f64;
    let wheel =
        Closure::<dyn FnMut(web_sys::WheelEvent)>::new(move |event: web_sys::WheelEvent| {
            event.prevent_default();
            let unit = match event.delta_mode() {
                1 => 16.0,
                2 => web_sys::window()
                    .and_then(|window| window.inner_height().ok())
                    .and_then(|value| value.as_f64())
                    .unwrap_or(600.0),
                _ => 1.0,
            };
            let delta = event.delta_y() * unit;
            if accumulated.signum() != delta.signum() {
                accumulated = 0.0;
            }
            accumulated += delta;
            if accumulated.abs() >= WHEEL_STEP {
                let steps = ((accumulated.abs() / WHEEL_STEP) as i32).clamp(1, 8);
                let direction = if accumulated > 0.0 { 1 } else { -1 };
                if let Ok(mut app) = state.try_borrow_mut() {
                    app.navigate(direction * steps);
                }
                accumulated -= direction as f64 * steps as f64 * WHEEL_STEP;
                accumulated = accumulated.clamp(-WHEEL_STEP, WHEEL_STEP);
            }
        });
    let options = AddEventListenerOptions::new();
    options.set_passive(false);
    if window
        .add_event_listener_with_callback_and_add_event_listener_options(
            "wheel",
            wheel.as_ref().unchecked_ref(),
            &options,
        )
        .is_ok()
    {
        wheel.forget();
    }
}

/// Touch: the page follows the finger row by row and glides on after release.
/// A tap (little movement) activates the link under it.
fn install_touch(window: &Window, state: Shared) {
    #[derive(Default)]
    struct Touch {
        start: Option<(i32, i32)>,
        last_y: f64,
        last_ms: f64,
        /// Pixels not yet turned into whole rows.
        carry: f64,
        /// Finger velocity in pixels per millisecond (positive: content moves up).
        velocity: f64,
        moved: bool,
        /// Generation of the running glide; a new touch stops it.
        glide: u32,
    }
    let touch = Rc::new(RefCell::new(Touch::default()));

    // Turn `pixels` of finger travel into row steps. Returns false at an end.
    fn step(state: &Shared, touch: &mut Touch, pixels: f64) -> bool {
        let Ok(mut app) = state.try_borrow_mut() else {
            return true;
        };
        let row = Viewport::current().height / app.height.max(1) as f64;
        touch.carry += pixels;
        let rows = (touch.carry / row).trunc() as i32;
        if rows == 0 {
            return true;
        }
        touch.carry -= rows as f64 * row;
        let before = app.scroll_offset;
        app.clear_hover();
        app.navigate(rows);
        app.scroll_offset != before
    }

    let down_touch = touch.clone();
    let down = Closure::<dyn FnMut(web_sys::TouchEvent)>::new(move |event: web_sys::TouchEvent| {
        if let Some(point) = event.touches().item(0) {
            let mut touch = down_touch.borrow_mut();
            touch.start = Some((point.client_x(), point.client_y()));
            touch.last_y = point.client_y() as f64;
            touch.last_ms = js_sys::Date::now();
            touch.carry = 0.0;
            touch.velocity = 0.0;
            touch.moved = false;
            touch.glide = touch.glide.wrapping_add(1);
        }
    });
    let _ = window.add_event_listener_with_callback("touchstart", down.as_ref().unchecked_ref());
    down.forget();

    let move_state = state.clone();
    let move_touch = touch.clone();
    let moved =
        Closure::<dyn FnMut(web_sys::TouchEvent)>::new(move |event: web_sys::TouchEvent| {
            let Some(point) = event.touches().item(0) else {
                return;
            };
            let mut touch = move_touch.borrow_mut();
            let Some((start_x, start_y)) = touch.start else {
                return;
            };
            let y = point.client_y() as f64;
            if !touch.moved {
                let (dx, dy) = (point.client_x() - start_x, point.client_y() - start_y);
                if dy.abs() < TAP_SLOP || dx.abs() > dy.abs() {
                    return;
                }
                touch.moved = true;
                touch.last_y = y;
            }
            event.prevent_default();
            let now = js_sys::Date::now();
            let pixels = touch.last_y - y;
            let elapsed = (now - touch.last_ms).max(1.0);
            // Smoothed, so one jittery event does not decide the glide.
            touch.velocity = 0.8 * (pixels / elapsed) + 0.2 * touch.velocity;
            touch.last_y = y;
            touch.last_ms = now;
            step(&move_state, &mut touch, pixels);
        });
    let options = AddEventListenerOptions::new();
    options.set_passive(false);
    let _ = window.add_event_listener_with_callback_and_add_event_listener_options(
        "touchmove",
        moved.as_ref().unchecked_ref(),
        &options,
    );
    moved.forget();

    let end_touch = touch;
    let end = Closure::<dyn FnMut(web_sys::TouchEvent)>::new(move |event: web_sys::TouchEvent| {
        let (start, moved, glide, velocity, last_ms) = {
            let mut touch = end_touch.borrow_mut();
            (
                touch.start.take(),
                touch.moved,
                touch.glide,
                touch.velocity,
                touch.last_ms,
            )
        };
        let (Some((start_x, start_y)), Some(point)) = (start, event.changed_touches().item(0))
        else {
            return;
        };
        if !moved {
            let (dx, dy) = (point.client_x() - start_x, point.client_y() - start_y);
            if dx.abs() < TAP_SLOP && dy.abs() < TAP_SLOP {
                let viewport = Viewport::current();
                let mut app = state.borrow_mut();
                let x = (start_x.max(0) as f64 / viewport.width * app.width as f64) as usize;
                let y = (start_y.max(0) as f64 / viewport.height * app.height as f64) as usize;
                let url = app.click(x, y);
                drop(app);
                if let Some(url) = url {
                    open(url);
                }
                event.prevent_default();
            }
            return;
        }
        event.prevent_default();
        // A finger that stopped before lifting does not glide.
        if js_sys::Date::now() - last_ms > 80.0 || velocity.abs() < GLIDE_MIN {
            return;
        }
        glide_on(state.clone(), end_touch.clone(), glide, velocity);
    });
    let options = AddEventListenerOptions::new();
    options.set_passive(false);
    let _ = window.add_event_listener_with_callback_and_add_event_listener_options(
        "touchend",
        end.as_ref().unchecked_ref(),
        &options,
    );
    end.forget();

    /// Momentum after release: one animation frame at a time, decaying, until
    /// it is slow, hits an end, or a new touch starts.
    fn glide_on(state: Shared, touch: Rc<RefCell<Touch>>, glide: u32, velocity: f64) {
        let frame: FrameSlot = Rc::new(RefCell::new(None));
        let next = frame.clone();
        let mut velocity = velocity;
        let mut last: Option<f64> = None;
        *frame.borrow_mut() = Some(Closure::new(move |now: f64| {
            let elapsed = last.map_or(16.0, |last| (now - last).clamp(1.0, 64.0));
            last = Some(now);
            let mut current = touch.borrow_mut();
            let running = current.glide == glide
                && velocity.abs() >= GLIDE_MIN
                && step(&state, &mut current, velocity * elapsed);
            drop(current);
            velocity *= GLIDE_DECAY.powf(elapsed / 16.0);
            if running {
                request_frame(next.borrow().as_ref());
            } else {
                // Drop the closure: this breaks the Rc cycle.
                next.borrow_mut().take();
            }
        }));
        request_frame(frame.borrow().as_ref());
    }

    fn request_frame(callback: Option<&Closure<dyn FnMut(f64)>>) {
        if let (Some(window), Some(callback)) = (web_sys::window(), callback) {
            let _ = window.request_animation_frame(callback.as_ref().unchecked_ref());
        }
    }
}

fn page_step(app: &crate::App) -> i32 {
    app.height.saturating_sub(4).max(1) as i32
}

/// Give the rendered canvas focus so that it can receive input.
fn focus_canvas(document: &Document) {
    if let Ok(Some(element)) =
        document.query_selector("canvas:not(#lichen-pixels):not(#unicode-glyph), #grid")
    {
        if let Some(element) = element.dyn_ref::<web_sys::HtmlElement>() {
            let _ = element.set_attribute("tabindex", "0");
            let _ = element.focus();
        }
    }
}

fn focus_canvas_of(window: &Window) {
    if let Some(document) = window.document() {
        focus_canvas(&document);
    }
}
