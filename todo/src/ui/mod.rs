//! Rendering. Reads the state, never changes it.
//!
//! One fixed skeleton: a header row, the task list, and a footer row.

mod footer;
mod help;
mod list;
pub mod text;

use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Layout, Margin, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::state::{Mode, State};
use crate::theme::Theme;

/// Below this the interface cannot be laid out; show a message instead.
pub const MIN_WIDTH: u16 = 40;
pub const MIN_HEIGHT: u16 = 8;

/// Render-only state: scroll position, measured by the view itself.
#[derive(Debug, Default)]
pub struct ViewState {
    offset: usize,
    rows: usize,
}

impl ViewState {
    /// Rows the list showed in the last frame.
    pub fn list_rows(&self) -> usize {
        self.rows
    }
}

pub fn render(frame: &mut Frame, state: &State, view: &mut ViewState, theme: &Theme) {
    let area = frame.area();
    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        render_too_small(frame, theme);
        return;
    }

    let [header, body, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(area);
    let margin = Margin::new(1, 0);

    render_header(frame, header.inner(margin), state, theme);
    list::render(frame, body, state, view, theme);
    footer::render(frame, footer.inner(margin), state, theme);

    if matches!(state.mode(), Mode::Help | Mode::ConfirmDelete) {
        help::render(frame, area, state, theme);
    }
}

fn render_header(frame: &mut Frame, area: Rect, state: &State, theme: &Theme) {
    let remaining = state.remaining();
    let total = state.len();
    let info = format!("{remaining} left \u{b7} {total} total");
    let info_width = u16::try_from(text::width(&info)).unwrap_or(u16::MAX);
    let [title_area, info_area] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Length(info_width)]).areas(area);

    frame.render_widget(Line::styled("todo", theme.title), title_area);
    frame.render_widget(Line::styled(info, theme.muted), info_area);
}

fn render_too_small(frame: &mut Frame, theme: &Theme) {
    let area = frame.area();
    let message = vec![
        Line::from("terminal too small"),
        Line::from(Span::styled(
            format!(
                "need {MIN_WIDTH}x{MIN_HEIGHT}, have {}x{}",
                area.width, area.height
            ),
            theme.muted,
        )),
    ];
    let [_, middle, _] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(2),
        Constraint::Fill(1),
    ])
    .areas(area);
    frame.render_widget(Paragraph::new(message).alignment(Alignment::Center), middle);
}

/// A rendered buffer as plain text, one line per row, trailing spaces trimmed.
/// Wide characters occupy two cells but print once. Used by tests and the
/// `dump` example to read a screen without a terminal.
pub fn buffer_text(buffer: &Buffer) -> String {
    let area = buffer.area;
    let mut out = String::new();
    for y in 0..area.height {
        let mut row = String::new();
        let mut skip = 0;
        for x in 0..area.width {
            if skip > 0 {
                skip -= 1;
                continue;
            }
            let symbol = buffer[(x, y)].symbol();
            skip = text::width(symbol).saturating_sub(1);
            row.push_str(symbol);
        }
        out.push_str(row.trim_end());
        out.push('\n');
    }
    out
}
