#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]

mod app;
pub use app::BootCon;
