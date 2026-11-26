//! The footer: contextual key hints, or the filter prompt while typing.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Position, Rect};
use ratatui::text::{Line, Span};

use super::text::{sanitize, truncate_end, width};
use crate::keymap;
use crate::state::{Mode, State};
use crate::theme::Theme;

pub fn render(frame: &mut Frame, area: Rect, state: &State, theme: &Theme) {
    let status = state.status().map(|s| sanitize(s).into_owned());
    let status_width = status
        .as_deref()
        .map_or(0, |s| width(s).min(area.width as usize / 2));
    let [left, right] = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(u16::try_from(status_width).unwrap_or(0)),
    ])
    .areas(area);

    if let Some(status) = status {
        frame.render_widget(
            Line::styled(truncate_end(&status, status_width), theme.error),
            right,
        );
    }

    if state.mode() == Mode::Filter {
        render_prompt(frame, left, state, theme);
    } else {
        frame.render_widget(hints(state.mode(), left.width as usize, theme), left);
    }
}

/// `/text` with the cursor after the text. The hints stay visible to the right
/// when they fit, so the way out is always on screen.
fn render_prompt(frame: &mut Frame, area: Rect, state: &State, theme: &Theme) {
    let text = sanitize(state.filter()).into_owned();
    let prompt = Line::from(vec![
        Span::styled("/", theme.accent),
        Span::styled(text.clone(), theme.text),
    ]);
    frame.render_widget(prompt, area);

    let x = area.x + 1 + u16::try_from(width(&text)).unwrap_or(0);
    let x = x.min(area.right().saturating_sub(1));
    frame.set_cursor_position(Position::new(x, area.y));

    let used = width(&text) + 4;
    if area.width as usize > used + 24 {
        let hint_width = area.width - u16::try_from(used).unwrap_or(0);
        let hint_area = Rect::new(area.right() - hint_width, area.y, hint_width, 1);
        let line = hints(Mode::Filter, hint_width as usize, theme).right_aligned();
        frame.render_widget(line, hint_area);
    }
}

/// Footer hints for a mode, dropping the last ones that do not fit.
fn hints(mode: Mode, max_width: usize, theme: &Theme) -> Line<'static> {
    let mut spans = Vec::new();
    let mut used = 0;
    for hint in keymap::bindings(mode).iter().filter_map(|b| b.hint) {
        let cost = width(hint.keys) + 1 + width(hint.text) + 2;
        if used + cost > max_width {
            break;
        }
        spans.push(Span::styled(hint.keys, theme.accent));
        spans.push(Span::styled(format!(" {}  ", hint.text), theme.muted));
        used += cost;
    }
    Line::from(spans)
}
