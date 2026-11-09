//! Entry point for the native terminal build and the browser (wasm32) build.
//!
//! The browser binary is the Trunk entry point; `cargo run` starts the native
//! terminal frontend. `ssh` (`src/bin/ssh.rs`) is a separate binary behind
//! the `ssh` feature.

#[cfg(target_arch = "wasm32")]
fn main() {
    apex::frontends::web::start();
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> std::io::Result<()> {
    apex::frontends::native::run()
}
