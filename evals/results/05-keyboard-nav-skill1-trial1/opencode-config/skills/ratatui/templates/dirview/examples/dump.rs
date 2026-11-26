//! Print what a screen looks like as plain text, at any size.
//!
//!     cargo run --example dump -- [DIR] [WIDTH] [HEIGHT] [KEYS]
//!
//! KEYS is a string of single-key commands, e.g. `jj?` or `/src`.
//! Reading the dump at several sizes is the cheapest way to check a layout.

use std::path::PathBuf;

use dirview::keymap;
use dirview::state::State;
use dirview::theme::Theme;
use dirview::ui::{self, ViewState};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[allow(clippy::print_stdout)]
fn main() -> color_eyre::Result<()> {
    let mut args = std::env::args().skip(1);
    let dir = args
        .next()
        .map_or_else(|| PathBuf::from("."), PathBuf::from);
    let width: u16 = args.next().map_or(Ok(100), |a| a.parse())?;
    let height: u16 = args.next().map_or(Ok(24), |a| a.parse())?;
    let keys = args.next().unwrap_or_default();

    let mut state = State::open(dir)?;
    for c in keys.chars() {
        let event = KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE);
        if let Some(action) = keymap::resolve(state.mode(), event) {
            state.update(action);
        }
    }

    let mut terminal = Terminal::new(TestBackend::new(width, height))?;
    let mut view = ViewState::default();
    let theme = Theme::mono();
    terminal.draw(|f| ui::render(f, &state, &mut view, &theme))?;
    print!("{}", ui::buffer_text(terminal.backend().buffer()));
    Ok(())
}
