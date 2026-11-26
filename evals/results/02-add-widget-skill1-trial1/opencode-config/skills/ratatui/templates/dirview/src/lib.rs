//! dirview: a directory browser.
//!
//! The modules follow the data flow. `event` and `keymap` turn input into an
//! `Action`, `state` applies it, `ui` draws the result. Nothing in `ui`
//! changes state, and nothing in `state` knows about the terminal.

pub mod action;
pub mod app;
pub mod event;
pub mod fs;
pub mod keymap;
pub mod state;
pub mod theme;
pub mod ui;

pub use app::App;
