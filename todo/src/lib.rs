//! todo-tui: a small persistent todo list.
//!
//! `event` and `keymap` turn input into an `Action`, `state` applies it and
//! persists through `store`, and `ui` draws the result. `ui` never changes
//! state, and `state` has no terminal types.

pub mod action;
pub mod app;
pub mod event;
pub mod keymap;
pub mod state;
pub mod store;
pub mod theme;
pub mod ui;

pub use app::App;
