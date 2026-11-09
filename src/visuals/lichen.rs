//! A grayscale cellular automaton: a slowly growing lichen colony.
//!
//! The grid holds an energy value per cell. Each generation decays random
//! occupied cells and lets their orthogonal neighbours inherit the maximum
//! energy, which produces the spreading, coral-like growth. Rendering lives in
//! `ui::visuals` for terminals and in `frontends::web` for the browser canvas.

/// Grid the colony starts with; it is resized to the viewport when opened.
const DEFAULT_WIDTH: usize = 320;
const DEFAULT_HEIGHT: usize = 200;

/// Energy of a freshly seeded or newly inherited cell.
const MAX_ENERGY: u8 = 30;

/// Cells below this energy stop spreading and fade out.
const SPREAD_THRESHOLD: u8 = 23;

/// Cell updates per generation on the default grid, scaled with the area.
const UPDATES_PER_GENERATION: u64 = 45_000;

/// One spore is added every `SEED_INTERVAL` random draws.
const SEED_INTERVAL: u32 = 240;

/// Display divisor: keeps the colony dim so it reads as a background texture.
const DISPLAY_DIVISOR: u16 = 39;

/// A lichen colony with its own deterministic random source.
#[derive(Debug, Clone)]
pub struct Lichen {
    width: usize,
    height: usize,
    energy: Vec<u8>,
    rng: u64,
    ticks: u64,
}

impl Lichen {
    /// A blank colony; `seed` makes a session reproducible.
    pub fn new(seed: u64) -> Self {
        Self {
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            energy: vec![0; DEFAULT_WIDTH * DEFAULT_HEIGHT],
            rng: seed | 1,
            ticks: 0,
        }
    }

    /// Current grid size in cells.
    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    /// Brightness of one cell as an 8-bit grey level.
    ///
    /// `index` must be within the grid, in row-major order.
    pub fn display_value(&self, index: usize) -> u8 {
        (self.energy[index] as u16 * 255 / DISPLAY_DIVISOR) as u8
    }

    /// Grow or shrink the grid, keeping the existing colony centred.
    pub fn resize(&mut self, width: usize, height: usize) {
        let (width, height) = (width.max(1), height.max(1));
        if (width, height) == (self.width, self.height) {
            return;
        }
        let mut resized = vec![0; width * height];
        let copied_width = width.min(self.width);
        let copied_height = height.min(self.height);
        let source_x = (self.width - copied_width) / 2;
        let source_y = (self.height - copied_height) / 2;
        let target_x = (width - copied_width) / 2;
        let target_y = (height - copied_height) / 2;
        for row in 0..copied_height {
            let from = (source_y + row) * self.width + source_x;
            let to = (target_y + row) * width + target_x;
            resized[to..to + copied_width].copy_from_slice(&self.energy[from..from + copied_width]);
        }
        self.width = width;
        self.height = height;
        self.energy = resized;
    }

    /// Clear the colony and restart the random sequence.
    pub fn reset(&mut self, seed: u64) {
        self.rng = seed | 1;
        self.ticks = 0;
        self.energy.fill(0);
    }

    /// Advance the colony by one generation.
    pub fn step(&mut self) {
        self.ticks += 1;
        if self.ticks == 1 || self.next_u32().is_multiple_of(SEED_INTERVAL) {
            let index = self.next_u32() as usize % self.energy.len();
            self.energy[index] = MAX_ENERGY;
        }
        for _ in 0..self.update_count() {
            let index = self.next_u32() as usize % self.energy.len();
            if self.energy[index] == 0 {
                continue;
            }
            self.energy[index] -= 1;
            if self.energy[index] < SPREAD_THRESHOLD {
                continue;
            }
            let (x, y) = (index % self.width, index / self.width);
            if x > 0 {
                self.spread(index - 1);
            }
            if x + 1 < self.width {
                self.spread(index + 1);
            }
            if y > 0 {
                self.spread(index - self.width);
            }
            if y + 1 < self.height {
                self.spread(index + self.width);
            }
        }
    }

    /// Let an empty neighbour inherit the maximum energy.
    fn spread(&mut self, index: usize) {
        if self.energy[index] == 0 {
            self.energy[index] = MAX_ENERGY;
        }
    }

    /// Number of random cell updates in one generation.
    fn update_count(&self) -> usize {
        let area = self.energy.len() as u64;
        (UPDATES_PER_GENERATION * area / (DEFAULT_WIDTH * DEFAULT_HEIGHT) as u64) as usize
    }

    /// xorshift32; the colony only needs cheap, deterministic noise.
    fn next_u32(&mut self) -> u32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        (self.rng >> 16) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resize_keeps_the_grid_consistent() {
        let mut lichen = Lichen::new(4);
        lichen.resize(1, 1);
        lichen.step();
        lichen.reset(9);
        lichen.resize(500, 300);
        lichen.step();
        assert_eq!(lichen.size(), (500, 300));
    }

    #[test]
    fn the_colony_grows_and_stays_dim() {
        let mut lichen = Lichen::new(41);
        for _ in 0..50 {
            lichen.step();
        }
        let (width, height) = lichen.size();
        let values = (0..width * height).map(|index| lichen.display_value(index));
        let brightest = (MAX_ENERGY as u16 * 255 / DISPLAY_DIVISOR) as u8;
        assert!(values.clone().any(|value| value > 0), "colony grows");
        assert!(values.into_iter().all(|value| value <= brightest));
    }

    #[test]
    fn reset_clears_the_colony() {
        let mut lichen = Lichen::new(7);
        lichen.step();
        lichen.reset(7);
        let (width, height) = lichen.size();
        assert!((0..width * height).all(|index| lichen.display_value(index) == 0));
    }
}
