//! Browser Unicode specimen catalogue, built from the bundled fonts' cmaps.
use std::collections::BTreeMap;
use unicode_blocks::{find_unicode_block, UnicodeBlock};
const CANDIDATES: &[u8] = include_bytes!("../assets/unifont-cmap.bin");
const NOTO: &[u8] = include_bytes!("../assets/noto-cmap.bin");

pub struct UnicodeStudy {
    groups: Vec<Vec<char>>,
    history: Vec<char>,
    cursor: usize,
    display_override: Option<char>,
}
impl Default for UnicodeStudy {
    fn default() -> Self {
        Self::new()
    }
}
impl UnicodeStudy {
    pub fn new() -> Self {
        let mut grouped: BTreeMap<u32, Vec<char>> = BTreeMap::new();
        for &bytes in CANDIDATES.as_chunks::<4>().0 {
            let code = u32::from_le_bytes(bytes);
            if let Some(ch) = char::from_u32(code) {
                if let Some(block) = find_unicode_block(ch) {
                    grouped.entry(block.start()).or_default().push(ch);
                }
            }
        }
        let mut s = Self {
            groups: grouped.into_values().collect(),
            history: Vec::new(),
            cursor: 0,
            display_override: None,
        };
        s.new_random();
        s
    }
    fn random(n: usize) -> usize {
        (js_sys::Math::random() * n as f64) as usize
    }
    fn new_random(&mut self) {
        let group = Self::random(self.groups.len());
        let chars = &self.groups[group];
        let next = chars[Self::random(chars.len())];
        self.history.push(next);
        self.cursor = self.history.len() - 1;
    }
    pub fn next(&mut self) {
        if self.cursor + 1 < self.history.len() {
            self.cursor += 1
        } else {
            self.new_random()
        }
    }
    pub fn previous(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }
    pub fn shuffle(&mut self) {
        self.history.truncate(self.cursor + 1);
        self.new_random();
    }
    pub fn font_family(&self) -> &'static str {
        let code = self.glyph() as u32;
        let mut lo = 0;
        let mut hi = NOTO.len() / 5;
        while lo < hi {
            let mid = (lo + hi) / 2;
            let i = mid * 5;
            let value = u32::from_le_bytes([NOTO[i], NOTO[i + 1], NOTO[i + 2], NOTO[i + 3]]);
            if value < code {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        let i = lo * 5;
        if i + 5 <= NOTO.len()
            && u32::from_le_bytes([NOTO[i], NOTO[i + 1], NOTO[i + 2], NOTO[i + 3]]) == code
        {
            match NOTO[i + 4] {
                1 => "NotoStudy",
                2 => "NotoSymbols",
                3 => "NotoSymbols2",
                4 => "NotoMath",
                5 => "NotoSansArabic",
                6 => "NotoSansArmenian",
                7 => "NotoSansBamum",
                8 => "NotoSansBengali",
                9 => "NotoSansCanadianAboriginal",
                10 => "NotoSansCherokee",
                11 => "NotoSansCuneiform",
                12 => "NotoSansDevanagari",
                13 => "NotoSansEgyptianHieroglyphs",
                14 => "NotoSansEthiopic",
                15 => "NotoSansGeorgian",
                16 => "NotoSansGlagolitic",
                17 => "NotoSansGothic",
                18 => "NotoSansGujarati",
                19 => "NotoSansGurmukhi",
                20 => "NotoSansHebrew",
                21 => "NotoSansKhmer",
                22 => "NotoSansLao",
                23 => "NotoSansLinearB",
                24 => "NotoSansMalayalam",
                25 => "NotoSansMongolian",
                26 => "NotoSansMyanmar",
                27 => "NotoSansOldItalic",
                28 => "NotoSansOldPersian",
                29 => "NotoSansOsmanya",
                30 => "NotoSansRunic",
                31 => "NotoSansSinhala",
                32 => "NotoSansSyriac",
                33 => "NotoSansTamil",
                34 => "NotoSansThaana",
                35 => "NotoSansThai",
                36 => "NotoSansUgaritic",
                37 => "NotoSansYi",
                38 => "BabelStoneHan",
                _ => "ApexGlyphBMP",
            }
        } else if code < 0x10000 {
            "ApexGlyphBMP"
        } else {
            "ApexGlyphUpper"
        }
    }
    pub fn glyph(&self) -> char {
        self.display_override.unwrap_or(self.history[self.cursor])
    }
    pub fn set_display_override(&mut self, glyph: Option<char>) {
        self.display_override = glyph;
    }
    pub fn name(&self) -> String {
        unicode_names2::name(self.glyph())
            .map(|n| n.to_string())
            .unwrap_or_else(|| "NAME UNAVAILABLE".into())
    }
    pub fn block(&self) -> Option<UnicodeBlock> {
        find_unicode_block(self.glyph())
    }
}
