//! The file list. Only the rows on screen are built each frame, so a
//! directory with 100,000 entries costs the same as one with 10.

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Margin, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, HighlightSpacing, List, ListItem, ListState, Paragraph, Scrollbar,
    ScrollbarOrientation, ScrollbarState,
};

use super::ViewState;
use super::text::{human_size, sanitize, truncate_end, width};
use crate::state::{Entry, EntryKind, State};
use crate::theme::Theme;

/// Width reserved for the size column, including its leading space.
const SIZE_COLUMN: usize = 7;

pub fn render(frame: &mut Frame, area: Rect, state: &State, view: &mut ViewState, theme: &Theme) {
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(theme.border_focus)
        .title(Line::styled(
            format!(" by {} ", state.sort().label()),
            theme.muted,
        ));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = inner.height as usize;
    view.rows = rows;

    let Some(selected) = state.selected_index() else {
        render_empty(frame, inner, state, theme);
        return;
    };

    view.offset = scroll_offset(view.offset, selected, rows, state.visible_len());
    let items: Vec<ListItem> = state
        .visible_entries()
        .skip(view.offset)
        .take(rows)
        .map(|entry| item(entry, inner.width as usize, theme))
        .collect();

    let list = List::new(items)
        .highlight_style(theme.selected)
        .highlight_spacing(HighlightSpacing::Never);
    let mut list_state = ListState::default().with_selected(Some(selected - view.offset));
    frame.render_stateful_widget(list, inner, &mut list_state);

    if state.visible_len() > rows {
        let mut scrollbar = ScrollbarState::new(state.visible_len().saturating_sub(rows))
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

fn item<'a>(entry: &Entry, width_cols: usize, theme: &Theme) -> ListItem<'a> {
    let is_dir = entry.kind == EntryKind::Dir;
    let mut name = sanitize(&entry.name).into_owned();
    if is_dir {
        name.push('/');
    }
    let size = if entry.kind == EntryKind::File {
        human_size(entry.size)
    } else {
        String::new()
    };
    let name = truncate_end(&name, width_cols.saturating_sub(SIZE_COLUMN));
    let padding = width_cols.saturating_sub(width(&name) + width(&size));
    let name_style = if is_dir { theme.accent } else { theme.text };
    ListItem::new(Line::from(vec![
        Span::styled(name, name_style),
        Span::raw(" ".repeat(padding)),
        Span::styled(size, theme.muted),
    ]))
}

/// Say why the list is empty, and what to do about it.
fn render_empty(frame: &mut Frame, area: Rect, state: &State, theme: &Theme) {
    let message = if !state.filter().is_empty() {
        format!("no match for \"{}\"", sanitize(state.filter()))
    } else if state.total_len() > 0 {
        "only hidden files here, press . to show them".to_owned()
    } else {
        "empty directory".to_owned()
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
