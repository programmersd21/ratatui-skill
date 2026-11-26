//! The help overlay, generated from the keymap.

use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Clear, Paragraph};

use super::text::width;
use crate::keymap::{self, Binding};
use crate::state::Mode;
use crate::theme::Theme;

const KEY_COLUMN: usize = 16;
const MAX_WIDTH: u16 = 58;

pub fn render(frame: &mut Frame, area: Rect, theme: &Theme, offset: usize) {
    let mut lines = vec![Line::styled("Browse", theme.title)];
    lines.extend(keymap::bindings(Mode::Normal).iter().map(|b| row(b, theme)));
    lines.push(Line::raw(""));
    lines.push(Line::styled("While filtering", theme.title));
    lines.extend(keymap::bindings(Mode::Filter).iter().map(|b| row(b, theme)));

    let max_height = area.height.saturating_sub(2).max(4);
    let content_height = u16::try_from(lines.len()).unwrap_or(u16::MAX);
    let height = content_height.saturating_add(2).min(max_height);
    let popup = centered(area, MAX_WIDTH.min(area.width), height);

    // When the content does not fit, drop the least important lines and show
    // a scroll hint instead of silently clipping.
    let inner_height = height.saturating_sub(2) as usize;
    let truncated = lines.len() > inner_height;
    if truncated {
        lines.truncate(inner_height.saturating_sub(1));
        lines.push(Line::styled("  ... (resize for more)", theme.muted));
    }

    let mut block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(theme.border_focus)
        .title(Line::styled(" Keys ", theme.title));
    if let Some(hint) = keymap::bindings(Mode::Help).iter().find_map(|b| b.hint) {
        let close = format!(" {} {} ", hint.keys, hint.text);
        block = block.title_bottom(Line::styled(close, theme.muted));
    }
    frame.render_widget(Clear, popup);
    let max_offset = lines
        .len()
        .saturating_sub(popup.height.saturating_sub(2) as usize);
    let offset = offset.min(max_offset);
    frame.render_widget(
        Paragraph::new(lines)
            .scroll((u16::try_from(offset).unwrap_or(u16::MAX), 0))
            .block(block),
        popup,
    );
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
