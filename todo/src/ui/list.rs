//! The task list. Only the rows on screen are built each frame.

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Margin, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, HighlightSpacing, List, ListItem, ListState, Paragraph, Scrollbar,
    ScrollbarOrientation, ScrollbarState,
};

use super::ViewState;
use super::text::{sanitize, truncate_end, width};
use crate::state::{Mode, State, Task};
use crate::theme::Theme;

pub fn render(frame: &mut Frame, area: Rect, state: &State, view: &mut ViewState, theme: &Theme) {
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(theme.border_focus);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = inner.height as usize;
    view.rows = rows;

    if state.selected_index().is_none() {
        render_empty(frame, inner, state, theme);
        return;
    }

    view.offset = scroll_offset(
        view.offset,
        state.selected_index().unwrap_or(0),
        rows,
        state.len(),
    );
    let items: Vec<ListItem> = state
        .tasks()
        .iter()
        .skip(view.offset)
        .take(rows)
        .map(|task| item(task, inner.width as usize, theme))
        .collect();

    let list = List::new(items)
        .highlight_style(theme.selected)
        .highlight_spacing(HighlightSpacing::Never);
    let selected = state.selected_index().unwrap_or(0) - view.offset;
    let mut list_state = ListState::default().with_selected(Some(selected));
    frame.render_stateful_widget(list, inner, &mut list_state);

    if state.len() > rows {
        let mut scrollbar = ScrollbarState::new(state.len().saturating_sub(rows))
            .position(view.offset)
            .viewport_content_length(rows);
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None)
                .track_symbol(None)
                .style(theme.border),
            area.inner(Margin::new(0, 1)),
            &mut scrollbar,
        );
    }
}

/// Keep the selection on screen while moving the window as little as possible.
fn scroll_offset(offset: usize, selected: usize, rows: usize, len: usize) -> usize {
    if rows == 0 {
        return 0;
    }
    let offset = if selected < offset {
        selected
    } else if selected >= offset + rows {
        selected + 1 - rows
    } else {
        offset
    };
    offset.min(len.saturating_sub(rows))
}

fn item<'a>(task: &Task, width_cols: usize, theme: &Theme) -> ListItem<'a> {
    let checkbox = if task.done { "[x] " } else { "[ ] " };
    let name = truncate_end(
        &sanitize(&task.text),
        width_cols.saturating_sub(width(checkbox)),
    );
    let style = if task.done { theme.done } else { theme.text };
    let mark_style = if task.done {
        theme.done_mark
    } else {
        theme.text
    };
    ListItem::new(Line::from(vec![
        Span::styled(checkbox, mark_style),
        Span::styled(name, style),
    ]))
}

/// Say why the list is empty, and what to do about it.
fn render_empty(frame: &mut Frame, area: Rect, state: &State, theme: &Theme) {
    let message = if state.mode() == Mode::Add {
        "type a task, then press Enter"
    } else {
        "no tasks yet \u{b7} press a to add one"
    };
    let [_, middle, _] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .areas(area);
    frame.render_widget(
        Paragraph::new(message)
            .style(theme.muted)
            .alignment(Alignment::Center),
        middle,
    );
}

#[cfg(test)]
mod tests {
    use super::scroll_offset;

    #[test]
    fn selection_scrolls_into_view_with_minimal_movement() {
        assert_eq!(scroll_offset(0, 3, 10, 100), 0);
        assert_eq!(scroll_offset(0, 10, 10, 100), 1);
        assert_eq!(scroll_offset(5, 2, 10, 100), 2);
        assert_eq!(scroll_offset(5, 7, 10, 100), 5);
    }

    #[test]
    fn offset_never_leaves_blank_rows_at_the_end() {
        assert_eq!(scroll_offset(50, 99, 10, 100), 90);
        assert_eq!(scroll_offset(8, 9, 10, 12), 2);
        assert_eq!(scroll_offset(3, 1, 10, 5), 0);
    }

    #[test]
    fn zero_rows_is_safe() {
        assert_eq!(scroll_offset(4, 4, 0, 10), 0);
    }
}
