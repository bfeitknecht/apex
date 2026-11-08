//! Apex: a shared, visual-only Ratatui portfolio core.
pub mod app;
#[cfg(feature = "crash-study")]
#[path = "experiments/crash.rs"]
pub mod crash;
pub mod lichen;
pub mod portfolio;
pub mod ui;
#[cfg(target_arch = "wasm32")]
pub mod unicode_study;
pub use app::{App, View};
