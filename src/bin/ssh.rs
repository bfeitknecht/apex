//! SSH entry point.
//!
//! ```sh
//! cargo run --release --features ssh --bin ssh
//! ```

fn main() -> Result<(), Box<dyn std::error::Error>> {
    apex::frontends::ssh::run()
}
