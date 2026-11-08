//! A visual-only grayscale cellular automaton without a logo or overlay.
const INITIAL_WIDTH: usize = 320;
const INITIAL_HEIGHT: usize = 200;
#[derive(Debug, Clone)]
pub struct Lichen {
    width: usize,
    height: usize,
    pixels: Vec<u8>,
    rng: u64,
    ticks: u64,
}
impl Lichen {
    pub fn new(seed: u64) -> Self {
        Self {
            width: INITIAL_WIDTH,
            height: INITIAL_HEIGHT,
            pixels: vec![0; INITIAL_WIDTH * INITIAL_HEIGHT],
            rng: seed | 1,
            ticks: 0,
        }
    }
    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }
    pub fn display_value(&self, i: usize) -> u8 {
        (self.pixels[i] as u16 * 255 / 39) as u8
    }
    fn random(&mut self) -> u32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        (self.rng >> 16) as u32
    }
    pub fn resize(&mut self, w: usize, h: usize) {
        let (w, h) = (w.max(1), h.max(1));
        if (w, h) == (self.width, self.height) {
            return;
        }
        let mut next = vec![0; w * h];
        let cw = w.min(self.width);
        let ch = h.min(self.height);
        let (oldx, oldy) = ((self.width - cw) / 2, (self.height - ch) / 2);
        let (newx, newy) = ((w - cw) / 2, (h - ch) / 2);
        for row in 0..ch {
            let src = (oldy + row) * self.width + oldx;
            let dst = (newy + row) * w + newx;
            next[dst..dst + cw].copy_from_slice(&self.pixels[src..src + cw]);
        }
        self.width = w;
        self.height = h;
        self.pixels = next;
    }
    pub fn reset(&mut self, seed: u64) {
        self.rng = seed | 1;
        self.ticks = 0;
        self.pixels.fill(0);
    }
    pub fn step(&mut self) {
        self.ticks += 1;
        if self.ticks == 1 || self.random().is_multiple_of(240) {
            let i = self.random() as usize % self.pixels.len();
            self.pixels[i] = 30;
        }
        let count = ((45_000u64 * self.pixels.len() as u64) / (320 * 200)) as usize;
        for _ in 0..count {
            let i = self.random() as usize % self.pixels.len();
            if self.pixels[i] == 0 {
                continue;
            }
            self.pixels[i] -= 1;
            if self.pixels[i] < 23 {
                continue;
            }
            let x = i % self.width;
            let y = i / self.width;
            let neighbours = [
                if x > 0 { Some(i - 1) } else { None },
                if x + 1 < self.width {
                    Some(i + 1)
                } else {
                    None
                },
                if y > 0 { Some(i - self.width) } else { None },
                if y + 1 < self.height {
                    Some(i + self.width)
                } else {
                    None
                },
            ];
            for n in neighbours.into_iter().flatten() {
                if self.pixels[n] == 0 {
                    self.pixels[n] = 30;
                }
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn resize_and_reset_safe() {
        let mut l = Lichen::new(4);
        l.resize(1, 1);
        l.step();
        l.reset(9);
        l.resize(500, 300);
        l.step();
        assert_eq!(l.pixels.len(), 150_000);
    }
}
