//! Frontends: one adapter per surface, each decoding its own input.
//!
//! All three drive the same [`crate::App`] and render through [`crate::ui`].
//! The browser adapter only builds for `wasm32`, the SSH adapter only with the
//! `ssh` feature, so a default native build pulls in neither Ratzilla nor russh.

#[cfg(not(target_arch = "wasm32"))]
pub mod native;

#[cfg(target_arch = "wasm32")]
pub mod web;

#[cfg(all(feature = "ssh", not(target_arch = "wasm32")))]
pub mod ssh;
