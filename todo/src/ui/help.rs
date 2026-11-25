//! The help overlay and the delete confirmation, generated from the keymap.

use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Clear, Paragraph};

use super::text::width;
use crate::keymap::{self, Binding};
use crate::state::{Mode, State};
use crate::theme::Theme;

const KEY_COLUMN: usize = 18;
const MAX_WIDTH: u16 = 58;

pub fn render(frame: &mut Frame, area: Rect, state: &State, theme: &Theme) {
    match state.mode() {
        Mode::Help => render_help(frame, area, theme),
        Mode::ConfirmDelete => render_confirm(frame, area, state, theme),
        _ => {}
    }
}

fn render_help(frame: &mut Frame, area: Rect, theme: &Theme) {
    let mut lines = vec![Line::styled("Tasks", theme.title)];
    lines.extend(keymap::bindings(Mode::Normal).iter().map(|b| row(b, theme)));
    lines.push(Line::raw(""));
    lines.push(Line::styled("Adding a task", theme.title));
    lines.extend(keymap::bindings(Mode::Add).iter().map(|b| row(b, theme)));
    lines.push(Line::raw(""));
    lines.push(Line::styled("Confirming a delete", theme.title));
    lines.extend(
        keymap::bindings(Mode::ConfirmDelete)
            .iter()
            .map(|b| row(b, theme)),
    );

    let height = u16::try_from(lines.len() + 2).unwrap_or(u16::MAX);
    let popup = centered(area, MAX_WIDTH.min(area.width), height.min(area.height));

    let mut block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(theme.border_focus)
        .title(Line::styled(" Keys ", theme.title));
    if let Some(hint) = keymap::bindings(Mode::Help).iter().find_map(|b| b.hint) {
        let close = format!(" {} {} ", hint.keys, hint.text);
        block = block.title_bottom(Line::styled(close, theme.muted));
    }
    frame.render_widget(Clear, popup);
    frame.render_widget(Paragraph::new(lines).block(block), popup);
}

fn render_confirm(frame: &mut Frame, area: Rect, state: &State, theme: &Theme) {
    let task = state.selected_task().map(|t| t.text.as_str()).unwrap_or("");
    let question = format!("Delete \"{task}\"?");
    let lines = vec![
        Line::styled(question, theme.text),
        Line::raw(""),
        Line::from(vec![
            Span::styled("y ", theme.accent),
            Span::styled("delete  ", theme.muted),
            Span::styled("n ", theme.accent),
            Span::styled("keep", theme.muted),
        ]),
    ];
    let popup = centered(area, MAX_WIDTH.min(area.width), 5.min(area.height));
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(theme.border_focus)
        .title(Line::styled(" Confirm ", theme.title));
    frame.render_widget(Clear, popup);
    frame.render_widget(Paragraph::new(lines).block(block), popup);
}

fn row(binding: &Binding, theme: &Theme) -> Line<'static> {
    let pad = KEY_COLUMN.saturating_sub(width(binding.label));
    Line::from(vec![
        Span::styled(format!(" {}", binding.label), theme.accent),
        Span::raw(" ".repeat(pad)),
        Span::styled(binding.help, theme.text),
    ])
}

fn centered(area: Rect, w: u16, h: u16) -> Rect {
    let [row] = Layout::vertical([Constraint::Length(h)])
        .flex(Flex::Center)
        .areas(area);
    let [cell] = Layout::horizontal([Constraint::Length(w)])
        .flex(Flex::Center)
        .areas(row);
    cell
}
