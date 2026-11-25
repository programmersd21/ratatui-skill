//! The single table of key bindings.
//!
//! Input dispatch, footer hints and the help overlay all come from `bindings`,
//! so a key that works is always documented and a documented key always works.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::action::Action;
use crate::state::Mode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Key {
    code: KeyCode,
    modifiers: KeyModifiers,
}

const fn plain(code: KeyCode) -> Key {
    Key {
        code,
        modifiers: KeyModifiers::NONE,
    }
}

const fn ch(c: char) -> Key {
    plain(KeyCode::Char(c))
}

const fn ctrl(c: char) -> Key {
    Key {
        code: KeyCode::Char(c),
        modifiers: KeyModifiers::CONTROL,
    }
}

/// Short text for the footer: the keys, then what they do.
#[derive(Debug, Clone, Copy)]
pub struct Hint {
    pub keys: &'static str,
    pub text: &'static str,
}

#[derive(Debug)]
pub struct Binding {
    pub keys: &'static [Key],
    pub action: Action,
    /// How the keys are written in the help overlay.
    pub label: &'static str,
    pub help: &'static str,
    /// Shown in the footer when set.
    pub hint: Option<Hint>,
}

const fn hint(keys: &'static str, text: &'static str) -> Option<Hint> {
    Some(Hint { keys, text })
}

static NORMAL: &[Binding] = &[
    Binding {
        keys: &[ch('j'), plain(KeyCode::Down)],
        action: Action::MoveDown,
        label: "j / Down",
        help: "Move down",
        hint: hint("j/k", "move"),
    },
    Binding {
        keys: &[ch('k'), plain(KeyCode::Up)],
        action: Action::MoveUp,
        label: "k / Up",
        help: "Move up",
        hint: None,
    },
    Binding {
        keys: &[ch('g'), plain(KeyCode::Home)],
        action: Action::MoveTop,
        label: "g / Home",
        help: "Jump to top",
        hint: None,
    },
    Binding {
        keys: &[ch('G'), plain(KeyCode::End)],
        action: Action::MoveBottom,
        label: "G / End",
        help: "Jump to bottom",
        hint: None,
    },
    Binding {
        keys: &[ctrl('u'), plain(KeyCode::PageUp)],
        action: Action::PageUp,
        label: "Ctrl-u / PgUp",
        help: "Page up",
        hint: None,
    },
    Binding {
        keys: &[ctrl('d'), plain(KeyCode::PageDown)],
        action: Action::PageDown,
        label: "Ctrl-d / PgDn",
        help: "Page down",
        hint: None,
    },
    Binding {
        keys: &[ch('a')],
        action: Action::StartAdd,
        label: "a",
        help: "Add a task",
        hint: hint("a", "add"),
    },
    Binding {
        keys: &[plain(KeyCode::Char(' '))],
        action: Action::ToggleDone,
        label: "Space",
        help: "Mark done or pending",
        hint: hint("Space", "toggle"),
    },
    Binding {
        keys: &[ch('d')],
        action: Action::StartDelete,
        label: "d",
        help: "Delete the selected task",
        hint: hint("d", "delete"),
    },
    Binding {
        keys: &[ch('?')],
        action: Action::ToggleHelp,
        label: "?",
        help: "Show this help",
        hint: hint("?", "help"),
    },
    Binding {
        keys: &[ch('q')],
        action: Action::Quit,
        label: "q / Ctrl-c",
        help: "Quit",
        hint: hint("q", "quit"),
    },
];

static ADD: &[Binding] = &[
    Binding {
        keys: &[plain(KeyCode::Enter)],
        action: Action::CommitAdd,
        label: "Enter",
        help: "Save the task",
        hint: hint("Enter", "save"),
    },
    Binding {
        keys: &[plain(KeyCode::Esc)],
        action: Action::Cancel,
        label: "Esc",
        help: "Cancel",
        hint: hint("Esc", "cancel"),
    },
    Binding {
        keys: &[plain(KeyCode::Backspace)],
        action: Action::AddBackspace,
        label: "Backspace",
        help: "Delete a character",
        hint: None,
    },
];

static CONFIRM: &[Binding] = &[
    Binding {
        keys: &[ch('y')],
        action: Action::ConfirmDelete,
        label: "y",
        help: "Yes, delete this task",
        hint: hint("y", "delete"),
    },
    Binding {
        keys: &[plain(KeyCode::Esc), ch('n')],
        action: Action::Cancel,
        label: "n / Esc",
        help: "No, keep it",
        hint: hint("n", "keep"),
    },
];

static HELP: &[Binding] = &[Binding {
    keys: &[plain(KeyCode::Esc), ch('?'), ch('q')],
    action: Action::ToggleHelp,
    label: "Esc / ? / q",
    help: "Close help",
    hint: hint("Esc", "close"),
}];

pub fn bindings(mode: Mode) -> &'static [Binding] {
    match mode {
        Mode::Normal => NORMAL,
        Mode::Add => ADD,
        Mode::ConfirmDelete => CONFIRM,
        Mode::Help => HELP,
    }
}

/// Translate a key event into an action for the given mode.
pub fn resolve(mode: Mode, event: KeyEvent) -> Option<Action> {
    // Windows reports both press and release; ignore releases.
    if !matches!(event.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
        return None;
    }
    if event.code == KeyCode::Char('c') && event.modifiers.contains(KeyModifiers::CONTROL) {
        return Some(Action::Quit);
    }
    // Shift is already encoded in the character ('G' not 'g'), so drop it.
    let modifiers = match event.code {
        KeyCode::Char(_) => event.modifiers - KeyModifiers::SHIFT,
        _ => event.modifiers,
    };
    let pressed = Key {
        code: event.code,
        modifiers,
    };
    bindings(mode)
        .iter()
        .find(|b| b.keys.contains(&pressed))
        .map(|b| b.action)
        .or(match (mode, event.code) {
            (Mode::Add, KeyCode::Char(c)) if modifiers.is_empty() => Some(Action::AddInput(c)),
            _ => None,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    const MODES: [Mode; 4] = [Mode::Normal, Mode::Add, Mode::ConfirmDelete, Mode::Help];

    fn press(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }

    #[test]
    fn no_key_is_bound_twice_in_a_mode() {
        for mode in MODES {
            let mut seen = Vec::new();
            for key in bindings(mode).iter().flat_map(|b| b.keys) {
                assert!(!seen.contains(key), "{key:?} bound twice in {mode:?}");
                seen.push(*key);
            }
        }
    }

    #[test]
    fn every_bound_key_resolves_to_its_action() {
        for mode in MODES {
            for binding in bindings(mode) {
                for key in binding.keys {
                    let got = resolve(mode, press(key.code, key.modifiers));
                    assert_eq!(got, Some(binding.action), "{key:?} in {mode:?}");
                }
            }
        }
    }

    #[test]
    fn every_binding_is_documented() {
        for mode in MODES {
            for b in bindings(mode) {
                assert!(!b.label.is_empty() && !b.help.is_empty());
            }
        }
    }

    #[test]
    fn every_mode_shows_at_least_one_hint() {
        for mode in MODES {
            assert!(bindings(mode).iter().any(|b| b.hint.is_some()));
        }
    }

    #[test]
    fn ctrl_c_quits_everywhere() {
        for mode in MODES {
            let got = resolve(mode, press(KeyCode::Char('c'), KeyModifiers::CONTROL));
            assert_eq!(got, Some(Action::Quit));
        }
    }

    #[test]
    fn shifted_letters_match() {
        let got = resolve(Mode::Normal, press(KeyCode::Char('G'), KeyModifiers::SHIFT));
        assert_eq!(got, Some(Action::MoveBottom));
    }

    #[test]
    fn release_events_are_ignored() {
        let mut event = press(KeyCode::Char('q'), KeyModifiers::NONE);
        event.kind = KeyEventKind::Release;
        assert_eq!(resolve(Mode::Normal, event), None);
    }

    #[test]
    fn add_mode_types_letters_that_are_commands_elsewhere() {
        let got = resolve(Mode::Add, press(KeyCode::Char('q'), KeyModifiers::NONE));
        assert_eq!(got, Some(Action::AddInput('q')));
    }
}
