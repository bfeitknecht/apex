//! Crash: an EGA text-mode framebuffer sweeping itself apart.
//!
//! This module only displays. It writes to its own buffer and to nothing else.
//! The visual advances byte by byte through 80x25 text mode, until the attribute
//! bytes describe colours that nobody chose.
//!
//! The visual returns colours as palette indices. `ui` decides what a terminal
//! colour is, so this module uses no rendering types.

/// Milliseconds between sweep steps.
const PERIOD_MS: u64 = 16;

/// The 16 EGA colours, in attribute order.
pub const PALETTE: [(u8, u8, u8); 16] = [
    (0, 0, 0),
    (0, 0, 170),
    (0, 170, 0),
    (0, 170, 170),
    (170, 0, 0),
    (170, 0, 170),
    (170, 85, 0),
    (170, 170, 170),
    (85, 85, 85),
    (85, 85, 255),
    (85, 255, 85),
    (85, 255, 255),
    (255, 85, 85),
    (255, 85, 255),
    (255, 255, 85),
    (255, 255, 255),
];
const CP437: [char; 256] = [
    ' ', '☺', '☻', '♥', '♦', '♣', '♠', '•', '◘', '○', '◙', '♂', '♀', '♪', '♫', '☼', '►', '◄', '↕',
    '‼', '¶', '§', '▬', '↨', '↑', '↓', '→', '←', '∟', '↔', '▲', '▼', ' ', '!', '"', '#', '$', '%',
    '&', '\'', '(', ')', '*', '+', ',', '-', '.', '/', '0', '1', '2', '3', '4', '5', '6', '7', '8',
    '9', ':', ';', '<', '=', '>', '?', '@', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K',
    'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '[', '\\', ']', '^',
    '_', '`', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q',
    'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '{', '|', '}', '~', '⌂', 'Ç', 'ü', 'é', 'â', 'ä',
    'à', 'å', 'ç', 'ê', 'ë', 'è', 'ï', 'î', 'ì', 'Ä', 'Å', 'É', 'æ', 'Æ', 'ô', 'ö', 'ò', 'û', 'ù',
    'ÿ', 'Ö', 'Ü', '¢', '£', '¥', '₧', 'ƒ', 'á', 'í', 'ó', 'ú', 'ñ', 'Ñ', 'ª', 'º', '¿', '⌐', '¬',
    '½', '¼', '¡', '«', '»', '░', '▒', '▓', '│', '┤', '╡', '╢', '╖', '╕', '╣', '║', '╗', '╝', '╜',
    '╛', '┐', '└', '┴', '┬', '├', '─', '┼', '╞', '╟', '╚', '╔', '╩', '╦', '╠', '═', '╬', '╧', '╨',
    '╤', '╥', '╙', '╘', '╒', '╓', '╫', '╪', '┘', '┌', '█', '▄', '▌', '▐', '▀', 'α', 'ß', 'Γ', 'π',
    'Σ', 'σ', 'µ', 'τ', 'Φ', 'Θ', 'Ω', 'δ', '∞', 'φ', 'ε', '∩', '≡', '±', '≥', '≤', '⌠', '⌡', '÷',
    '≈', '°', '∙', '·', '√', 'ⁿ', '²', '■', ' ',
];

/// An 80x25 text-mode framebuffer and the cursor sweeping through it.
#[derive(Clone, Debug)]
pub struct Crash {
    video: Vec<u8>,
    cursor: u16,
    elapsed_ms: u64,
}

/// Size of the emulated 80x25 video memory, character and attribute bytes.
const VIDEO_MEMORY: u16 = 0x3e80;
const COLUMNS: usize = 80;
const ROWS: usize = 25;
/// Bytes written per step: a prime, so the sweep never tiles neatly.
const STRIDE: u16 = 0x51d;

impl Default for Crash {
    fn default() -> Self {
        let mut crash = Self {
            video: vec![0; VIDEO_MEMORY as usize],
            cursor: 0,
            elapsed_ms: 0,
        };
        for cell in 0..COLUMNS * ROWS {
            crash.video[cell * 2] = b' ';
            crash.video[cell * 2 + 1] = 7;
        }
        crash
    }
}

impl Crash {
    pub fn new() -> Self {
        Self::default()
    }

    /// Start over from blank paper.
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Advance by wall-clock time. Returns `true` when a new frame is due.
    ///
    /// The catch-up limit stops a suspended tab or a stalled session from
    /// writing thousands of bytes in the next frame.
    pub fn advance(&mut self, elapsed_ms: u64) -> bool {
        const MAX_CATCH_UP: u64 = 4;
        self.elapsed_ms = self
            .elapsed_ms
            .saturating_add(elapsed_ms)
            .min(PERIOD_MS * MAX_CATCH_UP);
        let steps = self.elapsed_ms / PERIOD_MS;
        self.elapsed_ms %= PERIOD_MS;
        for _ in 0..steps {
            self.step();
        }
        steps > 0
    }

    fn step(&mut self) {
        self.cursor += STRIDE;
        if self.cursor >= VIDEO_MEMORY {
            self.cursor -= VIDEO_MEMORY;
        }
        let index = self.cursor as usize;
        self.video[index] = self.video[index].wrapping_add(self.cursor as u8);
    }

    /// The glyph and the foreground/background palette indices of one cell.
    pub fn cell(&self, x: usize, y: usize) -> (char, u8, u8) {
        let index = ((y % ROWS) * COLUMNS + x % COLUMNS) * 2;
        let byte = self.video[index];
        let attribute = self.video[index + 1];
        let glyph = if (32..=126).contains(&byte) || (128..=223).contains(&byte) {
            CP437[byte as usize]
        } else {
            ' '
        };
        (glyph, attribute & 15, (attribute >> 4) & 7)
    }
}
