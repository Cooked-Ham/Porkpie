//! Web entrypoint. Launches the shared Porkpie Dioxus UI inside a
//! browser via `dioxus_web::launch`.
//!
//! The resulting WebAssembly module must be served from a static
//! directory that includes the `index.html` shipped alongside this
//! crate. See `build-web.ps1` for a one-shot build that compiles the
//! crate, runs `wasm-bindgen`, and copies the assets into a single
//! distributable directory.

#![cfg_attr(target_arch = "wasm32", allow(unsafe_op_in_unsafe_fn))]

fn main() {
    let cfg = porkpie_web::LaunchConfig::load();
    dioxus_web::launch_cfg(porkpie_web::App, cfg.into_dioxus_config());
}
