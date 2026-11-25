//! Application state and the one function that changes it, `State::update`.
//!
//! No terminal types here, so every behavior is testable without a terminal.
//! Every mutation is saved immediately; a failed save keeps the in-memory
//! change and shows the reason in the footer.

use std::io;
use std::path::PathBuf;

use crate::action::Action;
use crate::store;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Add,
    ConfirmDelete,
    Help,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Task {
    pub text: String,
    pub done: bool,
}

#[derive(Debug)]
pub struct State {
    path: PathBuf,
    tasks: Vec<Task>,
    selected: usize,
    mode: Mode,
    input: String,
    status: Option<String>,
    page_size: usize,
    quit: bool,
}

impl State {
    pub fn open(path: PathBuf) -> io::Result<Self> {
        let tasks = store::load(&path)?;
        Ok(Self {
            path,
            tasks,
            selected: 0,
            mode: Mode::Normal,
            input: String::new(),
            status: None,
            page_size: 10,
            quit: false,
        })
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }
    pub fn input(&self) -> &str {
        &self.input
    }
    pub fn status(&self) -> Option<&str> {
        self.status.as_deref()
    }
    pub fn page_size(&self) -> usize {
        self.page_size
    }
    pub fn should_quit(&self) -> bool {
        self.quit
    }
    pub fn tasks(&self) -> &[Task] {
        &self.tasks
    }
    pub fn len(&self) -> usize {
        self.tasks.len()
    }
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }
    pub fn remaining(&self) -> usize {
        self.tasks.iter().filter(|t| !t.done).count()
    }
    pub fn selected_index(&self) -> Option<usize> {
        (!self.tasks.is_empty()).then_some(self.selected)
    }
    pub fn selected_task(&self) -> Option<&Task> {
        self.tasks.get(self.selected)
    }

    pub fn update(&mut self, action: Action) {
        if let Action::SetPageSize(rows) = action {
            self.page_size = rows;
            return;
        }
        self.status = None;
        let half_page = (self.page_size / 2).max(1);
        match action {
            Action::MoveDown => self.select(self.selected.saturating_add(1)),
            Action::MoveUp => self.select(self.selected.saturating_sub(1)),
            Action::MoveTop => self.select(0),
            Action::MoveBottom => self.select(usize::MAX),
            Action::PageUp => self.select(self.selected.saturating_sub(half_page)),
            Action::PageDown => self.select(self.selected.saturating_add(half_page)),
            Action::StartAdd => {
                self.input.clear();
                self.mode = Mode::Add;
            }
            Action::AddInput(c) => {
                // A task is one line; reject separators and control chars.
                if !c.is_control() {
                    self.input.push(c);
                }
            }
            Action::AddBackspace => {
                self.input.pop();
            }
            Action::CommitAdd => {
                let text = self.input.trim().to_owned();
                if text.is_empty() {
                    self.status = Some("empty task not added".to_owned());
                } else {
                    self.tasks.push(Task { text, done: false });
                    self.selected = self.tasks.len() - 1;
                    self.persist("task added");
                }
                self.input.clear();
                self.mode = Mode::Normal;
            }
            Action::ToggleDone => {
                if let Some(task) = self.tasks.get_mut(self.selected) {
                    task.done = !task.done;
                    self.persist("updated");
                }
            }
            Action::StartDelete => {
                if self.selected_task().is_some() {
                    self.mode = Mode::ConfirmDelete;
                }
            }
            Action::ConfirmDelete => {
                if self.selected_task().is_some() {
                    self.tasks.remove(self.selected);
                    self.selected = self.selected.min(self.tasks.len().saturating_sub(1));
                    self.persist("task deleted");
                }
                self.mode = Mode::Normal;
            }
            Action::Cancel => {
                self.input.clear();
                self.mode = Mode::Normal;
            }
            Action::ToggleHelp => {
                self.mode = if self.mode == Mode::Help {
                    Mode::Normal
                } else {
                    Mode::Help
                };
            }
            Action::Quit => self.quit = true,
            Action::SetPageSize(_) => {}
        }
    }

    fn select(&mut self, index: usize) {
        self.selected = index.min(self.tasks.len().saturating_sub(1));
    }

    fn persist(&mut self, ok_status: &str) {
        match store::save(&self.path, &self.tasks) {
            Ok(()) => self.status = Some(ok_status.to_owned()),
            Err(e) => self.status = Some(format!("cannot save: {e}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Result = std::result::Result<(), Box<dyn std::error::Error>>;
    type BoxResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    struct Fixture(PathBuf);

    impl Fixture {
        fn new(name: &str) -> BoxResult<Self> {
            let root =
                std::env::temp_dir().join(format!("todo-state-{name}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&root);
            std::fs::create_dir_all(&root)?;
            std::fs::write(
                root.join("tasks.txt"),
                "- buy milk\n+ call mom\n- write tests\n",
            )?;
            Ok(Self(root.join("tasks.txt")))
        }

        fn state(&self) -> io::Result<State> {
            State::open(self.0.clone())
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            if let Some(parent) = self.0.parent() {
                let _ = std::fs::remove_dir_all(parent);
            }
        }
    }

    fn add(state: &mut State, text: &str) {
        state.update(Action::StartAdd);
        for c in text.chars() {
            state.update(Action::AddInput(c));
        }
        state.update(Action::CommitAdd);
    }

    #[test]
    fn loads_tasks_in_file_order() -> Result {
        let fx = Fixture::new("load")?;
        let s = fx.state()?;
        assert_eq!(s.len(), 3);
        assert_eq!(s.remaining(), 2);
        assert_eq!(s.selected_task().map(|t| t.text.as_str()), Some("buy milk"));
        Ok(())
    }

    #[test]
    fn add_persists_and_selects_the_new_task() -> Result {
        let fx = Fixture::new("add")?;
        let mut s = fx.state()?;
        add(&mut s, "new task");
        assert_eq!(s.len(), 4);
        assert_eq!(s.selected_task().map(|t| t.text.as_str()), Some("new task"));
        assert_eq!(s.status(), Some("task added"));
        let reloaded = fx.state()?;
        assert_eq!(reloaded.len(), 4);
        assert_eq!(
            reloaded.tasks().last().map(|t| t.text.as_str()),
            Some("new task")
        );
        Ok(())
    }

    #[test]
    fn toggle_done_persists() -> Result {
        let fx = Fixture::new("toggle")?;
        let mut s = fx.state()?;
        s.update(Action::ToggleDone);
        assert!(s.selected_task().is_some_and(|t| t.done));
        let reloaded = fx.state()?;
        assert!(reloaded.tasks().first().is_some_and(|t| t.done));
        Ok(())
    }

    #[test]
    fn delete_is_confirmed_and_persists() -> Result {
        let fx = Fixture::new("delete")?;
        let mut s = fx.state()?;
        s.update(Action::StartDelete);
        assert_eq!(s.mode(), Mode::ConfirmDelete);
        s.update(Action::Cancel);
        assert_eq!(s.mode(), Mode::Normal);
        assert_eq!(s.len(), 3);
        s.update(Action::StartDelete);
        s.update(Action::ConfirmDelete);
        assert_eq!(s.len(), 2);
        assert_eq!(s.selected_task().map(|t| t.text.as_str()), Some("call mom"));
        let reloaded = fx.state()?;
        assert_eq!(reloaded.len(), 2);
        Ok(())
    }

    #[test]
    fn empty_input_is_not_added() -> Result {
        let fx = Fixture::new("empty")?;
        let mut s = fx.state()?;
        add(&mut s, "   ");
        assert_eq!(s.len(), 3);
        assert_eq!(s.status(), Some("empty task not added"));
        Ok(())
    }

    #[test]
    fn selection_is_clamped_after_deleting_the_last_item() -> Result {
        let fx = Fixture::new("clamp")?;
        let mut s = fx.state()?;
        s.update(Action::MoveBottom);
        s.update(Action::StartDelete);
        s.update(Action::ConfirmDelete);
        assert_eq!(s.selected_index(), Some(1));
        Ok(())
    }

    #[test]
    fn missing_file_starts_empty() -> Result {
        let root = std::env::temp_dir().join(format!("todo-new-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root)?;
        let mut s = State::open(root.join("tasks.txt"))?;
        assert_eq!(s.selected_index(), None);
        add(&mut s, "first");
        assert_eq!(s.len(), 1);
        let _ = std::fs::remove_dir_all(&root);
        Ok(())
    }
}
