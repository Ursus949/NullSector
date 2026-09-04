//! NullSector is a desktop GUI that wraps common networking and security
//! tools behind buttons, built with [egui] and [eframe].
//!
//! The crate exposes one type, [`BootCon`], which is the `eframe::App`
//! implementation that draws the window and runs each tool. See the
//! project [README](https://github.com/Ursus949/NullSector) for a feature
//! overview and build instructions, and `docs/USAGE.md` for a walkthrough
//! of each panel.
//!
//! [egui]: https://docs.rs/egui
//! [eframe]: https://docs.rs/eframe

#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]

mod app;
pub use app::BootCon;
