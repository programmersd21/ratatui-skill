//! Render every screen into a headless buffer and read it as text.

use std::path::PathBuf;

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::style::Color;
use todo_tui::action::Action;
use todo_tui::state::State;
use todo_tui::theme::Theme;
use todo_tui::ui::{self, ViewState};

type Result = std::result::Result<(), Box<dyn std::error::Error>>;
type BoxResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

struct Fixture(PathBuf);

impl Fixture {
    fn new(name: &str) -> BoxResult<Self> {
        let root = std::env::temp_dir().join(format!("todo-render-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root)?;
        std::fs::write(
            root.join("tasks.txt"),
            "- buy milk\n+ call mom\n- write the readme\n",
        )?;
        Ok(Self(root.join("tasks.txt")))
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if let Some(parent) = self.0.parent() {
            let _ = std::fs::remove_dir_all(parent);
        }
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
fn normal_screen_shows_tasks_and_counts() -> Result {
    let fx = Fixture::new("normal")?;
    let state = State::open(fx.0.clone())?;
    let text = screen(&state, 80, 24)?;
    assert!(text.contains("[ ] buy milk"), "{text}");
    assert!(text.contains("[x] call mom"), "{text}");
    assert!(text.contains("2 left"), "{text}");
    assert!(text.contains("3 total"), "{text}");
    Ok(())
}

#[test]
fn add_prompt_shows_the_input() -> Result {
    let fx = Fixture::new("add")?;
    let mut state = State::open(fx.0.clone())?;
    state.update(Action::StartAdd);
    for c in "new task".chars() {
        state.update(Action::AddInput(c));
    }
    let text = screen(&state, 80, 24)?;
    assert!(text.contains("> new task"), "{text}");
    assert!(text.contains("Enter save"), "{text}");
    Ok(())
}

#[test]
fn delete_confirmation_names_the_task() -> Result {
    let fx = Fixture::new("confirm")?;
    let mut state = State::open(fx.0.clone())?;
    state.update(Action::StartDelete);
    let text = screen(&state, 80, 24)?;
    assert!(text.contains("Delete \"buy milk\"?"), "{text}");
    assert!(text.contains("delete"), "{text}");
    assert!(text.contains("keep"), "{text}");
    Ok(())
}

#[test]
fn empty_list_explains_itself() -> Result {
    let root = std::env::temp_dir().join(format!("todo-empty-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root)?;
    let state = State::open(root.join("tasks.txt"))?;
    let text = screen(&state, 80, 24)?;
    assert!(text.contains("no tasks yet"), "{text}");
    assert!(text.contains("press a to add one"), "{text}");
    assert!(text.contains("0 left"), "{text}");
    let _ = std::fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn help_overlay_lists_the_real_keys() -> Result {
    let fx = Fixture::new("help")?;
    let mut state = State::open(fx.0.clone())?;
    state.update(Action::ToggleHelp);
    let text = screen(&state, 80, 30)?;
    for expected in [
        "Keys",
        "Add a task",
        "Mark done or pending",
        "Quit",
        "Esc close",
    ] {
        assert!(text.contains(expected), "missing {expected:?}\n{text}");
    }
    Ok(())
}

#[test]
fn tiny_terminal_gets_a_message_not_a_panic() -> Result {
    let fx = Fixture::new("tiny")?;
    let state = State::open(fx.0.clone())?;
    let text = screen(&state, 20, 5)?;
    assert!(text.contains("terminal too small"), "{text}");
    assert!(text.contains("have 20x5"), "{text}");
    Ok(())
}

#[test]
fn every_size_renders_without_panicking() -> Result {
    let fx = Fixture::new("sizes")?;
    let mut state = State::open(fx.0.clone())?;
    for action in [
        Action::MoveDown,
        Action::StartAdd,
        Action::StartDelete,
        Action::ToggleHelp,
    ] {
        state.update(action);
        for (w, h) in [(1, 1), (39, 7), (40, 8), (80, 24), (200, 60)] {
            screen(&state, w, h)?;
        }
        state.update(Action::Cancel);
    }
    Ok(())
}

#[test]
fn wide_characters_do_not_break_alignment() -> Result {
    let root = std::env::temp_dir().join(format!("todo-cjk-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root)?;
    std::fs::write(root.join("tasks.txt"), "- 日本語のタスク\n")?;
    let state = State::open(root.join("tasks.txt"))?;
    let text = screen(&state, 80, 24)?;
    assert!(text.contains("日本語のタスク"), "{text}");
    let _ = std::fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn catppuccin_theme_paints_palette_colors() -> Result {
    let fx = Fixture::new("catppuccin")?;
    let state = State::open(fx.0.clone())?;
    let mut terminal = Terminal::new(TestBackend::new(80, 24))?;
    let theme = Theme::catppuccin();
    let mut view = ViewState::default();
    terminal.draw(|f| ui::render(f, &state, &mut view, &theme))?;

    let blue = Color::Rgb(137, 180, 250);
    let green = Color::Rgb(166, 227, 161);
    let buffer = terminal.backend().buffer();
    let has = |color: Color| {
        buffer
            .content()
            .iter()
            .any(|cell| cell.fg == color || cell.bg == color)
    };
    assert!(has(blue), "accent blue missing");
    assert!(has(green), "done green missing");
    Ok(())
}
