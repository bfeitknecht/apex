//! The top navigation bar: labels, hit ranges and their terminal widths.
//!
//! The three section names belong to the app. The destinations after them come
//! from `content`, so a renamed link in `content/profile.toml` renames the top
//! bar too.

use crate::content;

/// Section label sets, widest first.
const SECTIONS: [[&str; 3]; 4] = [
    ["about", "projects", "contact"],
    ["about", "proj", "cont"],
    ["abt", "prj", "cnt"],
    ["a", "p", "c"],
];

/// How a destination label (`cv`, `blog`, `gallery`) is shortened.
#[derive(Clone, Copy)]
enum Destination {
    /// `(gallery)`
    Full,
    /// `(gal)`: labels longer than four characters keep three, in parentheses.
    Short,
    /// `gal`: no parentheses. The narrowest phones need these three columns.
    Bare,
}

/// The tiers, widest first. A narrower viewport takes the first tier that
/// fits, so words survive as long as the width allows and `(c)` for cv never
/// sits next to `c` for contact.
const TIERS: [(usize, Destination); 7] = [
    (0, Destination::Full),
    (1, Destination::Full),
    (1, Destination::Short),
    (2, Destination::Short),
    (3, Destination::Short),
    (3, Destination::Bare),
    (3, Destination::Bare),
];

fn tier_labels(sections: usize, destination: Destination) -> Vec<String> {
    let mut labels: Vec<String> = SECTIONS[sections]
        .iter()
        .map(|label| (*label).to_string())
        .collect();
    for link in content::profile().links {
        let short: String = if link.label.chars().count() <= 4 {
            link.label.to_string()
        } else {
            link.label.chars().take(3).collect()
        };
        labels.push(match destination {
            Destination::Full => format!("({})", link.label),
            Destination::Short => format!("({short})"),
            Destination::Bare => short,
        });
    }
    labels
}

/// Every label in the top bar for the inner width `inner` (= width - 4):
/// three sections, then the destinations.
pub(super) fn nav_labels(inner: usize) -> Vec<String> {
    let width = inner + 4;
    for (sections, destination) in TIERS {
        let labels = tier_labels(sections, destination);
        // The bar starts at column 7; every label adds ` | ` after it.
        let end = 7 + labels
            .iter()
            .map(|label| label.chars().count() + 3)
            .sum::<usize>()
            - 2;
        if end < width {
            return labels;
        }
    }
    tier_labels(3, Destination::Bare)
}

/// Start and end columns of every label in the top bar.
pub fn nav_ranges(w: usize) -> Vec<(usize, usize)> {
    let inner = w.saturating_sub(4);
    let mut x = 7;
    nav_labels(inner)
        .iter()
        .map(|label| {
            let start = x;
            x += label.chars().count() + 3;
            (start, start + label.chars().count())
        })
        .collect()
}
