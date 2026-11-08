//! Shared Ratatui portfolio and terminal screensaver rendering.
use crate::{
    app::{App, Document, View},
    portfolio,
};
use ratatui::{
    style::{Color, Modifier, Style},
    Frame,
};
const WHITE: Color = Color::Rgb(235, 235, 235);
const MUTED: Color = Color::Rgb(130, 130, 130);
const DIM: Color = Color::Rgb(75, 75, 75);
// Terminals supply their own foreground/background, including over SSH. Browsers
// use a dark canvas which CSS adapts to the system color scheme.
fn text_style(fg: Color) -> Style {
    #[cfg(target_arch = "wasm32")]
    {
        Style::default().fg(fg).bg(Color::Black)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Style::default()
            .fg(if fg == WHITE { Color::Reset } else { fg })
            .bg(Color::Reset)
    }
}
fn put(f: &mut Frame, x: usize, y: usize, s: &str, fg: Color) {
    let a = f.area();
    if x >= a.width as usize || y >= a.height as usize {
        return;
    }
    f.buffer_mut().set_stringn(
        a.x + x as u16,
        a.y + y as u16,
        s,
        a.width as usize - x,
        text_style(fg),
    );
}
fn put_selected(f: &mut Frame, x: usize, y: usize, s: &str, selected: bool) {
    let a = f.area();
    if x >= a.width as usize || y >= a.height as usize {
        return;
    }
    let mut style = text_style(if selected { WHITE } else { MUTED });
    if selected {
        style = style.add_modifier(Modifier::BOLD);
    }
    f.buffer_mut().set_stringn(
        a.x + x as u16,
        a.y + y as u16,
        s,
        a.width as usize - x,
        style,
    );
}
fn rule(f: &mut Frame, y: usize, w: usize) {
    put(f, 1, y, &"-".repeat(w.saturating_sub(4)), DIM);
}
pub fn nav_ranges(w: usize) -> Vec<(usize, usize)> {
    let inner = w.saturating_sub(4);
    let mut x = 7;
    nav_labels(inner)
        .iter()
        .map(|label| {
            let start = x;
            x += label.len() + 3;
            (start, start + label.len())
        })
        .collect()
}
fn nav_labels(w: usize) -> [&'static str; 6] {
    if w < 33 {
        ["a", "p", "c", "c", "b", "g"]
    } else if w < 43 {
        ["a", "p", "c", "(c)", "(b)", "(g)"]
    } else if w < 48 {
        ["a", "p", "c", "(cv)", "(blog)", "(gallery)"]
    } else if w < 62 {
        ["about", "proj", "cont", "(cv)", "(blog)", "(gallery)"]
    } else {
        [
            "about",
            "projects",
            "contact",
            "(cv)",
            "(blog)",
            "(gallery)",
        ]
    }
}
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
    offsets[0] = rows.len();
    line(&mut rows, "= 0xbf.ch", true);
    line(&mut rows, "", false);
    for text in wrap_lines(portfolio::INTRO, content_width) {
        line(&mut rows, text, false);
    }
    line(&mut rows, "", false);
    line(&mut rows, "== grüezi!", true);
    line(&mut rows, "", false);
    if !portfolio::ABOUT.is_empty() {
        for text in wrap_lines(portfolio::ABOUT, content_width) {
            line(&mut rows, text, false);
        }
    }
    line(&mut rows, "", false);
    line(&mut rows, "", false);
    offsets[1] = rows.len();
    line(&mut rows, "== cool stuff", true);
    line(&mut rows, "", false);
    for (i, project) in portfolio::PROJECTS.iter().enumerate() {
        let description = if project.description.is_empty() {
            String::new()
        } else {
            format!(" - {}", project.description)
        };
        let prefix = format!("#{:02}  ", i + 1);
        let text = format!(r"{}{} \", project.name, description);
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
    for (label, url) in portfolio::CONTACTS {
        link(&mut rows, &mut links, format!(r"{label} \"), url, 0);
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
        url: portfolio::AGE_URL,
        kind: LinkKind::Age,
    });
    links.push(PageLink {
        row,
        start: 12,
        end: 19,
        url: portfolio::PGP_URL,
        kind: LinkKind::Pgp,
    });
    let me_row = if content_width >= 46 { row } else { row + 1 };
    if let Some(start) = rows[me_row].text.rfind("me.") {
        links.push(PageLink {
            row: me_row,
            start,
            end: start + 2,
            url: portfolio::ME_URL,
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
pub const STUDY_LABELS: [&str; 3] = ["(unicode)", "(lichen)", "(fractal)"];
pub fn study_ranges(width: usize, height: usize) -> Vec<(usize, usize, usize)> {
    let total: usize = STUDY_LABELS.iter().map(|s| s.len()).sum::<usize>() + 4;
    if width >= total {
        let mut x = (width - total) / 2;
        STUDY_LABELS
            .iter()
            .map(|label| {
                let r = (x, height / 2, label.len());
                x += label.len() + 2;
                r
            })
            .collect()
    } else {
        let top = height.saturating_sub(3) / 2;
        STUDY_LABELS
            .iter()
            .enumerate()
            .map(|(i, label)| (width.saturating_sub(label.len()) / 2, top + i, label.len()))
            .collect()
    }
}
fn portfolio_header(f: &mut Frame, app: &App) {
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
            labels[i],
            (i < 3 && i == app.nav_index())
                || app.hovered_nav == Some(i)
                || app.focused_nav == Some(i),
        );
        put(f, end + 1, 1, "|", DIM);
    }
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(i) = app.focused_nav.or(app.hovered_nav).filter(|i| *i >= 3) {
        if let Some(url) = portfolio::EXTERNAL_NAV.get(i - 3) {
            put(f, 8, 3, url, MUTED);
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
fn portfolio_view(f: &mut Frame, app: &App) {
    let w = f.area().width as usize;
    let h = f.area().height as usize;
    let page = portfolio_page(w, h);
    let scroll = app.scroll_offset.min(page.max_scroll());
    portfolio_header(f, app);
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
#[cfg(target_arch = "wasm32")]
fn unicode_metadata(
    study: &crate::unicode_study::UnicodeStudy,
    width: usize,
) -> (Vec<String>, usize) {
    let width = width.clamp(1, 100);
    let code = study.glyph() as u32;
    let mut lines = Vec::new();
    fn push_wrapped(lines: &mut Vec<String>, text: &str, width: usize) {
        let mut line = String::new();
        for word in text.split_whitespace() {
            if !line.is_empty() && line.chars().count() + 1 + word.chars().count() > width {
                lines.push(std::mem::take(&mut line));
            }
            if !line.is_empty() {
                line.push(' ');
            }
            for ch in word.chars() {
                if line.chars().count() >= width {
                    lines.push(std::mem::take(&mut line));
                }
                line.push(ch);
            }
        }
        if !line.is_empty() {
            lines.push(line);
        }
    }
    let family = match study.font_family() {
        "NotoStudy" => "NOTO SANS".to_string(),
        "NotoSymbols" => "NOTO SYMBOLS".to_string(),
        "NotoSymbols2" => "NOTO SYMBOLS 2".to_string(),
        "NotoMath" => "NOTO MATH".to_string(),
        "BabelStoneHan" => "BABELSTONE HAN".to_string(),
        family if family.starts_with("NotoSans") => {
            let suffix = family.strip_prefix("NotoSans").unwrap_or(family);
            let mut label = String::from("NOTO SANS ");
            for (i, c) in suffix.chars().enumerate() {
                if i > 0 && c.is_ascii_uppercase() {
                    label.push(' ');
                }
                label.push(c.to_ascii_uppercase());
            }
            label
        }
        _ => "GNU UNIFONT".to_string(),
    };
    let heading = format!("U+{code:04X} {} ({family})", study.name());
    push_wrapped(&mut lines, &heading, width);
    let heading_lines = lines.len();
    if let Some(block) = study.block() {
        lines.push(String::new());
        let bar = |value: u32, lo: u32, hi: u32| {
            let mut b = vec![b'-'; width];
            let pos = ((value.saturating_sub(lo) as u64 * (width - 1) as u64)
                / (hi.saturating_sub(lo).max(1) as u64)) as usize;
            b[pos.min(width - 1)] = b'+';
            String::from_utf8(b).unwrap_or_default()
        };
        lines.push(bar(code, 0, 0x10ffff));
        push_wrapped(&mut lines, "UNICODE U+0000..U+10FFFF", width);
        lines.push(String::new());
        lines.push(bar(code, block.start(), block.end()));
        push_wrapped(
            &mut lines,
            &format!("BLOCK U+{:04X}..U+{:04X}", block.start(), block.end()),
            width,
        );
    }
    (lines, heading_lines)
}
#[cfg(any(target_arch = "wasm32", test))]
fn metadata_foreground(line: usize, heading_lines: usize) -> Color {
    if line < heading_lines {
        WHITE
    } else {
        MUTED
    }
}
#[cfg(target_arch = "wasm32")]
pub struct UnicodeLayout {
    pub metadata_x: usize,
    pub metadata_y: usize,
    pub metadata_width: usize,
    pub glyph_x: usize,
    pub glyph_y: usize,
    pub glyph_width: usize,
    pub glyph_height: usize,
    pub divider: Option<usize>,
}
#[cfg(target_arch = "wasm32")]
pub fn unicode_layout(
    study: &crate::unicode_study::UnicodeStudy,
    width: usize,
    height: usize,
) -> UnicodeLayout {
    let metadata_width = width.saturating_sub(4);
    let (lines, _) = unicode_metadata(study, metadata_width);
    let y = height.saturating_sub(lines.len() + 1).max(6);
    UnicodeLayout {
        metadata_x: 2,
        metadata_y: y,
        metadata_width,
        glyph_x: 2,
        glyph_y: 1,
        glyph_width: width.saturating_sub(4),
        glyph_height: y.saturating_sub(2),
        divider: None,
    }
}
pub fn document_lines(document: Document, width: usize) -> Vec<String> {
    let width = width.saturating_sub(4).max(1);
    let mut output = Vec::new();
    for line in document.text().lines() {
        if line.is_empty() {
            output.push(String::new());
            continue;
        }
        let mut chunk = String::new();
        for ch in line.chars() {
            if chunk.chars().count() == width {
                output.push(std::mem::take(&mut chunk));
            }
            chunk.push(ch);
        }
        output.push(chunk);
    }
    output
}
pub fn document_max_scroll(document: Document, width: usize, height: usize) -> usize {
    document_lines(document, width)
        .len()
        .saturating_sub(height.saturating_sub(6).max(1))
}
pub fn draw(f: &mut Frame, app: &App) {
    let a = f.area();
    let w = a.width as usize;
    let h = a.height as usize;
    f.render_widget(ratatui::widgets::Clear, a);
    if app.help {
        put(f, 1, 1, "apex / keyboard", WHITE);
        rule(f, 3, w);
        for (i, line) in [
            "h / l       previous / next section",
            "j / k       scroll down / up",
            "1 - 5       jump to section",
            "g g / shift g   top / bottom",
            "n / shift n next / previous link",
            "enter       open selected link",
            "esc         back / close help",
            "?           toggle this help",
            "q           quit terminal",
        ]
        .iter()
        .enumerate()
        {
            put(f, 1, 5 + i * 2, line, if i == 0 { WHITE } else { MUTED });
        }
        return;
    }
    if let View::Document(document) = app.view {
        put(f, 2, 1, &format!("== {}", document.title()), WHITE);
        rule(f, 3, w);
        for (row, line) in document_lines(document, w)
            .iter()
            .skip(app.document_scroll)
            .enumerate()
        {
            if row + 6 >= h {
                break;
            }
            put(f, 2, row + 6, line, MUTED);
        }
        return;
    }
    if app.view == View::Study {
        for (i, (x, y, _)) in study_ranges(w, h).iter().enumerate() {
            put_selected(
                f,
                *x,
                *y,
                STUDY_LABELS[i],
                app.study == i || app.hovered_link == Some(i),
            );
        }
        return;
    }
    if app.view == View::Fractal {
        // A moving plane in four-dimensional quaternion space. Iterate the
        // quaternion Julia map q -> q² + c, using an orbit trap to expose
        // fine structure both inside and around the escape boundary.
        let ramp = b".,:;irsXA253hMHGS#9B&@";
        let phase = app.fractal_phase;
        let (sin, cos) = (phase.sin(), phase.cos());
        let slice_z = 0.28 * (phase * 0.73).sin();
        let slice_w = 0.26 * (phase * 0.47).cos();
        for y in 0..h {
            let py = app.fractal_y
                + (y as f64 - h as f64 / 2.0) * app.fractal_scale * 2.0 / w.max(1) as f64;
            let mut line = String::with_capacity(w);
            for x in 0..w {
                let px = app.fractal_x + (x as f64 / w.max(1) as f64 - 0.5) * app.fractal_scale;
                // Rotate in two independent four-dimensional planes.
                let (mut a, mut b) = (px * cos - slice_z * sin, py * cos - slice_w * sin);
                let (mut c, mut d) = (px * sin + slice_z * cos, py * sin + slice_w * cos);
                let mut trap = 4.0_f64;
                let mut n = 0usize;
                while n < 28 {
                    let r2 = a * a + b * b + c * c + d * d;
                    trap =
                        trap.min((a.abs() + b.abs() * 0.6 + c.abs() * 0.4 + d.abs() * 0.3).abs());
                    if r2 > 16.0 {
                        break;
                    }
                    (a, b, c, d) = (
                        a * a - b * b - c * c - d * d - 0.23,
                        2.0 * a * b + 0.71,
                        2.0 * a * c + 0.16,
                        2.0 * a * d + 0.08,
                    );
                    n += 1;
                }
                let detail = ((1.0 / (0.15 + trap)).ln_1p() * 4.2) as usize;
                let bands = (n * 3 + detail * 2) % ramp.len();
                line.push(ramp[bands] as char);
            }
            put(f, 0, y, &line, MUTED);
        }
        return;
    }
    #[cfg(not(target_arch = "wasm32"))]
    if app.view == View::Unicode {
        let glyphs = ['A', 'λ', '中', 'Ж', 'あ', '∑', '∞', '☼'];
        let glyph = glyphs[app.native_glyph % glyphs.len()];
        put(f, 2, 6, &glyph.to_string(), WHITE);
        put(f, 2, 8, &format!("u+{:04x}", glyph as u32), MUTED);
        return;
    }
    #[cfg(target_arch = "wasm32")]
    if app.view == View::Unicode {
        if let Some(study) = app.unicode.as_ref() {
            let layout = unicode_layout(study, w, h);
            if let Some(x) = layout.divider {
                for y in 4..h.saturating_sub(1) {
                    put(f, x, y, "|", DIM);
                }
            }
            let (lines, heading_lines) = unicode_metadata(study, layout.metadata_width);
            for (i, line) in lines.iter().enumerate() {
                put(
                    f,
                    layout.metadata_x,
                    layout.metadata_y + i,
                    line,
                    metadata_foreground(i, heading_lines),
                );
            }
        }
        if app.unicode_font_status == crate::app::FontStatus::Failed {
            put(f, 1, 1, "font unavailable", MUTED);
        }
        return;
    }
    if app.view == View::Lichen {
        let (gw, gh) = app.lichen.size();
        let factor = 1usize << app.lichen_pixelation;
        let pw = gw.div_ceil(factor);
        let ph = gh.div_ceil(factor);
        let mut pooled = vec![0u8; pw * ph];
        for sy in 0..gh {
            for sx in 0..gw {
                let at = (sy / factor) * pw + sx / factor;
                pooled[at] = pooled[at].max(app.lichen.display_value(sy * gw + sx));
            }
        }
        let b = f.buffer_mut();
        for y in 0..h {
            for x in 0..w {
                let px = (x * gw / w.max(1)) / factor;
                let p1 = ((y * 2) * gh / (h * 2).max(1)) / factor;
                let p2 = ((y * 2 + 1) * gh / (h * 2).max(1)) / factor;
                let c = |i| {
                    let n = pooled[i];
                    Color::Rgb(n, n, n)
                };
                b[(a.x + x as u16, a.y + y as u16)]
                    .set_char('▀')
                    .set_fg(c(p1 * pw + px))
                    .set_bg(c(p2 * pw + px));
            }
        }
        return;
    }
    portfolio_view(f, app);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_key_lines_wrap_without_losing_characters() {
        let lines = document_lines(Document::Age, 24);
        assert_eq!(lines.concat(), crate::portfolio::AGE_TEXT.trim_end());
        assert!(lines.iter().all(|line| line.chars().count() <= 20));
        assert!(document_max_scroll(Document::Pgp, 32, 12) > 0);
    }
    #[test]
    fn wrapped_unicode_heading_keeps_its_foreground() {
        assert_eq!(metadata_foreground(0, 2), WHITE);
        assert_eq!(metadata_foreground(1, 2), WHITE);
        assert_eq!(metadata_foreground(2, 2), MUTED);
    }
}
