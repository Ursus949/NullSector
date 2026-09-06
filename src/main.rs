#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]
//#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "NullSector BootCon",
        native_options,
        Box::new(|cc| Ok(Box::new(nullsector::BootCon::new(cc)))),
    )
}
