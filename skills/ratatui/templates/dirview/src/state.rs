//! Application state and the one function that changes it, `State::update`.
//!
//! No terminal types here, so every behavior is testable without a terminal.

use std::io;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::time::SystemTime;

use crate::action::Action;
use crate::fs;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Filter,
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    Dir,
    File,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortKey {
    Name,
    Size,
    Modified,
}

impl SortKey {
    pub fn label(self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::Size => "size",
            Self::Modified => "modified",
        }
    }

    fn next(self) -> Self {
        match self {
            Self::Name => Self::Size,
            Self::Size => Self::Modified,
            Self::Modified => Self::Name,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub name: String,
    pub kind: EntryKind,
    pub size: u64,
    pub modified: Option<SystemTime>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Preview {
    Empty,
    Loading,
    Binary,
    Error(String),
    Dir { names: Vec<String>, total: usize },
    Text { lines: Vec<String>, truncated: bool },
}

struct PreviewJob {
    request: u64,
    path: PathBuf,
    kind: EntryKind,
    cancelled: Arc<AtomicBool>,
}

fn spawn_preview_worker() -> (Sender<PreviewJob>, Receiver<(u64, Preview)>) {
    let (job_tx, job_rx) = mpsc::channel::<PreviewJob>();
    let (result_tx, result_rx) = mpsc::channel::<(u64, Preview)>();

    thread::spawn(move || {
        while let Ok(job) = job_rx.recv() {
            if job.cancelled.load(AtomicOrdering::Relaxed) {
                continue;
            }
            if let Some(preview) =
                fs::preview_cancellable(&job.path, job.kind, &job.cancelled)
            {
                if !job.cancelled.load(AtomicOrdering::Relaxed) {
                    let _ = result_tx.send((job.request, preview));
                }
            }
        }
    });

    (job_tx, result_rx)
}

pub struct State {
    dir: PathBuf,
    entries: Vec<Entry>,
    /// Indices into `entries`, after filtering and hidden-file rules.
    visible: Vec<usize>,
    /// Position within `visible`.
    selected: usize,
    mode: Mode,
    filter: String,
    show_hidden: bool,
    sort: SortKey,
    preview: Preview,
    preview_tx: Sender<PreviewJob>,
    preview_rx: Receiver<(u64, Preview)>,
    preview_cancel: Option<Arc<AtomicBool>>,
    preview_request: u64,
    status: Option<String>,
    page_size: usize,
    quit: bool,
}

impl State {
    pub fn open(dir: PathBuf) -> io::Result<Self> {
        let dir = dir.canonicalize()?;
        let entries = fs::read_dir(&dir)?;
        let (preview_tx, preview_rx) = spawn_preview_worker();
        let mut state = Self {
            dir,
            entries,
            visible: Vec::new(),
            selected: 0,
            mode: Mode::Normal,
            filter: String::new(),
            show_hidden: false,
            sort: SortKey::Name,
            preview: Preview::Empty,
            preview_tx,
            preview_rx,
            preview_cancel: None,
            preview_request: 0,
            status: None,
            page_size: 10,
            quit: false,
        };
        state.rebuild(None);
        Ok(state)
    }

    pub fn dir(&self) -> &std::path::Path {
        &self.dir
    }
    pub fn mode(&self) -> Mode {
        self.mode
    }
    pub fn filter(&self) -> &str {
        &self.filter
    }
    pub fn sort(&self) -> SortKey {
        self.sort
    }
    pub fn preview(&self) -> &Preview {
        &self.preview
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
    pub fn total_len(&self) -> usize {
        self.entries.len()
    }
    pub fn visible_len(&self) -> usize {
        self.visible.len()
    }
    pub fn selected_index(&self) -> Option<usize> {
        (!self.visible.is_empty()).then_some(self.selected)
    }
    pub fn selected_entry(&self) -> Option<&Entry> {
        self.visible.get(self.selected).map(|&i| &self.entries[i])
    }
    pub fn visible_entries(&self) -> impl Iterator<Item = &Entry> {
        self.visible.iter().filter_map(|&i| self.entries.get(i))
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
            Action::Enter => self.enter(),
            Action::Parent => self.parent(),
            Action::StartFilter => self.mode = Mode::Filter,
            Action::CommitFilter => self.mode = Mode::Normal,
            Action::ClearFilter => {
                self.mode = Mode::Normal;
                if !self.filter.is_empty() {
                    self.filter.clear();
                    self.rebuild(self.selected_name());
                }
            }
            Action::FilterInput(c) => {
                self.filter.push(c);
                self.rebuild(self.selected_name());
            }
            Action::FilterBackspace => {
                self.filter.pop();
                self.rebuild(self.selected_name());
            }
            Action::ToggleHidden => {
                self.show_hidden = !self.show_hidden;
                self.rebuild(self.selected_name());
            }
            Action::CycleSort => {
                self.sort = self.sort.next();
                self.rebuild(self.selected_name());
            }
            Action::ToggleHelp => {
                self.mode = if self.mode == Mode::Help {
                    Mode::Normal
                } else {
                    Mode::Help
                };
            }
            Action::Quit => self.quit = true,
            Action::HelpScrollUp
            | Action::HelpScrollDown
            | Action::HelpPageUp
            | Action::HelpPageDown => {}
            Action::SetPageSize(_) => {}
        }
    }

    fn selected_name(&self) -> Option<String> {
        self.selected_entry().map(|e| e.name.clone())
    }

    fn select(&mut self, index: usize) {
        let last = self.visible.len().saturating_sub(1);
        let index = index.min(last);
        if index != self.selected {
            self.selected = index;
            self.refresh_preview();
        }
    }

    fn enter(&mut self) {
        let Some(entry) = self.selected_entry() else {
            return;
        };
        if entry.kind == EntryKind::Dir {
            let path = self.dir.join(&entry.name);
            self.navigate(path, None);
        } else {
            self.status = Some("not a directory".to_owned());
        }
    }

    fn parent(&mut self) {
        let Some(parent) = self.dir.parent().map(std::path::Path::to_path_buf) else {
            self.status = Some("already at the root".to_owned());
            return;
        };
        let came_from = self
            .dir
            .file_name()
            .map(|n| n.to_string_lossy().into_owned());
        self.navigate(parent, came_from);
    }

    /// Switch directory. On failure the current directory stays and the
    /// reason is shown in the footer.
    fn navigate(&mut self, path: PathBuf, select_name: Option<String>) {
        match fs::read_dir(&path) {
            Ok(entries) => {
                self.dir = path;
                self.entries = entries;
                self.filter.clear();
                self.selected = 0;
                self.rebuild(select_name);
            }
            Err(e) => self.status = Some(e.to_string()),
        }
    }

    /// Re-sort, re-filter and keep the selection on `keep` when it is still
    /// visible. Otherwise select the visible entry nearest the old position,
    /// so a filter or sort change does not highlight an unrelated file.
    fn rebuild(&mut self, keep: Option<String>) {
        let sort = self.sort;
        self.entries.sort_by_cached_key(|e| sort_key(e, sort));
        let needle = self.filter.to_lowercase();
        self.visible = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| self.show_hidden || !e.name.starts_with('.'))
            .filter(|(_, e)| needle.is_empty() || e.name.to_lowercase().contains(&needle))
            .map(|(i, _)| i)
            .collect();
        self.selected = keep
            .and_then(|name| {
                self.visible
                    .iter()
                    .position(|&i| self.entries[i].name == name)
            })
            .unwrap_or(self.selected)
            .min(self.visible.len().saturating_sub(1));
        self.refresh_preview();
    }

    fn refresh_preview(&mut self) {
        if let Some(cancelled) = self.preview_cancel.take() {
            cancelled.store(true, AtomicOrdering::Relaxed);
        }
        self.preview_request = self.preview_request.wrapping_add(1);
        let request = self.preview_request;

        let Some((path, kind)) = self
            .selected_entry()
            .map(|entry| (self.dir.join(&entry.name), entry.kind))
        else {
            self.preview = Preview::Empty;
            return;
        };

        let cancelled = Arc::new(AtomicBool::new(false));
        self.preview = Preview::Loading;
        self.preview_cancel = Some(cancelled.clone());

        if self
            .preview_tx
            .send(PreviewJob {
                request,
                path,
                kind,
                cancelled,
            })
            .is_err()
        {
            self.preview = Preview::Error("preview worker stopped".to_owned());
            self.preview_cancel = None;
        }
    }

    pub fn poll_preview(&mut self) {
        while let Ok((request, preview)) = self.preview_rx.try_recv() {
            if request == self.preview_request {
                self.preview = preview;
                self.preview_cancel = None;
            }
        }
    }
}

impl Drop for State {
    fn drop(&mut self) {
        if let Some(cancelled) = self.preview_cancel.take() {
            cancelled.store(true, AtomicOrdering::Relaxed);
        }
    }
}

/// Directories first, then by the chosen key; ties break on name.
fn sort_key(entry: &Entry, sort: SortKey) -> (bool, std::cmp::Reverse<u64>, String) {
    let name = entry.name.to_lowercase();
    let not_dir = entry.kind != EntryKind::Dir;
    match sort {
        SortKey::Name => (not_dir, std::cmp::Reverse(0), name),
        SortKey::Size => (not_dir, std::cmp::Reverse(entry.size), name),
        SortKey::Modified => {
            let secs = entry
                .modified
                .and_then(|m| m.duration_since(SystemTime::UNIX_EPOCH).ok())
                .map_or(0, |d| d.as_secs());
            (not_dir, std::cmp::Reverse(secs), name)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Result = std::result::Result<(), Box<dyn std::error::Error>>;

    /// A scratch directory: Alpha/ (with inner.txt), beta.txt, zeta.txt (big), .hidden
    struct Fixture(PathBuf);

    impl Fixture {
        fn new(name: &str) -> std::result::Result<Self, Box<dyn std::error::Error>> {
            let root = std::env::temp_dir().join(format!("dirview-{name}-{}", std::process::id()));
            let _cleanup = std::fs::remove_dir_all(&root);
            std::fs::create_dir_all(root.join("Alpha"))?;
            std::fs::write(root.join("Alpha/inner.txt"), "inner")?;
            std::fs::write(root.join("beta.txt"), "beta")?;
            std::fs::write(root.join("zeta.txt"), vec![b'z'; 900])?;
            std::fs::write(root.join(".hidden"), "h")?;
            Ok(Self(root))
        }

        fn state(&self) -> io::Result<State> {
            State::open(self.0.clone())
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _cleanup = std::fs::remove_dir_all(&self.0);
        }
    }

    fn names(s: &State) -> Vec<&str> {
        s.visible_entries().map(|e| e.name.as_str()).collect()
    }

    fn wait_for_preview(state: &mut State) -> Result {
        for _ in 0..200 {
            state.poll_preview();
            if !matches!(state.preview(), Preview::Loading) {
                return Ok(());
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        Err("preview worker did not finish".into())
    }

    #[test]
    fn dirs_come_first_and_dotfiles_are_hidden() -> Result {
        let fx = Fixture::new("order")?;
        let s = fx.state()?;
        assert_eq!(names(&s), ["Alpha", "beta.txt", "zeta.txt"]);
        assert_eq!(s.total_len(), 4);
        Ok(())
    }

    #[test]
    fn toggling_hidden_shows_dotfiles() -> Result {
        let fx = Fixture::new("hidden")?;
        let mut s = fx.state()?;
        s.update(Action::ToggleHidden);
        assert_eq!(names(&s), ["Alpha", ".hidden", "beta.txt", "zeta.txt"]);
        Ok(())
    }

    #[test]
    fn sort_cycles_and_keeps_the_selection() -> Result {
        let fx = Fixture::new("sort")?;
        let mut s = fx.state()?;
        s.update(Action::MoveBottom);
        s.update(Action::CycleSort);
        assert_eq!(s.sort(), SortKey::Size);
        assert_eq!(names(&s), ["Alpha", "zeta.txt", "beta.txt"]);
        assert_eq!(
            s.selected_entry().map(|e| e.name.as_str()),
            Some("zeta.txt")
        );
        Ok(())
    }

    #[test]
    fn selection_is_clamped() -> Result {
        let fx = Fixture::new("clamp")?;
        let mut s = fx.state()?;
        s.update(Action::MoveUp);
        assert_eq!(s.selected_index(), Some(0));
        s.update(Action::MoveBottom);
        s.update(Action::MoveDown);
        assert_eq!(s.selected_index(), Some(2));
        s.update(Action::PageUp);
        assert_eq!(s.selected_index(), Some(0));
        Ok(())
    }

    #[test]
    fn filter_narrows_and_clears() -> Result {
        let fx = Fixture::new("filter")?;
        let mut s = fx.state()?;
        s.update(Action::StartFilter);
        assert_eq!(s.mode(), Mode::Filter);
        s.update(Action::FilterInput('E'));
        s.update(Action::FilterInput('t'));
        assert_eq!(names(&s), ["beta.txt", "zeta.txt"]);
        s.update(Action::FilterBackspace);
        assert_eq!(names(&s), ["beta.txt", "zeta.txt"]);
        s.update(Action::ClearFilter);
        assert_eq!(s.mode(), Mode::Normal);
        assert_eq!(s.visible_len(), 3);
        Ok(())
    }

    #[test]
    fn enter_and_parent_round_trip() -> Result {
        let fx = Fixture::new("nav")?;
        let mut s = fx.state()?;
        s.update(Action::Enter);
        wait_for_preview(&mut s)?;
        assert_eq!(names(&s), ["inner.txt"]);
        assert_eq!(
            s.preview(),
            &Preview::Text {
                lines: vec!["inner".into()],
                truncated: false
            }
        );
        s.update(Action::Parent);
        wait_for_preview(&mut s)?;
        assert_eq!(s.selected_entry().map(|e| e.name.as_str()), Some("Alpha"));
        Ok(())
    }

    #[test]
    fn entering_a_file_reports_instead_of_failing_silently() -> Result {
        let fx = Fixture::new("file")?;
        let mut s = fx.state()?;
        s.update(Action::MoveDown);
        s.update(Action::Enter);
        assert_eq!(s.status(), Some("not a directory"));
        s.update(Action::MoveUp);
        assert_eq!(s.status(), None);
        Ok(())
    }

    #[test]
    fn empty_result_has_no_selection() -> Result {
        let fx = Fixture::new("empty")?;
        let mut s = fx.state()?;
        s.update(Action::FilterInput('q'));
        s.update(Action::FilterInput('q'));
        assert_eq!(s.selected_index(), None);
        assert_eq!(s.preview(), &Preview::Empty);
        s.update(Action::MoveDown);
        s.update(Action::Enter);
        Ok(())
    }

    #[test]
    fn large_directory_preview_runs_in_the_background() -> Result {
        let fx = Fixture::new("async-preview")?;
        let large = fx.0.join("large");
        std::fs::create_dir(&large)?;
        for index in 0..50_000 {
            std::fs::write(large.join(format!("entry-{index:05}")), b"")?;
        }

        let mut state = fx.state()?;
        state.update(Action::MoveDown);
        assert!(matches!(state.preview(), Preview::Loading));
        wait_for_preview(&mut state)?;

        match state.preview() {
            Preview::Dir { names, total } => {
                assert_eq!(*total, 50_000);
                assert_eq!(names.len(), 200);
            }
            other => panic!("expected completed directory preview, got {other:?}"),
        }
        Ok(())
    }

    #[test]
    fn missing_directory_is_an_error_not_a_panic() {
        assert!(State::open(PathBuf::from("/definitely/not/here")).is_err());
    }
}
