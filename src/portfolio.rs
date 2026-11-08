//! Editable portfolio copy and destinations.
pub const INTRO: &str = "bachelor of science in computer science at ETH zürich.\nstudent. teacher. artist. engineer. generalist.\nprogramming languages, formal methods, information security,\ncomputer systems and networks, machine learning.";
pub const ABOUT: &str = "welcome to my personal website slash portfolio.\nhere you can find information about me, my projects, and how to get in touch.\nmy interests span wide and deep. from computer science (both in theory and practice)\nto math, music, and sports. i want to understand how things work and make them better.";
pub const CV_URL: &str = "https://0xbf.ch/cv";
pub const BLOG_URL: &str = "https://blog.0xbf.ch";
pub const GALLERY_URL: &str = "https://gallery.0xbf.ch";
pub const EXTERNAL_NAV: [&str; 3] = [CV_URL, BLOG_URL, GALLERY_URL];
// These bundled public keys are served as text/plain and shown inside the TUI.
// Replace the files when the corresponding keys change.
pub const AGE_URL: &str = "age.txt";
pub const PGP_URL: &str = "pgp.txt";
pub const ME_URL: &str = "placeholder-me.svg";
pub const AGE_TEXT: &str = include_str!("../assets/age.txt");
pub const PGP_TEXT: &str = include_str!("../assets/pgp.txt");
pub const ME_ART: &str = r#"       *                    .
                  .      *
          /\_/\\
     .   ( o.o )        *
          > ^ <
       __/     \__
      /           \       .
     /_____________\
          /   \
     *   /_____\     .
             .        *"#;
pub struct Project {
    pub name: &'static str,
    pub description: &'static str,
    pub url: &'static str,
}
// Empty destinations are placeholders until the project has a public URL.
pub const PROJECTS: &[Project] = &[
    Project {
        name: "ethz-vpn",
        description: "",
        url: "https://github.com/bfeitknecht/ethz-vpn",
    },
    Project {
        name: "macho",
        description: "abnormal machine sound detection",
        url: "",
    },
    Project {
        name: "impli",
        description: "the IMP language interpreter",
        url: "https://github.com/bfeitknecht/impli",
    },
    Project {
        name: "spicky",
        description: "Typst cheatsheet package and template",
        url: "https://github.com/bfeitknecht/spicky",
    },
    Project {
        name: "infra",
        description: "homelab cluster configuration",
        url: "https://github.com/bfeitknecht/infra",
    },
    Project {
        name: "omni",
        description: "everywhere nix configuration",
        url: "https://github.com/bfeitknecht/omni",
    },
    Project {
        name: "thyme",
        description: "Typst based SSG",
        url: "",
    },
    Project {
        name: "apex",
        description: "personal website and portfolio",
        url: "https://github.com/bfeitknecht/apex",
    },
    Project {
        name: "glob",
        description: "personal blog built with thyme",
        url: "",
    },
];
pub const CONTACTS: [(&str, &str); 4] = [
    ("io@0xbf.ch", "mailto:io@0xbf.ch"),
    ("github", "https://github.com/bfeitknecht"),
    ("tangled", ""),
    ("linkedin", "https://linkedin.com/in/bfeitknecht"),
];
