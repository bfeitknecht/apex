//! Apex, the dynamic version of 0xbf.ch: a monochrome portfolio that a
//! keyboard can drive. (The static version is plain HTML from `static/`.)
//!
//! One core drives three frontends: browser, ssh and native. The dependencies point one way:
//!
//! ```text
//! frontends ──▶ app ──▶ content      copy and destinations
//!                 │
//!                 ├──▶ visuals        simulations and catalogues, no rendering
//!                 └──▶ ui             Ratatui rendering, no state
//! ```
//!
//! `app` owns the mutable state. `content` owns the published text. `visuals`
//! holds the generative visuals. `ui` draws an [`App`] and does not change it.
//! `frontends` decodes the input of one frontend and calls `ui`.

pub mod app;
pub mod content;
pub mod frontends;
pub mod ui;
pub mod visuals;

pub use app::{App, View};
pub use content::Document;
