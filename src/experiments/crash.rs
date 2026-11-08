//! Optional, inactive display-only Crash study. Enable the `crash-study` feature
//! to compile this module; it is intentionally not wired into the portfolio UI.
use ratatui::style::Color;
const EGA: [Color; 16] = [
    Color::Rgb(0, 0, 0),
    Color::Rgb(0, 0, 170),
    Color::Rgb(0, 170, 0),
    Color::Rgb(0, 170, 170),
    Color::Rgb(170, 0, 0),
    Color::Rgb(170, 0, 170),
    Color::Rgb(170, 85, 0),
    Color::Rgb(170, 170, 170),
    Color::Rgb(85, 85, 85),
    Color::Rgb(85, 85, 255),
    Color::Rgb(85, 255, 85),
    Color::Rgb(85, 255, 255),
    Color::Rgb(255, 85, 85),
    Color::Rgb(255, 85, 255),
    Color::Rgb(255, 255, 85),
    Color::Rgb(255, 255, 255),
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

/// Visual video-memory byte sweep; not an executable or a virus.
#[derive(Clone)]
pub struct CrashStudy {
    video: Vec<u8>,
    cursor: u16,
}
impl Default for CrashStudy {
    fn default() -> Self {
        let mut s = Self {
            video: vec![0; 0x3e80],
            cursor: 0,
        };
        for i in 0..80 * 25 {
            s.video[i * 2] = b' ';
            s.video[i * 2 + 1] = 7;
        }
        s
    }
}
impl CrashStudy {
    pub fn advance(&mut self, updates: usize) {
        for _ in 0..updates {
            self.cursor += 0x51d;
            if self.cursor >= 0x3e80 {
                self.cursor -= 0x3e80;
            }
            let i = self.cursor as usize;
            self.video[i] = self.video[i].wrapping_add(self.cursor as u8);
        }
    }
    pub fn cell(&self, x: usize, y: usize) -> (char, Color, Color) {
        let i = ((y % 25) * 80 + x % 80) * 2;
        let byte = self.video[i];
        let attr = self.video[i + 1];
        let glyph = if (32..=126).contains(&byte) || (128..=223).contains(&byte) {
            CP437[byte as usize]
        } else {
            ' '
        };
        (
            glyph,
            EGA[(attr & 15) as usize],
            EGA[((attr >> 4) & 7) as usize],
        )
    }
}
