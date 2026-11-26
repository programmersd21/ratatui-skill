//! Colors by role. This is the only file that names a color.
//!
//! Three layers: the monochrome theme works on any terminal and is used when
//! `NO_COLOR` is set; the default theme uses the 16 ANSI colors, so it follows
//! the user's own palette; no background is ever forced.

use ratatui::style::{Color, Modifier, Style};

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub text: Style,
    pub muted: Style,
    pub accent: Style,
    pub title: Style,
    pub border: Style,
    pub border_focus: Style,
    pub selected: Style,
    pub error: Style,
}

impl Theme {
    /// Honors the `NO_COLOR` convention: any non-empty value turns color off.
    pub fn detect() -> Self {
        let no_color = std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty());
        if no_color { Self::mono() } else { Self::ansi() }
    }

    pub fn ansi() -> Self {
        Self {
            text: Style::new(),
            muted: Style::new().add_modifier(Modifier::DIM),
            accent: Style::new().fg(Color::Cyan),
            title: Style::new().add_modifier(Modifier::BOLD),
            border: Style::new().add_modifier(Modifier::DIM),
            border_focus: Style::new().fg(Color::Cyan),
            selected: Style::new().add_modifier(Modifier::REVERSED),
            error: Style::new().fg(Color::Red),
        }
    }

    pub fn mono() -> Self {
        Self {
            text: Style::new(),
            muted: Style::new().add_modifier(Modifier::DIM),
            accent: Style::new().add_modifier(Modifier::BOLD),
            title: Style::new().add_modifier(Modifier::BOLD),
            border: Style::new().add_modifier(Modifier::DIM),
            border_focus: Style::new(),
            selected: Style::new().add_modifier(Modifier::REVERSED),
            error: Style::new().add_modifier(Modifier::BOLD),
        }
    }
}
