#[cfg(not(target_arch = "wasm32"))]
#[path = "frontends/native.rs"]
mod native;
#[cfg(target_arch = "wasm32")]
#[path = "frontends/web.rs"]
mod web;
#[cfg(not(target_arch = "wasm32"))]
fn main() -> std::io::Result<()> {
    native::run()
}
#[cfg(target_arch = "wasm32")]
fn main() {
    web::start();
}
