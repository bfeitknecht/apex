//! The top navigation bar: labels, hit ranges and their terminal widths.
//!
//! The three section names are the app's own; the destinations after them come
//! from `content`, so renaming a link in `content/profile.toml` renames the
//! top bar too.

use crate::content;

/// Section labels, shortened as the viewport narrows.
fn section_labels(inner: usize) -> [&'static str; 3] {
    if inner < 48 {
        ["a", "p", "c"]
    } else if inner < 62 {
        ["about", "proj", "cont"]
    } else {
        ["about", "projects", "contact"]
    }
}

/// Every label in the top bar: three sections, then the destinations.
pub(super) fn nav_labels(inner: usize) -> Vec<String> {
    let mut labels: Vec<String> = section_labels(inner)
        .iter()
        .map(|label| (*label).to_string())
        .collect();
    let abbreviated = inner < 48;
    for link in content::profile().links {
        let label = if abbreviated {
            link.label.chars().take(1).collect()
        } else {
            link.label.to_string()
        };
        labels.push(format!("({label})"));
    }
    labels
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
