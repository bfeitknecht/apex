//! The top navigation bar: labels, hit ranges and their terminal widths.
//!
//! The three section names belong to the dynamic version. The destinations after them come
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

/// How a destination label (`cv`, `blog`, `gallery`) is shortened. Labels of
/// four characters or fewer (`cv`, `blog`) never change.
#[derive(Clone, Copy)]
enum Destination {
    /// `(gallery)`
    Full,
    /// `(glry)`: the first letter, then the consonants, repeats dropped.
    Short,
    /// `glry`: no parentheses.
    Bare,
    /// `g`: the first letter. The narrowest phones need these columns.
    Initial,
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
    (3, Destination::Initial),
];

/// `gallery` -> `glry`: keep the first letter, drop the vowels after it and
/// collapse repeated letters, at most four characters.
fn abbreviate(label: &str) -> String {
    let mut chars = label.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    let mut out = vec![first];
    for c in chars {
        if !"aeiou".contains(c) && out.last() != Some(&c) {
            out.push(c);
        }
    }
    out.into_iter().take(4).collect()
}

fn tier_labels(sections: usize, destination: Destination) -> Vec<String> {
    let mut labels: Vec<String> = SECTIONS[sections]
        .iter()
        .map(|label| (*label).to_string())
        .collect();
    for link in content::profile().links {
        let label = link.label;
        let fits = label.chars().count() <= 4;
        labels.push(match destination {
            Destination::Full => format!("({label})"),
            _ if fits => match destination {
                Destination::Short => format!("({label})"),
                _ => label.to_string(),
            },
            Destination::Short => format!("({})", abbreviate(label)),
            Destination::Bare => abbreviate(label),
            Destination::Initial => label.chars().take(1).collect(),
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
    tier_labels(3, Destination::Initial)
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

/// Whether top-bar entry `nav` lights up for portfolio section `section`:
/// the three section names for their own sections, (cv) for "hire me", and
/// the other destinations (blog, gallery) together for "click them".
pub(super) fn nav_highlights(section: usize, nav: usize) -> bool {
    if nav < 3 {
        return nav == section;
    }
    let Some(link) = content::profile().links.get(nav - 3) else {
        return false;
    };
    match section {
        3 => link.label == "cv",
        4 => link.label != "cv",
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn destinations_shorten_to_consonants_then_initial() {
        assert_eq!(abbreviate("gallery"), "glry");
        let short = tier_labels(3, Destination::Short);
        let bare = tier_labels(3, Destination::Bare);
        let initial = tier_labels(3, Destination::Initial);
        assert!(short.contains(&"(glry)".to_string()));
        assert!(short.contains(&"(blog)".to_string()));
        assert!(bare.contains(&"glry".to_string()));
        assert!(initial.contains(&"g".to_string()));
        assert!(initial.contains(&"blog".to_string()));
        assert!(initial.contains(&"cv".to_string()));
    }

    #[test]
    fn sections_light_up_their_top_bar_entries() {
        let labels: Vec<&str> = content::profile().links.iter().map(|l| l.label).collect();
        let lit = |section: usize| -> Vec<usize> {
            (0..3 + labels.len())
                .filter(|&nav| nav_highlights(section, nav))
                .collect()
        };
        assert_eq!(lit(0), [0]);
        assert_eq!(lit(2), [2]);
        let cv = 3 + labels.iter().position(|l| *l == "cv").unwrap();
        assert_eq!(lit(3), [cv]);
        let others: Vec<usize> = (0..labels.len())
            .filter(|&i| labels[i] != "cv")
            .map(|i| 3 + i)
            .collect();
        assert_eq!(lit(4), others);
        assert!(lit(5).is_empty(), "visuals menu lights nothing");
    }
}
