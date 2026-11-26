//! The event loop: draw, wait for input, update.

use std::io;
use std::path::PathBuf;

use ratatui::DefaultTerminal;

use crate::action::Action;
use crate::event;
use crate::state::State;
use crate::theme::Theme;
use crate::ui::{self, ViewState};

pub struct App {
    state: State,
    view: ViewState,
    theme: Theme,
}

impl App {
    pub fn new(dir: PathBuf) -> io::Result<Self> {
        Ok(Self {
            state: State::open(dir)?,
            view: ViewState::default(),
            theme: Theme::detect(),
        })
    }

    pub fn run(mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.state.should_quit() {
            terminal.draw(|frame| ui::render(frame, &self.state, &mut self.view, &self.theme))?;

            // The view measured the list; tell the state so paging matches.
            if self.view.list_rows() != self.state.page_size() {
                self.state
                    .update(Action::SetPageSize(self.view.list_rows()));
            }

            if let Some(action) = event::next_action(self.state.mode())? {
                if self.state.mode() == crate::state::Mode::Help {
                    match action {
                        Action::HelpScrollUp => self.view.scroll_help(-1),
                        Action::HelpScrollDown => self.view.scroll_help(1),
                        Action::HelpPageUp => self.view.scroll_help(-8),
                        Action::HelpPageDown => self.view.scroll_help(8),
                        _ => self.state.update(action),
                    }
                } else {
                    self.state.update(action);
                }
            }
        }
        Ok(())
    }
}
