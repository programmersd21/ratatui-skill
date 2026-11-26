//! Terminal input, translated into actions.

use std::io;

use ratatui::crossterm::event::{self, Event};

use crate::action::Action;
use crate::keymap;
use crate::state::Mode;

/// Block until the next terminal event. Returns `None` for events that do not
/// map to an action (resize, focus, mouse); the caller simply redraws.
pub fn next_action(mode: Mode) -> io::Result<Option<Action>> {
    Ok(match event::read()? {
        Event::Key(key) => keymap::resolve(mode, key),
        _ => None,
    })
}
