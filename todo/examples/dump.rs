//! Print what a screen looks like as plain text, at any size.
//!
//!     cargo run --example dump -- [FILE] [WIDTH] [HEIGHT] [KEYS]
//!
//! KEYS is a string of single-key commands, e.g. `aa` or `?`.
//! Respects TODO_THEME and NO_COLOR, like the app.

use std::path::PathBuf;

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use todo_tui::keymap;
use todo_tui::state::State;
use todo_tui::theme::Theme;
use todo_tui::ui::{self, ViewState};

#[allow(clippy::print_stdout)]
fn main() -> color_eyre::Result<()> {
    let mut args = std::env::args().skip(1);
    let file = args
        .next()
        .map_or_else(|| PathBuf::from("/tmp/todo-dump.txt"), PathBuf::from);
    let width: u16 = args.next().map_or(Ok(100), |a| a.parse())?;
    let height: u16 = args.next().map_or(Ok(24), |a| a.parse())?;
    let keys = args.next().unwrap_or_default();

    let mut state = State::open(file)?;
    for c in keys.chars() {
        let event = KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE);
        if let Some(action) = keymap::resolve(state.mode(), event) {
            state.update(action);
        }
    }

    let mut terminal = Terminal::new(TestBackend::new(width, height))?;
    let mut view = ViewState::default();
    let theme = Theme::detect();
    terminal.draw(|f| ui::render(f, &state, &mut view, &theme))?;
    print!("{}", ui::buffer_text(terminal.backend().buffer()));
    Ok(())
}
