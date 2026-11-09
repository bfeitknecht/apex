//! The portfolio page: its text layout model and its rendering.
//!
//! The layout is built from `content` for a given viewport, which makes link
//! hit testing, scrolling and rendering share one source of truth.

use ratatui::style::Modifier;
use ratatui::Frame;

use super::nav::{nav_labels, nav_ranges};
use super::theme::{put, put_selected, text_style, DIM, MUTED, WHITE};
use crate::app::App;
use crate::content;

pub fn wrap_lines(text: &str, width: usize) -> Vec<String> {
    let width = width.max(1);
    let mut result = Vec::new();
    for paragraph in text.split('\n') {
        let mut line = String::new();
        for word in paragraph.split_whitespace() {
            let mut chars = word.chars().peekable();
            if !line.is_empty() && line.chars().count() + 1 + word.chars().count() > width {
                result.push(std::mem::take(&mut line));
            }
            if !line.is_empty() {
                line.push(' ');
            }
            while let Some(c) = chars.next() {
                line.push(c);
                if line.chars().count() == width && chars.peek().is_some() {
                    result.push(std::mem::take(&mut line));
                }
            }
        }
        result.push(line);
    }
    result
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkKind {
    External,
    Age,
    Pgp,
    Me,
}
#[derive(Clone, Debug)]
pub struct PageLink {
    pub row: usize,
    pub start: usize,
    pub end: usize,
    pub url: &'static str,
    pub kind: LinkKind,
}
#[derive(Clone, Debug)]
pub struct PageRow {
    pub text: String,
    pub heading: bool,
}
pub struct PortfolioPage {
    pub rows: Vec<PageRow>,
    pub offsets: [usize; 3],
    pub links: Vec<PageLink>,
    pub footer_start: usize,
    pub viewport: usize,
}
impl PortfolioPage {
    pub fn max_scroll(&self) -> usize {
        self.rows.len().saturating_sub(self.viewport)
    }
    pub fn active(&self, scroll: usize) -> usize {
        self.offsets
            .iter()
            .rposition(|&offset| offset <= scroll)
            .unwrap_or(0)
    }
}
pub fn portfolio_page(width: usize, height: usize) -> PortfolioPage {
    let viewport = height.saturating_sub(4).max(1);
    let content_width = width.saturating_sub(12).max(1);
    let mut rows = Vec::<PageRow>::new();
    let mut links = Vec::<PageLink>::new();
    let mut offsets = [0; 3];
    fn line(rows: &mut Vec<PageRow>, text: impl Into<String>, heading: bool) {
        rows.push(PageRow {
            text: text.into(),
            heading,
        });
    }
    fn link(
        rows: &mut Vec<PageRow>,
        links: &mut Vec<PageLink>,
        label: String,
        url: &'static str,
        start: usize,
    ) {
        let row = rows.len();
        let end = start + label.chars().count();
        line(rows, format!("{}{}", " ".repeat(start), label), false);
        if !url.is_empty() {
            links.push(PageLink {
                row,
                start,
                end,
                url,
                kind: LinkKind::External,
            });
        }
    }
    let profile = content::profile();
    offsets[0] = rows.len();
    line(&mut rows, "= 0xbf.ch", true);
    line(&mut rows, "", false);
    // The terminal reads the shared copy in its own voice: lowercase.
    for text in wrap_lines(&profile.lede.to_lowercase(), content_width) {
        line(&mut rows, text, false);
    }
    for text in profile.intro {
        for wrapped in wrap_lines(text, content_width) {
            line(&mut rows, wrapped, false);
        }
    }
    line(&mut rows, "", false);
    line(
        &mut rows,
        format!("== {}!", profile.eyebrow.to_lowercase()),
        true,
    );
    line(&mut rows, "", false);
    if !profile.about_terminal.is_empty() {
        for text in wrap_lines(profile.about_terminal, content_width) {
            line(&mut rows, text, false);
        }
    }
    line(&mut rows, "", false);
    line(&mut rows, "", false);
    offsets[1] = rows.len();
    line(&mut rows, "== cool stuff", true);
    line(&mut rows, "", false);
    for (i, project) in profile.projects.iter().enumerate() {
        let description = if project.summary.is_empty() {
            String::new()
        } else {
            format!(" - {}", project.summary)
        };
        let prefix = format!("#{:02}  ", i + 1);
        let text = format!("{}{} \\", project.name, description);
        for (part_index, part) in
            wrap_lines(&text, content_width.saturating_sub(prefix.len()).max(1))
                .into_iter()
                .enumerate()
        {
            let part = format!(
                "{}{}",
                if part_index == 0 {
                    prefix.as_str()
                } else {
                    "     "
                },
                part
            );
            let row = rows.len();
            let end = part.chars().count();
            line(&mut rows, part, false);
            if !project.url.is_empty() {
                links.push(PageLink {
                    row,
                    start: 0,
                    end,
                    url: project.url,
                    kind: LinkKind::External,
                });
            }
        }
    }
    line(&mut rows, "", false);
    line(&mut rows, "", false);
    offsets[2] = rows.len();
    line(&mut rows, "== get in touch", true);
    line(&mut rows, "", false);
    for contact in profile.contacts {
        link(
            &mut rows,
            &mut links,
            format!("{} \\", contact.label),
            contact.url,
            0,
        );
    }
    let footer_rows = if content_width >= 46 { 3 } else { 4 };
    // Keep every section jumpable to the top, even while the portfolio is sparse.
    let target = (offsets[2] + viewport).max(rows.len() + footer_rows);
    while rows.len() + footer_rows < target {
        line(&mut rows, "", false);
    }
    let footer_start = rows.len();
    line(&mut rows, "", false); // footer divider
    let row = rows.len();
    if content_width >= 46 {
        line(
            &mut rows,
            "age.txt  |  pgp.txt  |  made with <3 by me.  |",
            false,
        );
    } else {
        line(&mut rows, "age.txt  |  pgp.txt  |", false);
        line(&mut rows, "made with <3 by me.  |", false);
    }
    line(&mut rows, "", false); // empty final line
    links.push(PageLink {
        row,
        start: 0,
        end: 7,
        url: content::AGE_URL,
        kind: LinkKind::Age,
    });
    links.push(PageLink {
        row,
        start: 12,
        end: 19,
        url: content::PGP_URL,
        kind: LinkKind::Pgp,
    });
    let me_row = if content_width >= 46 { row } else { row + 1 };
    if let Some(start) = rows[me_row].text.rfind("me.") {
        links.push(PageLink {
            row: me_row,
            start,
            end: start + 2,
            url: content::ME_URL,
            kind: LinkKind::Me,
        });
    }
    PortfolioPage {
        rows,
        offsets,
        links,
        footer_start,
        viewport,
    }
}
fn header(f: &mut Frame, app: &App) {
    let w = f.area().width as usize;
    let a = f.area();
    if a.width > 3 && a.height > 1 {
        f.buffer_mut().set_stringn(
            a.x + 2,
            a.y + 1,
            "^@",
            2,
            text_style(WHITE).add_modifier(
                Modifier::REVERSED
                    | Modifier::BOLD
                    | if app.focused_nav == Some(0) {
                        Modifier::UNDERLINED
                    } else {
                        Modifier::empty()
                    },
            ),
        );
    }
    put(f, 5, 1, "|", DIM);
    let labels = nav_labels(w.saturating_sub(4));
    let ranges = nav_ranges(w);
    for (i, (start, end)) in ranges.iter().enumerate() {
        put_selected(
            f,
            *start,
            1,
            &labels[i],
            (i < 3 && i == app.nav_index())
                || app.hovered_nav == Some(i)
                || app.focused_nav == Some(i),
        );
        put(f, end + 1, 1, "|", DIM);
    }
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(i) = app.focused_nav.or(app.hovered_nav).filter(|i| *i >= 3) {
        if let Some(link) = i
            .checked_sub(3)
            .and_then(|slot| content::profile().links.get(slot))
        {
            put(f, 8, 3, link.url, MUTED);
        }
    }
    if w > 4 {
        let mut top = vec![b'-'; w.saturating_sub(1)];
        if top.len() > 4 {
            top[4] = b'+';
        }
        for (_, end) in ranges.iter() {
            if *end < top.len() {
                top[*end] = b'+';
            }
        }
        put(f, 1, 2, &String::from_utf8(top).unwrap_or_default(), DIM);
    }
}
pub(super) fn draw(f: &mut Frame, app: &App) {
    let w = f.area().width as usize;
    let h = f.area().height as usize;
    let page = portfolio_page(w, h);
    let scroll = app.scroll_offset.min(page.max_scroll());
    header(f, app);
    for y in 3..h {
        if scroll == page.max_scroll() && y >= 4 && scroll + y - 4 == page.rows.len() - 1 {
            continue;
        }
        put(f, 5, y, "|", DIM);
    }
    for (i, row) in page
        .rows
        .iter()
        .enumerate()
        .skip(scroll)
        .take(page.viewport)
    {
        let y = 4 + i - scroll;
        if i >= page.footer_start && scroll != page.max_scroll() {
            continue;
        }
        if i == page.footer_start {
            let mut line = vec![b'-'; w.saturating_sub(1)];
            for x in [4usize, 16, 28, 52] {
                if x == 52 && w.saturating_sub(12) < 46 {
                    continue;
                }
                if x < line.len() {
                    line[x] = b'+';
                }
            }
            put(f, 1, y, &String::from_utf8(line).unwrap_or_default(), DIM);
            continue;
        }
        put(f, 8, y, &row.text, if row.heading { WHITE } else { MUTED });
        if i > page.footer_start {
            for (column, ch) in row.text.chars().enumerate() {
                if ch == '|' {
                    put(f, 8 + column, y, "|", DIM);
                }
            }
        }
        for selected in [app.focused_link, app.hovered_link].into_iter().flatten() {
            if let Some(link) = page.links.get(selected).filter(|link| link.row == i) {
                let text: String = row
                    .text
                    .chars()
                    .skip(link.start)
                    .take(link.end.saturating_sub(link.start))
                    .collect();
                put_selected(f, 8 + link.start, y, &text, true);
                #[cfg(not(target_arch = "wasm32"))]
                if i < page.footer_start && !link.url.is_empty() {
                    put(f, 8 + link.end + 2, y, link.url, MUTED);
                }
            }
        }
    }
}
