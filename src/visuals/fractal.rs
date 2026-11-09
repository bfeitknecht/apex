//! A fullscreen ASCII explorer of the quaternion Julia set.
//!
//! The plane is rotated through four-dimensional space while it slowly zooms;
//! each cell iterates `q -> q² + c` and maps its escape count and orbit trap to
//! a character ramp. Only the character grid is produced here — the terminal
//! paints it and the browser shows the same text through Ratzilla.

/// Characters from sparse to dense, chosen for even visual weight.
const RAMP: &[u8] = b".,:;irsXA253hMHGS#9B&@";

/// Initial half-width of the visible plane.
const DEFAULT_SCALE: f64 = 3.2;
const MIN_SCALE: f64 = 0.4;
const MAX_SCALE: f64 = 3.2;

/// Rotation speed in radians per second.
const PHASE_SPEED: f64 = 0.28;

/// Exponential zoom speed per second.
const ZOOM_SPEED: f64 = 0.035;

/// How far one key press pans, relative to the visible width and height.
const HORIZONTAL_PAN: f64 = 0.12;
const VERTICAL_PAN: f64 = 0.08;

const MAX_ITERATIONS: usize = 28;
const ESCAPE_RADIUS_SQUARED: f64 = 16.0;

/// Milliseconds between rendered frames, and the most that is folded into one.
const FRAME_MS: u64 = 50;
const MAX_FRAME_MS: u64 = 250;
const MAX_ELAPSED_MS: u64 = 200;

/// Camera state of the explorer.
#[derive(Debug, Clone)]
pub struct Fractal {
    x: f64,
    y: f64,
    scale: f64,
    phase: f64,
    direction: f64,
    elapsed_ms: u64,
}

impl Default for Fractal {
    fn default() -> Self {
        Self::new()
    }
}

impl Fractal {
    pub fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            scale: DEFAULT_SCALE,
            phase: 0.0,
            direction: 1.0,
            elapsed_ms: 0,
        }
    }

    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn y(&self) -> f64 {
        self.y
    }

    pub fn scale(&self) -> f64 {
        self.scale
    }

    pub fn phase(&self) -> f64 {
        self.phase
    }

    /// Pan by whole steps in viewport units; positive moves right and down.
    pub fn pan(&mut self, horizontal: f64, vertical: f64) {
        self.x += horizontal * self.scale * HORIZONTAL_PAN;
        self.y += vertical * self.scale * VERTICAL_PAN;
    }

    /// Zoom in by `factor`; values above one zoom in, below one zoom out.
    pub fn zoom(&mut self, factor: f64) {
        self.scale = (self.scale / factor).clamp(MIN_SCALE, MAX_SCALE);
    }

    /// Return to the starting camera.
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Discard accumulated time, so leaving and re-entering the visual does not
    /// make the camera jump by however long it was hidden.
    pub fn reset_clock(&mut self) {
        self.elapsed_ms = 0;
    }

    /// Rotate and zoom. Returns `true` when a new frame should be drawn.
    pub fn advance(&mut self, elapsed_ms: u64) -> bool {
        self.elapsed_ms = (self.elapsed_ms + elapsed_ms.min(MAX_ELAPSED_MS)).min(MAX_FRAME_MS);
        if self.elapsed_ms < FRAME_MS {
            return false;
        }
        let seconds = self.elapsed_ms as f64 / 1000.0;
        self.elapsed_ms = 0;
        self.phase += seconds * PHASE_SPEED;
        self.scale = (self.scale * (-self.direction * ZOOM_SPEED * seconds).exp())
            .clamp(MIN_SCALE, MAX_SCALE);
        if self.scale <= MIN_SCALE {
            self.direction = -1.0;
        }
        if self.scale >= MAX_SCALE {
            self.direction = 1.0;
        }
        true
    }

    /// Render `height` rows of `width` characters for the current camera.
    pub fn rows(&self, width: usize, height: usize) -> Vec<String> {
        let columns = width.max(1) as f64;
        let (sin, cos) = (self.phase.sin(), self.phase.cos());
        // The two remaining quaternion axes drift on their own, so the slice
        // never repeats and detail keeps emerging.
        let slice_z = 0.28 * (self.phase * 0.73).sin();
        let slice_w = 0.26 * (self.phase * 0.47).cos();
        let mut rows = Vec::with_capacity(height);
        for row in 0..height {
            let py = self.y + (row as f64 - height as f64 / 2.0) * self.scale * 2.0 / columns;
            let mut line = String::with_capacity(width);
            for column in 0..width {
                let px = self.x + (column as f64 / columns - 0.5) * self.scale;
                line.push(char::from(
                    RAMP[self.shade(px, py, sin, cos, slice_z, slice_w)],
                ));
            }
            rows.push(line);
        }
        rows
    }

    /// Iterate one point and map it onto the character ramp.
    fn shade(&self, px: f64, py: f64, sin: f64, cos: f64, slice_z: f64, slice_w: f64) -> usize {
        // Rotate in two independent four-dimensional planes.
        let (mut a, mut b) = (px * cos - slice_z * sin, py * cos - slice_w * sin);
        let (mut c, mut d) = (px * sin + slice_z * cos, py * sin + slice_w * cos);
        let mut trap = 4.0_f64;
        let mut iterations = 0;
        while iterations < MAX_ITERATIONS {
            let radius_squared = a * a + b * b + c * c + d * d;
            trap = trap.min((a.abs() + b.abs() * 0.6 + c.abs() * 0.4 + d.abs() * 0.3).abs());
            if radius_squared > ESCAPE_RADIUS_SQUARED {
                break;
            }
            (a, b, c, d) = (
                a * a - b * b - c * c - d * d - 0.23,
                2.0 * a * b + 0.71,
                2.0 * a * c + 0.16,
                2.0 * a * d + 0.08,
            );
            iterations += 1;
        }
        let detail = ((1.0 / (0.15 + trap)).ln_1p() * 4.2) as usize;
        (iterations * 3 + detail * 2) % RAMP.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rendering_fills_every_cell() {
        let fractal = Fractal::new();
        let rows = fractal.rows(40, 12);
        assert_eq!(rows.len(), 12);
        assert!(rows.iter().all(|row| row.chars().count() == 40));
    }

    #[test]
    fn panning_and_zooming_move_the_camera() {
        let mut fractal = Fractal::new();
        let scale = fractal.scale();
        fractal.pan(1.0, 1.0);
        fractal.zoom(1.5);
        assert!(fractal.x() > 0.0 && fractal.y() > 0.0);
        assert!(fractal.scale() < scale);
        fractal.zoom(1000.0);
        assert_eq!(fractal.scale(), MIN_SCALE);
        fractal.reset();
        assert_eq!(fractal.scale(), DEFAULT_SCALE);
    }

    #[test]
    fn frames_throttle_and_rotate() {
        let mut fractal = Fractal::new();
        assert!(!fractal.advance(10));
        assert!(fractal.advance(100));
        assert!(fractal.phase() > 0.0);
        assert!(fractal.scale() < DEFAULT_SCALE);
    }
}
