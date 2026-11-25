//! Reading and writing the todo file. No terminal types in here.
//!
//! Format: one task per line, `+ ` for done and `- ` for pending, then the
//! text. Control characters in the text are replaced so a hand-edited file
//! cannot move the cursor or inject escape sequences into the terminal.

use std::io::{self, Write};
use std::path::Path;

use crate::state::Task;

const DONE_MARK: char = '+';
const TODO_MARK: char = '-';

pub fn load(path: &Path) -> io::Result<Vec<Task>> {
    let content = match std::fs::read_to_string(path) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e),
    };
    Ok(content.lines().filter_map(parse_line).collect())
}

fn parse_line(line: &str) -> Option<Task> {
    let mut chars = line.chars();
    let mark = chars.next()?;
    if chars.next() != Some(' ') {
        return None;
    }
    let done = match mark {
        DONE_MARK => true,
        TODO_MARK => false,
        _ => return None,
    };
    let text: String = chars
        .map(|c| if c.is_control() { '\u{b7}' } else { c })
        .collect();
    if text.is_empty() {
        None
    } else {
        Some(Task { text, done })
    }
}

/// Write every task, then atomically replace the file. A crash mid-write
/// leaves the previous file intact.
pub fn save(path: &Path, tasks: &[Task]) -> io::Result<()> {
    let mut out = String::new();
    for task in tasks {
        let mark = if task.done { DONE_MARK } else { TODO_MARK };
        out.push(mark);
        out.push(' ');
        for c in task.text.chars() {
            if c.is_control() {
                out.push('\u{b7}');
            } else {
                out.push(c);
            }
        }
        out.push('\n');
    }

    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "todo-tui.txt".to_owned());
    let tmp = parent.join(format!(".{name}.{}.tmp", std::process::id()));
    let written = (|| -> io::Result<()> {
        let mut file = std::fs::File::create(&tmp)?;
        file.write_all(out.as_bytes())?;
        file.flush()?;
        file.sync_all()
    })();
    let renamed = written.and_then(|()| std::fs::rename(&tmp, path));
    if let Err(error) = &renamed {
        if let Err(cleanup_error) = std::fs::remove_file(&tmp) {
            return Err(io::Error::other(format!(
                "cannot save {}: {error}; also cannot remove temporary file {}: {cleanup_error}",
                path.display(),
                tmp.display()
            )));
        }
        return Err(io::Error::other(format!(
            "cannot save {}: {error}",
            path.display()
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_preserves_text_and_done_flags() -> std::io::Result<()> {
        let root = std::env::temp_dir().join(format!("todo-store-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root)?;
        let path = root.join("tasks.txt");

        let tasks = vec![
            Task {
                text: "buy milk".into(),
                done: false,
            },
            Task {
                text: "call mom".into(),
                done: true,
            },
        ];
        save(&path, &tasks)?;
        assert_eq!(load(&path)?, tasks);
        let _ = std::fs::remove_dir_all(&root);
        Ok(())
    }

    #[test]
    fn missing_file_is_an_empty_list() -> std::io::Result<()> {
        let root = std::env::temp_dir().join(format!("todo-missing-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root)?;
        let tasks = load(&root.join("nope.txt"))?;
        assert!(tasks.is_empty());
        let _ = std::fs::remove_dir_all(&root);
        Ok(())
    }

    #[test]
    fn control_characters_in_a_hand_edited_file_are_neutralized() {
        let task = parse_line("- a\u{1b}[31mb").expect("a task");
        assert_eq!(task.text, "a\u{b7}[31mb");
        assert!(!task.done);
        assert!(parse_line("garbage").is_none());
        assert!(parse_line("- ").is_none());
    }

    #[test]
    fn save_fails_clearly_when_the_parent_is_missing_and_leaves_no_temp() {
        let root = std::env::temp_dir().join(format!("todo-store-missing-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).ok();
        let target = root.join("nested/missing/tasks.txt");
        let tasks = vec![Task {
            text: "x".into(),
            done: false,
        }];
        assert!(save(&target, &tasks).is_err());
        let leftovers: Vec<_> = std::fs::read_dir(&root)
            .map(|rd| rd.flatten().map(|e| e.path()).collect())
            .unwrap_or_default();
        assert!(
            leftovers.is_empty(),
            "no temp files may remain: {leftovers:?}"
        );
        let _ = std::fs::remove_dir_all(&root);
    }
}
