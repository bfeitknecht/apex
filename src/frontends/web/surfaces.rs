//! Browser paint targets.
//!
//! Ratzilla draws the Ratatui cells. Two extra canvases layer on top of them:
//! one shows the colony as square pixels, the other the enlarged specimen. The
//! third surface is a transparent text mirror that makes the rendered cells
//! selectable and copyable with the normal browser gestures.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use ratatui::layout::Rect;
use wasm_bindgen::{Clamped, JsCast};
use web_sys::{CanvasRenderingContext2d, Document, HtmlCanvasElement, HtmlElement, ImageData};

use crate::app::{App, FontStatus};
use crate::ui;
use crate::visuals::{Lichen, Unicode};

/// The browser viewport in CSS pixels.
#[derive(Clone, Copy, Debug)]
pub struct Viewport {
    pub width: f64,
    pub height: f64,
}

impl Viewport {
    pub fn current() -> Self {
        let window = web_sys::window().expect("window");
        let width = window
            .inner_width()
            .ok()
            .and_then(|value| value.as_f64())
            .unwrap_or(640.0)
            .max(1.0);
        let height = window
            .inner_height()
            .ok()
            .and_then(|value| value.as_f64())
            .unwrap_or(400.0)
            .max(1.0);
        Self { width, height }
    }

    /// Size of one character cell in pixels.
    pub fn cell(self, area: Rect) -> (f64, f64) {
        (
            self.width / area.width.max(1) as f64,
            self.height / area.height.max(1) as f64,
        )
    }
}

fn canvas(document: &Document, id: &str) -> Option<HtmlCanvasElement> {
    document
        .get_element_by_id(id)
        .and_then(|element| element.dyn_into::<HtmlCanvasElement>().ok())
}

fn context(canvas: &HtmlCanvasElement) -> Option<CanvasRenderingContext2d> {
    canvas
        .get_context("2d")
        .ok()
        .flatten()
        .and_then(|value| value.dyn_into::<CanvasRenderingContext2d>().ok())
}

fn show(canvas: &Option<HtmlCanvasElement>, visible: bool) {
    if let Some(canvas) = canvas {
        let _ = canvas
            .style()
            .set_property("display", if visible { "block" } else { "none" });
    }
}

/// The colony drawn as square pixels.
pub struct LichenSurface {
    canvas: Option<HtmlCanvasElement>,
    context: Option<CanvasRenderingContext2d>,
}

impl LichenSurface {
    pub fn new(document: &Document) -> Self {
        let canvas = canvas(document, "lichen-pixels");
        let context = canvas.as_ref().and_then(context);
        Self { canvas, context }
    }

    pub fn set_visible(&self, visible: bool) {
        show(&self.canvas, visible);
    }

    /// Paint one frame, downsampled by the display pixelation.
    pub fn paint(
        &mut self,
        lichen: &Lichen,
        pixelation: u8,
        columns: usize,
        rows: usize,
        cell_size: usize,
    ) {
        let (Some(canvas), Some(context)) = (self.canvas.as_ref(), self.context.as_ref()) else {
            return;
        };
        if lichen.size() != (columns, rows) {
            return;
        }
        // Pixelation is a display-only downsample: zooming never resizes the
        // simulation, and pooling keeps sparse spores visible.
        let factor = 1usize << pixelation;
        let out_columns = columns.div_ceil(factor);
        let out_rows = rows.div_ceil(factor);
        let pixel_size = cell_size * factor;
        if canvas.width() != out_columns as u32 || canvas.height() != out_rows as u32 {
            canvas.set_width(out_columns as u32);
            canvas.set_height(out_rows as u32);
        }
        let _ = canvas
            .style()
            .set_property("width", &format!("{}px", out_columns * pixel_size));
        let _ = canvas
            .style()
            .set_property("height", &format!("{}px", out_rows * pixel_size));

        let mut rgba = Vec::with_capacity(out_columns * out_rows * 4);
        for y in 0..out_rows {
            for x in 0..out_columns {
                let mut level = 0;
                for source_y in y * factor..((y + 1) * factor).min(rows) {
                    for source_x in x * factor..((x + 1) * factor).min(columns) {
                        level = level.max(lichen.display_value(source_y * columns + source_x));
                    }
                }
                rgba.extend_from_slice(&[level, level, level, 255]);
            }
        }
        if let Ok(image) = ImageData::new_with_u8_clamped_array_and_sh(
            Clamped(&rgba),
            out_columns as u32,
            out_rows as u32,
        ) {
            let _ = context.put_image_data(&image, 0.0, 0.0);
        }
    }
}

/// The enlarged Unicode specimen, measured and centred in its cell area.
pub struct GlyphSurface {
    canvas: Option<HtmlCanvasElement>,
    context: Option<CanvasRenderingContext2d>,
    painted: Option<Painted>,
}

/// What the last painted frame shows, so an unchanged frame is not repainted.
type Painted = (char, u32, u32, u16, u16, FontStatus);

/// Largest font size the specimen is magnified to, as a multiple of 100px.
const MAX_MAGNIFICATION: f64 = 15.0;

impl GlyphSurface {
    pub fn new(document: &Document) -> Self {
        let canvas = canvas(document, "unicode-glyph");
        let context = canvas.as_ref().and_then(context);
        Self {
            canvas,
            context,
            painted: None,
        }
    }

    pub fn set_visible(&self, visible: bool) {
        show(&self.canvas, visible);
    }

    /// Draw the specimen as large as the glyph area allows.
    pub fn paint(&mut self, unicode: &Unicode, status: FontStatus, area: Rect, viewport: Viewport) {
        let (Some(canvas), Some(context)) = (self.canvas.as_ref(), self.context.as_ref()) else {
            return;
        };
        let glyph = unicode.glyph();
        let painted = (
            glyph,
            viewport.width as u32,
            viewport.height as u32,
            area.width,
            area.height,
            status,
        );
        if self.painted == Some(painted) {
            return;
        }
        self.painted = Some(painted);

        canvas.set_width(viewport.width as u32);
        canvas.set_height(viewport.height as u32);
        let _ = canvas
            .style()
            .set_property("width", &format!("{}px", viewport.width));
        let _ = canvas
            .style()
            .set_property("height", &format!("{}px", viewport.height));
        context.clear_rect(0.0, 0.0, viewport.width, viewport.height);
        if status != FontStatus::Ready {
            return;
        }

        let family = unicode.font_family();
        let text = glyph.to_string();
        let (cell_width, cell_height) = viewport.cell(area);
        let layout = ui::unicode_layout(unicode, area.width as usize, area.height as usize);
        let left = layout.glyph_x as f64 * cell_width;
        let top = layout.glyph_y as f64 * cell_height;
        let width = (layout.glyph_width as f64 * cell_width).max(1.0);
        let height = (layout.glyph_height as f64 * cell_height).max(1.0);

        context.set_text_baseline("alphabetic");
        context.set_font(&format!("100px '{family}'"));
        let Ok(metrics) = context.measure_text(&text) else {
            return;
        };
        let ink_width =
            (metrics.actual_bounding_box_left() + metrics.actual_bounding_box_right()).max(1.0);
        let ink_height =
            (metrics.actual_bounding_box_ascent() + metrics.actual_bounding_box_descent()).max(1.0);
        let scale = (width * 0.88 / ink_width)
            .min(height * 0.88 / ink_height)
            .min(MAX_MAGNIFICATION);
        context.set_font(&format!("{}px '{family}'", 100.0 * scale));
        let Ok(metrics) = context.measure_text(&text) else {
            return;
        };
        let ink_width = metrics.actual_bounding_box_left() + metrics.actual_bounding_box_right();
        let ink_height =
            metrics.actual_bounding_box_ascent() + metrics.actual_bounding_box_descent();
        let x = left + (width - ink_width) / 2.0 + metrics.actual_bounding_box_left();
        let y = top + (height - ink_height) / 2.0 + metrics.actual_bounding_box_ascent();
        context.set_fill_style_str("#eeeeee");
        let _ = context.fill_text(&text, x, y);
    }
}

/// A transparent copy of the rendered cells for native text selection.
pub struct TextMirror {
    layer: Option<HtmlElement>,
    text: String,
}

impl TextMirror {
    pub fn new(document: &Document) -> Self {
        let layer = document
            .get_element_by_id("selectable-text")
            .and_then(|element| element.dyn_into::<HtmlElement>().ok());
        Self {
            layer,
            text: String::new(),
        }
    }

    pub fn set_visible(&self, visible: bool) {
        if let Some(layer) = self.layer.as_ref() {
            let _ = layer
                .style()
                .set_property("display", if visible { "block" } else { "none" });
        }
    }

    /// Copy the cells into the layer and align it with the rendered grid.
    pub fn sync(&mut self, frame: &mut ratatui::Frame, viewport: Viewport) {
        let Some(layer) = self.layer.as_ref() else {
            return;
        };
        let area = frame.area();
        let mut text = String::with_capacity(
            area.width as usize * area.height as usize + area.height as usize,
        );
        let buffer = frame.buffer_mut();
        for y in 0..area.height {
            for x in 0..area.width {
                text.push_str(buffer[(area.x + x, area.y + y)].symbol());
            }
            if y + 1 < area.height {
                text.push('\n');
            }
        }
        if text != self.text {
            layer.set_text_content(Some(&text));
            self.text = text;
        }
        let (cell_width, cell_height) = viewport.cell(area);
        let _ = layer
            .style()
            .set_property("--cell-width", &format!("{cell_width}px"));
        let _ = layer
            .style()
            .set_property("--cell-height", &format!("{cell_height}px"));
        let _ = layer
            .style()
            .set_property("line-height", &format!("{cell_height}px"));
        let _ = layer
            .style()
            .set_property("letter-spacing", &format!("calc({cell_width}px - 1ch)"));
    }
}

/// Font loading state per family, shared with the request futures.
#[derive(Clone, Default)]
pub struct FontCache {
    states: Rc<RefCell<BTreeMap<&'static str, FontStatus>>>,
}

impl FontCache {
    /// Ask for the specimen's font once, then report the cached state.
    pub fn status(&self, app: &App) -> FontStatus {
        let Some(unicode) = app.unicode.as_ref() else {
            return FontStatus::NotRequested;
        };
        let family = unicode.font_family();
        let glyph = unicode.glyph().to_string();
        let state = self
            .states
            .borrow()
            .get(family)
            .copied()
            .unwrap_or_default();
        if state == FontStatus::NotRequested {
            self.load(family, glyph);
            FontStatus::Loading
        } else {
            state
        }
    }

    fn load(&self, family: &'static str, glyph: String) {
        self.states.borrow_mut().insert(family, FontStatus::Loading);
        let states = self.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let Some(document) = web_sys::window().and_then(|window| window.document()) else {
                states
                    .states
                    .borrow_mut()
                    .insert(family, FontStatus::Failed);
                return;
            };
            let loaded = wasm_bindgen_futures::JsFuture::from(
                document
                    .fonts()
                    .load_with_text(&format!("100px '{family}'"), &glyph),
            )
            .await;
            let ready = loaded
                .map(|faces| js_sys::Array::from(&faces).length() > 0)
                .unwrap_or(false);
            states.states.borrow_mut().insert(
                family,
                if ready {
                    FontStatus::Ready
                } else {
                    FontStatus::Failed
                },
            );
        });
    }
}
