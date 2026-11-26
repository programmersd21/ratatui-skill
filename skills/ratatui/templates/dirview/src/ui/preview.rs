//! The preview pane for the selected entry.

use std::time::SystemTime;

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType, Paragraph};

use super::text::{human_age, human_size, sanitize, truncate_end};
use crate::state::{Entry, EntryKind, Preview, State};
use crate::theme::Theme;

pub fn render(frame: &mut Frame, area: Rect, state: &State, theme: &Theme) {
    let mut block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(theme.border);
    if let Some(entry) = state.selected_entry() {
        let name = truncate_end(
            &sanitize(&entry.name),
            area.width.saturating_sub(4) as usize,
        );
        block = block.title(Line::styled(format!(" {name} "), theme.muted));
    }
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(entry) = state.selected_entry() else {
        return;
    };
    let mut lines = vec![Line::styled(meta(entry, state.preview()), theme.muted)];
    lines.push(Line::raw(""));
    match state.preview() {
        Preview::Empty => {}
        Preview::Binary => lines.push(Line::styled("binary file", theme.muted)),
        Preview::Error(message) => {
            lines.push(Line::styled(sanitize(message).into_owned(), theme.error));
        }
        Preview::Dir { names, total, truncated } => {
            lines.extend(names.iter().map(|n| {
                let style = if n.ends_with('/') {
                    theme.accent
                } else {
                    theme.text
                };
                Line::styled(sanitize(n).into_owned(), style)
            }));
            if *truncated {
                lines.push(Line::styled("\u{2026} more entries", theme.muted));
            }
        }
        Preview::Text {
            lines: text,
            truncated,
        } => {
            lines.extend(text.iter().map(|l| Line::styled(l.clone(), theme.text)));
            if *truncated {
                lines.push(Line::styled("\u{2026}", theme.muted));
            }
        }
    }
    // Paragraph clips lines at the pane edge; nothing wraps.
    frame.render_widget(Paragraph::new(lines), inner);
}

fn meta(entry: &Entry, preview: &Preview) -> String {
    match (entry.kind, preview) {
        (EntryKind::Dir, Preview::Dir { total, truncated, .. }) => {
            if *truncated { format!("{}+ items", total - 1) } else { format!("{total} items") }
        },
        (EntryKind::Dir, _) => "directory".to_owned(),
        (EntryKind::File, _) => {
            let modified = entry
                .modified
                .and_then(|m| SystemTime::now().duration_since(m).ok())
                .map_or_else(|| "unknown".to_owned(), human_age);
            format!("{} \u{b7} modified {modified}", human_size(entry.size))
        }
        (EntryKind::Other, _) => "special file".to_owned(),
    }
}
