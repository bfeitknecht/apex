//! Unicode specimen catalogue, shared by the terminal and the browser.
//!
//! The visual picks a block first and a code point second, so it moves across
//! scripts. Two bundled tables drive it, and `just cmaps` writes both:
//!
//! - `assets/unifont-cmap.bin`: little-endian `u32` code points that have a glyph
//!   in at least one bundled font.
//! - `assets/noto-cmap.bin`: five-byte records of a little-endian `u32` code
//!   point followed by the id of the font that covers it.

use std::collections::BTreeMap;

use unicode_blocks::{find_unicode_block, UnicodeBlock};

const CANDIDATES: &[u8] = include_bytes!("../../assets/unifont-cmap.bin");
const NOTO: &[u8] = include_bytes!("../../assets/noto-cmap.bin");

/// Length of one record in [`NOTO`].
const NOTO_RECORD: usize = 5;

/// Family id used for code points that fall back to the bitmap fonts.
const BITMAP_BMP: &str = "ApexGlyphBMP";
const BITMAP_UPPER: &str = "ApexGlyphUpper";

/// Font families by id, in the order `just cmaps` writes them.
/// The browser resolves these names through `@font-face` rules in `browser/app.css`.
const NOTO_FAMILIES: [&str; 39] = [
    "",
    "NotoStudy",
    "NotoSymbols",
    "NotoSymbols2",
    "NotoMath",
    "NotoSansArabic",
    "NotoSansArmenian",
    "NotoSansBamum",
    "NotoSansBengali",
    "NotoSansCanadianAboriginal",
    "NotoSansCherokee",
    "NotoSansCuneiform",
    "NotoSansDevanagari",
    "NotoSansEgyptianHieroglyphs",
    "NotoSansEthiopic",
    "NotoSansGeorgian",
    "NotoSansGlagolitic",
    "NotoSansGothic",
    "NotoSansGujarati",
    "NotoSansGurmukhi",
    "NotoSansHebrew",
    "NotoSansKhmer",
    "NotoSansLao",
    "NotoSansLinearB",
    "NotoSansMalayalam",
    "NotoSansMongolian",
    "NotoSansMyanmar",
    "NotoSansOldItalic",
    "NotoSansOldPersian",
    "NotoSansOsmanya",
    "NotoSansRunic",
    "NotoSansSinhala",
    "NotoSansSyriac",
    "NotoSansTamil",
    "NotoSansThaana",
    "NotoSansThai",
    "NotoSansUgaritic",
    "NotoSansYi",
    "BabelStoneHan",
];

/// Limit of the browsing history, so a long session does not grow without bound.
const MAX_HISTORY: usize = 512;

/// A catalogue of specimens plus the visitor's browsing history.
#[derive(Debug)]
pub struct Unicode {
    /// Candidate code points, grouped by the start of their Unicode block.
    groups: Vec<Vec<char>>,
    history: Vec<char>,
    cursor: usize,
    /// Shown instead of the current specimen while a font loads.
    pinned: Option<char>,
    rng: u64,
}

impl Unicode {
    /// Build the catalogue and show the first specimen. Parsing the tables takes
    /// a few milliseconds, so callers build the visual on first use.
    pub fn new(seed: u64) -> Self {
        let mut grouped: BTreeMap<u32, Vec<char>> = BTreeMap::new();
        for record in CANDIDATES.as_chunks::<4>().0 {
            let Some(glyph) = char::from_u32(u32::from_le_bytes(*record)) else {
                continue;
            };
            if let Some(block) = find_unicode_block(glyph) {
                grouped.entry(block.start()).or_default().push(glyph);
            }
        }
        if grouped.is_empty() {
            grouped.insert(0, vec!['\u{fffd}']);
        }
        let mut visual = Self {
            groups: grouped.into_values().collect(),
            history: Vec::new(),
            cursor: 0,
            pinned: None,
            rng: seed | 1,
        };
        visual.push_specimen();
        visual
    }

    /// Move forward, sampling a new specimen at the end of the history.
    pub fn next(&mut self) {
        if self.cursor + 1 < self.history.len() {
            self.cursor += 1;
        } else {
            self.push_specimen();
        }
    }

    /// Move back, stopping at the oldest specimen still in the history.
    pub fn previous(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    /// Drop the history after the cursor and sample a fresh specimen.
    pub fn shuffle(&mut self) {
        self.history.truncate(self.cursor + 1);
        self.push_specimen();
    }

    /// The specimen that belongs on screen.
    pub fn glyph(&self) -> char {
        // `pinned` only exists while the replacement's font is loading, and the
        // history always holds at least one specimen.
        self.pinned.unwrap_or(self.history[self.cursor])
    }

    /// Show `glyph` instead of the current specimen until it is cleared.
    pub fn pin_glyph(&mut self, glyph: Option<char>) {
        self.pinned = glyph;
    }

    /// The official Unicode name, or a placeholder when the table has no entry.
    pub fn name(&self) -> String {
        unicode_names2::name(self.glyph())
            .map_or_else(|| "NAME UNAVAILABLE".to_string(), |name| name.to_string())
    }

    /// The Unicode block that contains the specimen.
    pub fn block(&self) -> Option<UnicodeBlock> {
        find_unicode_block(self.glyph())
    }

    /// The bundled font family that covers the specimen.
    pub fn font_family(&self) -> &'static str {
        let code = self.glyph() as u32;
        match noto_family(code) {
            Some(id) => family_name(id),
            None if code < 0x1_0000 => BITMAP_BMP,
            None => BITMAP_UPPER,
        }
    }

    /// Append a specimen, letting the oldest one fall out of a full history.
    fn push_specimen(&mut self) {
        if self.history.len() >= MAX_HISTORY {
            self.history.remove(0);
            self.cursor = self.cursor.saturating_sub(1);
        }
        let glyph = self.sample();
        self.history.push(glyph);
        self.cursor = self.history.len() - 1;
    }

    /// Pick a random block, then a random specimen inside it.
    fn sample(&mut self) -> char {
        let group = self.random_index(self.groups.len());
        let member = self.random_index(self.groups[group].len());
        self.groups[group][member]
    }

    fn random_index(&mut self, len: usize) -> usize {
        (self.next_u32() as usize) % len.max(1)
    }

    /// xorshift64; the visual only needs cheap, deterministic noise.
    fn next_u32(&mut self) -> u32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        (self.rng >> 32) as u32
    }
}

/// Binary search the font table for `code`.
fn noto_family(code: u32) -> Option<u8> {
    let mut low = 0;
    let mut high = NOTO.len() / NOTO_RECORD;
    while low < high {
        let middle = (low + high) / 2;
        let value = record_at(middle);
        if value < code {
            low = middle + 1;
        } else {
            high = middle;
        }
    }
    (record_at(low) == code).then(|| NOTO[low * NOTO_RECORD + 4])
}

/// The code point of the record at `index`, or `u32::MAX` past the end.
fn record_at(index: usize) -> u32 {
    let at = index * NOTO_RECORD;
    match NOTO.get(at..at + 4) {
        Some(bytes) => u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        None => u32::MAX,
    }
}

fn family_name(id: u8) -> &'static str {
    NOTO_FAMILIES
        .get(id as usize)
        .copied()
        .filter(|name| !name.is_empty())
        .unwrap_or(BITMAP_BMP)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_font_id_has_a_family_name() {
        assert!(NOTO_FAMILIES.iter().skip(1).all(|name| !name.is_empty()));
        assert_eq!(family_name(1), "NotoStudy");
        assert_eq!(family_name(38), "BabelStoneHan");
        assert_eq!(family_name(0), BITMAP_BMP);
    }

    #[test]
    fn seeded_studies_open_with_the_same_specimen() {
        let first = Unicode::new(7);
        let second = Unicode::new(7);
        assert_eq!(first.glyph(), second.glyph());
        assert!(!first.name().is_empty());
    }

    #[test]
    fn history_walks_back_to_the_first_specimen() {
        let mut visual = Unicode::new(11);
        let first = visual.glyph();
        visual.next();
        visual.next();
        visual.previous();
        visual.previous();
        visual.previous();
        assert_eq!(visual.glyph(), first);
        assert_eq!(visual.cursor, 0);
    }

    #[test]
    fn pinned_glyph_wins_until_cleared() {
        let mut visual = Unicode::new(3);
        let first = visual.glyph();
        visual.next();
        visual.pin_glyph(Some(first));
        assert_eq!(visual.glyph(), first);
        visual.pin_glyph(None);
        assert_eq!(visual.glyph(), visual.history[visual.cursor]);
    }

    #[test]
    fn shuffle_drops_the_history_after_the_cursor() {
        let mut visual = Unicode::new(5);
        for _ in 0..5 {
            visual.next();
        }
        visual.previous();
        visual.previous();
        visual.shuffle();
        assert_eq!(visual.cursor, 4);
        assert_eq!(visual.history.len(), 5);
    }

    #[test]
    fn history_is_bounded() {
        let mut visual = Unicode::new(9);
        for _ in 0..MAX_HISTORY * 2 {
            visual.next();
        }
        assert_eq!(visual.history.len(), MAX_HISTORY);
        assert_eq!(visual.cursor, MAX_HISTORY - 1);
    }

    #[test]
    fn specimens_are_covered_by_a_bundled_font() {
        let mut visual = Unicode::new(13);
        for _ in 0..64 {
            let family = visual.font_family();
            assert!(
                NOTO_FAMILIES.contains(&family) || family == BITMAP_BMP || family == BITMAP_UPPER,
                "{family} has no @font-face rule"
            );
            visual.next();
        }
    }
}
