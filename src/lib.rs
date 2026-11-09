//! Apex — a monochrome, keyboard-first portfolio.
//!
//! One shared core drives three surfaces: the native terminal, an SSH session
//! and the browser. The dependencies point one way:
//!
//! ```text
//! frontends ──▶ app ──▶ content      copy and destinations
//!                 │
//!                 ├──▶ visuals        simulations and catalogues, no rendering
//!                 └──▶ ui             Ratatui rendering, no state
//! ```
//!
//! `app` owns every piece of mutable state, `content` owns the published text,
//! `visuals` holds the generative visuals, `ui` draws an [`App`] and never
//! mutates it, and `frontends` decodes input for one surface and calls `ui`.

pub mod app;
pub mod content;
pub mod frontends;
pub mod ui;
pub mod visuals;

pub use app::{App, View};
pub use content::Document;
