//! Render every screen into a headless buffer and read it as text.

use std::path::PathBuf;

use dirview::action::Action;
use dirview::state::State;
use dirview::theme::Theme;
use dirview::ui::{self, ViewState};
use ratatui::Terminal;
use ratatui::backend::TestBackend;

type Result = std::result::Result<(), Box<dyn std::error::Error>>;

struct Fixture(PathBuf);

impl Fixture {
    fn new(name: &str) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        let root =
            std::env::temp_dir().join(format!("dirview-render-{name}-{}", std::process::id()));
        let _cleanup = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("docs"))?;
        std::fs::write(root.join("docs/guide.md"), "# Guide\n")?;
        std::fs::write(root.join("notes.txt"), "first line\nsecond line\n")?;
        std::fs::write(root.join("blob.bin"), [0u8, 1, 2, 3])?;
        std::fs::write(root.join("日本語のファイル名.txt"), "x")?;
        Ok(Self(root))
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _cleanup = std::fs::remove_dir_all(&self.0);
    }
}

fn screen(
    state: &State,
    width: u16,
    height: u16,
) -> std::result::Result<String, Box<dyn std::error::Error>> {
    let mut terminal = Terminal::new(TestBackend::new(width, height))?;
    let mut view = ViewState::default();
    let theme = Theme::mono();
    terminal.draw(|f| ui::render(f, state, &mut view, &theme))?;
    Ok(ui::buffer_text(terminal.backend().buffer()))
}

#[test]
fn wide_layout_shows_list_and_preview() -> Result {
    let fx = Fixture::new("wide")?;
    let mut state = State::open(fx.0.clone())?;
    state.update(Action::MoveDown);
    let text = screen(&state, 100, 20)?;
    assert!(text.contains("docs/"), "{text}");
    assert!(text.contains("notes.txt"), "{text}");
    assert!(text.contains("4 items"), "{text}");
    assert!(text.contains("by name"), "{text}");
    Ok(())
}

#[test]
fn text_file_preview_is_shown() -> Result {
    let fx = Fixture::new("preview")?;
    let mut state = State::open(fx.0.clone())?;
    state.update(Action::StartFilter);
    for c in "notes".chars() {
        state.update(Action::FilterInput(c));
    }
    let text = screen(&state, 100, 20)?;
    assert!(text.contains("first line"), "{text}");
    assert!(text.contains("1 of 4 match"), "{text}");
    Ok(())
}

#[test]
fn binary_file_is_not_dumped() -> Result {
    let fx = Fixture::new("binary")?;
    let mut state = State::open(fx.0.clone())?;
    state.update(Action::MoveDown);
    let text = screen(&state, 100, 20)?;
    assert!(text.contains("binary file"), "{text}");
    Ok(())
}

#[test]
fn narrow_layout_drops_the_preview() -> Result {
    let fx = Fixture::new("narrow")?;
    let state = State::open(fx.0.clone())?;
    let text = screen(&state, 60, 12)?;
    assert!(text.contains("docs/"), "{text}");
    assert!(!text.contains("items\n  "), "{text}");
    Ok(())
}

#[test]
fn tiny_terminal_gets_a_message_not_a_panic() -> Result {
    let fx = Fixture::new("tiny")?;
    let state = State::open(fx.0.clone())?;
    let text = screen(&state, 20, 5)?;
    assert!(text.contains("terminal too small"), "{text}");
    Ok(())
}

#[test]
fn every_size_renders_without_panicking() -> Result {
    let fx = Fixture::new("sizes")?;
    let mut state = State::open(fx.0.clone())?;
    for action in [Action::MoveDown, Action::StartFilter, Action::ToggleHelp] {
        state.update(action);
        for (w, h) in [(1, 1), (39, 7), (40, 8), (79, 24), (80, 24), (200, 60)] {
            screen(&state, w, h)?;
        }
    }
    Ok(())
}

#[test]
fn empty_filter_result_explains_itself() -> Result {
    let fx = Fixture::new("nomatch")?;
    let mut state = State::open(fx.0.clone())?;
    state.update(Action::StartFilter);
    for c in "zzz".chars() {
        state.update(Action::FilterInput(c));
    }
    let text = screen(&state, 100, 20)?;
    assert!(text.contains("no match for \"zzz\""), "{text}");
    Ok(())
}

#[test]
fn help_overlay_lists_the_real_keys() -> Result {
    let fx = Fixture::new("help")?;
    let mut state = State::open(fx.0.clone())?;
    state.update(Action::ToggleHelp);
    let text = screen(&state, 100, 40)?;
    for expected in [
        "Keys",
        "Filter by name",
        "Cycle sort",
        "Quit",
        "Scroll one page down",
        "Scroll one page up",
        "Esc close",
    ] {
        assert!(text.contains(expected), "missing {expected:?}\n{text}");
    }
    Ok(())
}

#[test]
fn help_scroll_clamps_and_uses_the_visible_page_size() -> Result {
    let fx = Fixture::new("help-scroll")?;
    let mut state = State::open(fx.0.clone())?;
    state.update(Action::ToggleHelp);
    let mut terminal = Terminal::new(TestBackend::new(40, 8))?;
    let mut view = ViewState::default();
    let theme = Theme::mono();

    terminal.draw(|frame| ui::render(frame, &state, &mut view, &theme))?;
    let page_size = view.help_page_size();
    assert!(page_size > 0);

    view.scroll_help(isize::MAX);
    terminal.draw(|frame| ui::render(frame, &state, &mut view, &theme))?;
    let end = view.help_offset();
    assert!(end > 0);

    view.scroll_help(-1);
    terminal.draw(|frame| ui::render(frame, &state, &mut view, &theme))?;
    assert_eq!(view.help_offset(), end - 1);

    view.scroll_help(-(page_size as isize));
    terminal.draw(|frame| ui::render(frame, &state, &mut view, &theme))?;
    assert!(view.help_offset() < end - 1);
    Ok(())
}

#[test]
fn wide_characters_do_not_break_alignment() -> Result {
    let fx = Fixture::new("cjk")?;
    let state = State::open(fx.0.clone())?;
    let text = screen(&state, 100, 20)?;
    assert!(text.contains("日本語のファイル名.txt"), "{text}");
    Ok(())
}
