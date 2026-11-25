//! Colors by role. This is the only file that names a color.
//!
//! Three layers: monochrome works anywhere and is used under `NO_COLOR`; the
//! default uses the 16 ANSI colors so it follows the user's palette; the
//! opt-in Catppuccin Mocha theme uses truecolor. No app-wide background is
//! forced; only the selected row is highlighted, so terminal transparency and
//! the user's own palette stay intact.
//!
//! Catppuccin Mocha palette values: https://catppuccin.com/palette/

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
    pub done: Style,
    pub done_mark: Style,
    pub error: Style,
}

impl Theme {
    /// `NO_COLOR` always wins. Otherwise `TODO_THEME=catppuccin` opts into the
    /// truecolor theme; anything else gets the ANSI default.
    pub fn detect() -> Self {
        let no_color = std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty());
        if no_color {
            return Self::mono();
        }
        let catppuccin = std::env::var_os("TODO_THEME").is_some_and(|v| v == "catppuccin");
        if catppuccin {
            Self::catppuccin()
        } else {
            Self::ansi()
        }
    }

    /// Catppuccin Mocha, with one accent (Blue) for selection and prompt,
    /// Green for done, Red for errors and Mauve for the title. Secondary text
    /// is Subtext1, not a second accent.
    pub fn catppuccin() -> Self {
        let text = Color::Rgb(205, 214, 244); // Text
        let subtext = Color::Rgb(186, 194, 222); // Subtext1
        let overlay = Color::Rgb(108, 112, 134); // Overlay0
        let blue = Color::Rgb(137, 180, 250); // Blue
        Self {
            text: Style::new().fg(text),
            muted: Style::new().fg(subtext),
            accent: Style::new().fg(blue).add_modifier(Modifier::BOLD),
            title: Style::new()
                .fg(Color::Rgb(203, 166, 247))
                .add_modifier(Modifier::BOLD),
            border: Style::new().fg(overlay),
            border_focus: Style::new().fg(blue),
            selected: Style::new()
                .fg(text)
                .bg(Color::Rgb(69, 71, 90))
                .add_modifier(Modifier::BOLD),
            done: Style::new().fg(subtext).add_modifier(Modifier::DIM),
            done_mark: Style::new().fg(Color::Rgb(166, 227, 161)), // Green
            error: Style::new().fg(Color::Rgb(243, 139, 168)),     // Red
        }
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
            done: Style::new().add_modifier(Modifier::DIM),
            done_mark: Style::new().fg(Color::Green),
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
            done: Style::new().add_modifier(Modifier::DIM),
            done_mark: Style::new().add_modifier(Modifier::BOLD),
            error: Style::new().add_modifier(Modifier::BOLD),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catppuccin_uses_one_accent_and_palette_colors() {
        let theme = Theme::catppuccin();
        assert_eq!(theme.accent.fg, Some(Color::Rgb(137, 180, 250)));
        assert_eq!(theme.border_focus.fg, theme.accent.fg);
        assert_eq!(theme.done_mark.fg, Some(Color::Rgb(166, 227, 161)));
        assert_eq!(theme.error.fg, Some(Color::Rgb(243, 139, 168)));
        assert_eq!(theme.title.fg, Some(Color::Rgb(203, 166, 247)));
        assert_eq!(theme.text.fg, Some(Color::Rgb(205, 214, 244)));
    }

    #[test]
    fn mono_has_no_colors() {
        let theme = Theme::mono();
        for style in [
            theme.text,
            theme.muted,
            theme.accent,
            theme.title,
            theme.border,
            theme.border_focus,
            theme.selected,
            theme.done,
            theme.done_mark,
            theme.error,
        ] {
            assert_eq!(style.fg, None);
            assert_eq!(style.bg, None);
        }
    }
}
