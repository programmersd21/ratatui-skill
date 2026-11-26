//! The help overlay, generated from the keymap.

use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Clear, Paragraph};

use super::text::width;
use crate::action::Action;
use crate::keymap::{self, Binding};
use crate::state::Mode;
use crate::theme::Theme;

const MAX_WIDTH: u16 = 58;

pub fn render(frame: &mut Frame, area: Rect, theme: &Theme, offset: &mut usize) -> usize {
    let sections = [
        ("Browse", Mode::Normal),
        ("While filtering", Mode::Filter),
        ("Help", Mode::Help),
    ];
    let key_column = sections
        .iter()
        .flat_map(|(_, mode)| keymap::bindings(*mode))
        .map(|binding| width(binding.label))
        .max()
        .unwrap_or(0)
        + 2;

    let mut lines = Vec::new();
    for (index, (title, mode)) in sections.iter().enumerate() {
        if index > 0 {
            lines.push(Line::raw(""));
        }
        lines.push(Line::styled(*title, theme.title));
        lines.extend(
            keymap::bindings(*mode)
                .iter()
                .map(|binding| row(binding, theme, key_column)),
        );
    }

    let max_height = area.height.saturating_sub(2).max(4);
    let content_height = u16::try_from(lines.len()).unwrap_or(u16::MAX);
    let height = content_height.saturating_add(2).min(max_height);
    let popup = centered(area, MAX_WIDTH.min(area.width), height);
    let page_size = popup.height.saturating_sub(2).max(1) as usize;
    let max_offset = lines.len().saturating_sub(page_size);
    *offset = (*offset).min(max_offset);

    let close_hint = keymap::bindings(Mode::Help)
        .iter()
        .find(|binding| binding.action == Action::ToggleHelp)
        .and_then(|binding| binding.hint);
    let mut block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(theme.border_focus)
        .title(Line::styled(" Keys ", theme.title));
    if let Some(hint) = close_hint {
        block = block.title_bottom(Line::styled(
            format!(" {} {} ", hint.keys, hint.text),
            theme.muted,
        ));
    }

    frame.render_widget(Clear, popup);
    frame.render_widget(
        Paragraph::new(lines)
            .scroll((u16::try_from(*offset).unwrap_or(u16::MAX), 0))
            .block(block),
        popup,
    );
    page_size
}

fn row(binding: &Binding, theme: &Theme, key_column: usize) -> Line<'static> {
    let label = format!(" {}", binding.label);
    let pad = key_column.saturating_sub(width(&label));
    Line::from(vec![
        Span::styled(label, theme.accent),
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
