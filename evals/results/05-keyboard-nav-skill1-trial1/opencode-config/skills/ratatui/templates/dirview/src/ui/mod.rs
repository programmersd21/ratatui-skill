//! Rendering. Reads the state, never changes it.
//!
//! Layout is one fixed skeleton: a header row, a body, a footer row. The body
//! is the list, plus a preview pane when there is room.

mod footer;
mod help;
mod list;
mod preview;
pub mod text;

use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Layout, Margin, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::state::{Mode, State};
use crate::theme::Theme;
use text::{sanitize, truncate_start};

/// Below this the interface cannot be laid out; show a message instead.
pub const MIN_WIDTH: u16 = 40;
pub const MIN_HEIGHT: u16 = 8;
/// The preview pane needs this much width to be useful.
const PREVIEW_MIN_WIDTH: u16 = 80;

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

    if area.width >= PREVIEW_MIN_WIDTH {
        let [list_area, preview_area] =
            Layout::horizontal([Constraint::Percentage(40), Constraint::Fill(1)]).areas(body);
        list::render(frame, list_area, state, view, theme);
        preview::render(frame, preview_area, state, theme);
    } else {
        list::render(frame, body, state, view, theme);
    }

    footer::render(frame, footer.inner(margin), state, theme);

    if state.mode() == Mode::Help {
        help::render(frame, area, theme);
    }
}

fn render_header(frame: &mut Frame, area: Rect, state: &State, theme: &Theme) {
    let info = if state.filter().is_empty() {
        format!("{} items", state.visible_len())
    } else {
        format!(
            "{} of {} match \"{}\"",
            state.visible_len(),
            state.total_len(),
            sanitize(state.filter())
        )
    };
    let info_width = u16::try_from(text::width(&info)).unwrap_or(u16::MAX);
    let [path_area, info_area] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Length(info_width)]).areas(area);

    let path = sanitize(&state.dir().to_string_lossy()).into_owned();
    let path = truncate_start(&path, path_area.width.saturating_sub(2) as usize);
    frame.render_widget(Line::styled(path, theme.title), path_area);
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
