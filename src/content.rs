//! The published text: the shared definition in `content/`, parsed once.
//!
//! `content/profile.toml` and `content/projects.toml` define all the text of the
//! site. The Typst site reads the same files at build time. A changed link
//! therefore reaches the terminal, the SSH session, the browser app and the page
//! without JavaScript at the same time.
//!
//! The code parses the tables at runtime, because Typst cannot read Rust types
//! and Rust cannot read Typst. It then freezes the strings with `Box::leak`, so
//! the rest of the crate uses `&'static str` and needs no lifetime parameters.
//! This work happens once per process and covers a few kilobytes.

use std::sync::LazyLock;

/// A destination in the top bar: the CV, the blog, the gallery.
#[derive(Clone, Copy, Debug)]
pub struct Destination {
    pub label: &'static str,
    pub url: &'static str,
}

/// A contact row in the footer. An empty `url` has no target yet.
#[derive(Clone, Copy, Debug)]
pub struct Contact {
    pub label: &'static str,
    pub url: &'static str,
}

/// A project in the form that the terminal needs: one line, no final period.
#[derive(Clone, Copy, Debug)]
pub struct Project {
    pub name: &'static str,
    pub summary: &'static str,
    pub url: &'static str,
}

/// Everything the site says, in the shape the UI wants.
pub struct Profile {
    pub name: &'static str,
    pub email: &'static str,
    pub eyebrow: &'static str,
    /// One sentence for the hero of the page.
    pub lede: &'static str,
    /// The shared paragraph, exactly as written.
    pub about: &'static str,
    /// The same paragraph in lowercase, for the terminal.
    pub about_terminal: &'static str,
    /// Extra hero lines that only the terminal shows.
    pub intro: &'static [&'static str],
    pub links: &'static [Destination],
    pub contacts: &'static [Contact],
    pub projects: &'static [Project],
}

/// The parsed definition. Read it through [`profile`].
static PROFILE: LazyLock<Profile> = LazyLock::new(load);

/// The published definition of this site.
pub fn profile() -> &'static Profile {
    &PROFILE
}

/// The origin of the site. Terminal links need absolute URLs.
pub const SITE_URL: &str = "https://0xbf.ch";

/// `url` as an absolute URL: a root-relative path gets [`SITE_URL`].
pub fn absolute(url: &str) -> String {
    if url.starts_with('/') {
        format!("{SITE_URL}{url}")
    } else {
        url.to_string()
    }
}

/// Polyring, the ETH Zurich webring. The hub and a snapshot of its members
/// (`content/polyring.txt`, refreshed by `just polyring` from the ring's own
/// embed script). The ring has no redirect service, so every surface computes
/// its neighbours from the snapshot, as the ring's banner does.
pub const POLYRING_URL: &str = "https://polyring.ch";
const POLYRING_MEMBERS: &str = include_str!("../content/polyring.txt");

/// The member URLs in ring order.
pub fn polyring_members() -> Vec<&'static str> {
    POLYRING_MEMBERS
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect()
}

/// The previous and the next site in the ring, wrapping at both ends. The hub
/// twice if this site is not a member.
pub fn polyring_neighbours() -> (&'static str, &'static str) {
    let members = polyring_members();
    let Some(me) = members
        .iter()
        .position(|url| url.trim_end_matches('/') == SITE_URL)
    else {
        return (POLYRING_URL, POLYRING_URL);
    };
    let n = members.len();
    (members[(me + n - 1) % n], members[(me + 1) % n])
}

/// A member other than this site, chosen by `seed`.
pub fn polyring_pick(seed: u64) -> &'static str {
    let others: Vec<_> = polyring_members()
        .into_iter()
        .filter(|url| url.trim_end_matches('/') != SITE_URL)
        .collect();
    if others.is_empty() {
        return POLYRING_URL;
    }
    // splitmix64: neighbouring seeds land far apart.
    let mut z = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    others[((z ^ (z >> 31)) % others.len() as u64) as usize]
}

/// Public key files, served from the document root by every surface.
pub const AGE_URL: &str = "/age.txt";
pub const PGP_URL: &str = "/pgp.txt";
/// The portrait is a document for the terminal. The browser has no link for it.
pub const ME_URL: &str = "";

pub const AGE_TEXT: &str = include_str!("../content/age.txt");
pub const PGP_TEXT: &str = include_str!("../content/pgp.txt");
pub const ME_ART: &str = include_str!("../content/portrait.txt");

const PROFILE_TOML: &str = include_str!("../content/profile.toml");
const PROJECTS_TOML: &str = include_str!("../content/projects.toml");

/// A footer document that opens inside the terminal instead of in a browser.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Document {
    Age,
    Pgp,
    Me,
}

impl Document {
    pub const fn title(self) -> &'static str {
        match self {
            Self::Age => "age.txt",
            Self::Pgp => "pgp.txt",
            Self::Me => "me",
        }
    }

    pub const fn text(self) -> &'static str {
        match self {
            Self::Age => AGE_TEXT,
            Self::Pgp => PGP_TEXT,
            Self::Me => ME_ART,
        }
    }
}

/// Freeze a parsed string for the lifetime of the process.
fn frozen(text: String) -> &'static str {
    Box::leak(text.trim().to_string().into_boxed_str())
}

/// Freeze a parsed sequence for the lifetime of the process.
fn frozen_slice<T>(items: Vec<T>) -> &'static [T] {
    Box::leak(items.into_boxed_slice())
}

#[derive(serde::Deserialize)]
struct RawEntry {
    label: String,
    #[serde(default)]
    url: String,
}

#[derive(serde::Deserialize)]
struct RawProfile {
    name: String,
    email: String,
    eyebrow: String,
    lede: String,
    about: String,
    #[serde(default)]
    intro: Vec<String>,
    #[serde(default)]
    link: Vec<RawEntry>,
    #[serde(default)]
    contact: Vec<RawEntry>,
}

#[derive(serde::Deserialize)]
struct RawProject {
    name: String,
    #[serde(default)]
    summary: String,
    #[serde(default)]
    url: String,
}

#[derive(serde::Deserialize)]
struct RawProjects {
    #[serde(default)]
    project: Vec<RawProject>,
}

fn load() -> Profile {
    let raw: RawProfile = toml::from_str(PROFILE_TOML).expect("content/profile.toml");
    let projects: RawProjects = toml::from_str(PROJECTS_TOML).expect("content/projects.toml");

    let links = frozen_slice(
        raw.link
            .into_iter()
            .map(|entry| Destination {
                label: frozen(entry.label),
                url: frozen(entry.url),
            })
            .collect(),
    );
    let contacts = frozen_slice(
        raw.contact
            .into_iter()
            .map(|entry| Contact {
                label: frozen(entry.label),
                url: frozen(entry.url),
            })
            .collect(),
    );
    let intro = frozen_slice(raw.intro.into_iter().map(frozen).collect());
    let projects = frozen_slice(
        projects
            .project
            .into_iter()
            .map(|project| Project {
                name: frozen(project.name),
                summary: frozen(project.summary),
                url: frozen(project.url),
            })
            .collect(),
    );

    let about = frozen(raw.about);
    Profile {
        name: frozen(raw.name),
        email: frozen(raw.email),
        eyebrow: frozen(raw.eyebrow),
        lede: frozen(raw.lede),
        about,
        about_terminal: frozen(about.to_lowercase()),
        intro,
        links,
        contacts,
        projects,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_is_populated() {
        let profile = profile();
        assert!(!profile.name.is_empty());
        assert!(!profile.lede.is_empty());
        assert!(!profile.about.is_empty());
        assert!(!profile.intro.is_empty());
        assert_eq!(profile.intro[0], profile.intro[0].to_lowercase());
    }

    #[test]
    fn destinations_are_absolute() {
        let profile = profile();
        assert!(!profile.links.is_empty());
        for link in profile.links {
            assert!(!link.label.is_empty());
            assert!(
                link.url.starts_with("https://"),
                "{} is not absolute",
                link.label
            );
        }
    }

    #[test]
    fn contacts_and_projects_are_absolute_or_intentionally_empty() {
        let profile = profile();
        assert!(!profile.contacts.is_empty());
        assert!(!profile.projects.is_empty());
        for url in profile
            .contacts
            .iter()
            .map(|contact| contact.url)
            .chain(profile.projects.iter().map(|project| project.url))
        {
            assert!(url.is_empty() || url.starts_with("https://") || url.starts_with("mailto:"));
        }
    }

    #[test]
    fn terminal_copy_is_lowercase() {
        let profile = profile();
        assert_eq!(
            profile.about_terminal,
            profile.about_terminal.to_lowercase()
        );
    }

    #[test]
    fn published_documents_are_not_empty() {
        for document in [Document::Age, Document::Pgp, Document::Me] {
            assert!(!document.text().trim().is_empty(), "{}", document.title());
        }
    }

    #[test]
    fn artifacts_are_root_relative() {
        assert!(AGE_URL.starts_with('/') && PGP_URL.starts_with('/'));
    }

    #[test]
    fn polyring_snapshot_has_this_site_and_distinct_neighbours() {
        assert!(polyring_members().len() > 2);
        let (previous, next) = polyring_neighbours();
        assert_ne!(previous, POLYRING_URL);
        assert_ne!(previous, next);
        for seed in 0..64 {
            assert_ne!(polyring_pick(seed).trim_end_matches('/'), SITE_URL);
        }
    }
}
