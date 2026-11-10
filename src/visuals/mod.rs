//! Generative visuals: simulation without rendering.
//!
//! Every type here advances its own state and exposes plain data: glyphs,
//! palette indices and rows of characters. `ui` decides how a terminal draws that
//! data, and the browser decides how a canvas does. This module knows nothing
//! about ratatui, crossterm or the DOM. The visuals are therefore testable on any
//! target and cheap to run inside an SSH session.

pub mod crash;
pub mod fractal;
pub mod lichen;
pub mod unicode;

pub use crash::Crash;
pub use fractal::Fractal;
pub use lichen::Lichen;
pub use unicode::Unicode;

/// Which visual the hidden menu is pointing at.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Visual {
    Unicode,
    Lichen,
    Fractal,
    Crash,
}

impl Visual {
    /// The label shown in the hidden menu.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Unicode => "(unicode)",
            Self::Lichen => "(lichen)",
            Self::Fractal => "(fractal)",
            Self::Crash => "(crash)",
        }
    }
}

/// The menu, in order. One list decides the labels and the index mapping.
pub const VISUALS: [Visual; 4] = [
    Visual::Unicode,
    Visual::Lichen,
    Visual::Fractal,
    Visual::Crash,
];
