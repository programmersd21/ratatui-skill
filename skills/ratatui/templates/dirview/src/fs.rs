//! Reading the file system. No terminal types in here.

use std::cmp::Ordering;
use std::io::{self, Read};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};
use std::time::SystemTime;

use crate::state::{Entry, EntryKind, Preview};
use crate::ui::text::sanitize;

/// How much of a file the preview reads. Enough to fill a pane, small enough
/// that selecting a multi-gigabyte log does not stall the UI.
const PREVIEW_BYTES: u64 = 16 * 1024;
const PREVIEW_LINES: usize = 200;
const PREVIEW_NAMES: usize = 200;

pub fn read_dir(dir: &Path) -> io::Result<Vec<Entry>> {
    let mut entries = Vec::new();
    for item in std::fs::read_dir(dir)? {
        let item = item?;
        // Follow symlinks so a link to a directory opens like a directory.
        let meta = item.path().metadata().or_else(|_| item.metadata());
        let (kind, size, modified) = match meta {
            Ok(m) => (kind_of(&m), m.len(), m.modified().ok()),
            Err(_) => (EntryKind::Other, 0, None::<SystemTime>),
        };
        entries.push(Entry {
            name: item.file_name().to_string_lossy().into_owned(),
            kind,
            size,
            modified,
        });
    }
    Ok(entries)
}

fn kind_of(meta: &std::fs::Metadata) -> EntryKind {
    if meta.is_dir() {
        EntryKind::Dir
    } else if meta.is_file() {
        EntryKind::File
    } else {
        EntryKind::Other
    }
}

/// Build the preview for an entry. Never blocks on special files.
pub fn preview(path: &Path, kind: EntryKind) -> Preview {
    let cancelled = AtomicBool::new(false);
    preview_cancellable(path, kind, &cancelled).unwrap_or(Preview::Empty)
}

pub(crate) fn preview_cancellable(
    path: &Path,
    kind: EntryKind,
    cancelled: &AtomicBool,
) -> Option<Preview> {
    if cancelled.load(AtomicOrdering::Relaxed) {
        return None;
    }
    match kind {
        EntryKind::Dir => dir_preview_cancellable(path, cancelled),
        EntryKind::File => Some(file_preview(path)),
        EntryKind::Other => Some(Preview::Empty),
    }
}

fn compare_preview_entries(a: &(bool, String, String), b: &(bool, String, String)) -> Ordering {
    b.0.cmp(&a.0)
        .then_with(|| a.2.cmp(&b.2))
        .then_with(|| a.1.cmp(&b.1))
}

fn entry_is_dir(entry: &std::fs::DirEntry) -> io::Result<bool> {
    let file_type = entry.file_type()?;
    if !file_type.is_symlink() {
        return Ok(file_type.is_dir());
    }

    match entry.path().metadata() {
        Ok(metadata) => Ok(metadata.is_dir()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

fn dir_preview(path: &Path) -> Preview {
    let cancelled = AtomicBool::new(false);
    dir_preview_cancellable(path, &cancelled).unwrap_or(Preview::Empty)
}

fn dir_preview_cancellable(path: &Path, cancelled: &AtomicBool) -> Option<Preview> {
    let entries = match std::fs::read_dir(path) {
        Ok(entries) => entries,
        Err(error) => return Some(Preview::Error(error.to_string())),
    };
    let mut names = Vec::with_capacity(PREVIEW_NAMES);
    let mut total = 0usize;

    for entry in entries {
        if cancelled.load(AtomicOrdering::Relaxed) {
            return None;
        }
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => return Some(Preview::Error(error.to_string())),
        };
        let is_dir = match entry_is_dir(&entry) {
            Ok(is_dir) => is_dir,
            Err(error) => return Some(Preview::Error(error.to_string())),
        };
        let name = entry.file_name().to_string_lossy().into_owned();
        let candidate = (is_dir, name.clone(), name.to_lowercase());
        total = total.saturating_add(1);

        let index = names
            .binary_search_by(|current| compare_preview_entries(current, &candidate))
            .unwrap_or_else(|index| index);
        if index < PREVIEW_NAMES {
            names.insert(index, candidate);
            names.truncate(PREVIEW_NAMES);
        }
    }

    let names = names
        .into_iter()
        .map(|entry| {
            if entry.0 {
                format!("{}/", entry.1)
            } else {
                entry.1
            }
        })
        .collect();

    Some(Preview::Dir { names, total })
}

fn file_preview(path: &Path) -> Preview {
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(e) => return Preview::Error(e.to_string()),
    };
    let mut buf = Vec::new();
    // Read one extra byte so an exactly-sized file is not marked truncated.
    if let Err(e) = file.take(PREVIEW_BYTES + 1).read_to_end(&mut buf) {
        return Preview::Error(e.to_string());
    }
    if buf.contains(&0) {
        return Preview::Binary;
    }
    let truncated = buf.len() as u64 > PREVIEW_BYTES;
    buf.truncate(PREVIEW_BYTES as usize);
    let text = String::from_utf8_lossy(&buf);
    let mut lines: Vec<String> = text
        .lines()
        .take(PREVIEW_LINES + 1)
        .map(|l| sanitize(l).into_owned())
        .collect();
    let truncated = truncated || lines.len() > PREVIEW_LINES;
    lines.truncate(PREVIEW_LINES);
    Preview::Text { lines, truncated }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(name: &str) -> io::Result<Self> {
            let path =
                std::env::temp_dir().join(format!("dirview-preview-{name}-{}", std::process::id()));
            if path.exists() {
                std::fs::remove_dir_all(&path)?;
            }
            std::fs::create_dir_all(&path)?;
            Ok(Self(path))
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _cleanup = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn directory_preview_keeps_the_sorted_sample_and_exact_total() -> io::Result<()> {
        let root = TempDir::new("sorted")?;
        for index in (0..PREVIEW_NAMES + 50).rev() {
            std::fs::write(root.0.join(format!("entry-{index:03}")), "")?;
        }

        match dir_preview(&root.0) {
            Preview::Dir { names, total } => {
                assert_eq!(names.len(), PREVIEW_NAMES);
                assert_eq!(total, PREVIEW_NAMES + 50);
                assert_eq!(names.first().map(String::as_str), Some("entry-000"));
                assert_eq!(names.last().map(String::as_str), Some("entry-199"));
            }
            other => panic!("expected directory preview, got {other:?}"),
        }
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn directory_preview_follows_directory_symlinks() -> io::Result<()> {
        use std::os::unix::fs::symlink;

        let root = TempDir::new("symlinks")?;
        std::fs::create_dir(root.0.join("target"))?;
        std::fs::write(root.0.join("target/inner.txt"), "")?;
        symlink(root.0.join("target"), root.0.join("linked"))?;
        symlink(root.0.join("missing"), root.0.join("broken"))?;

        match dir_preview(&root.0) {
            Preview::Dir { names, total } => {
                assert_eq!(total, 3);
                assert!(names.contains(&"linked/".to_owned()));
                assert!(names.contains(&"broken".to_owned()));
            }
            other => panic!("expected directory preview, got {other:?}"),
        }
        Ok(())
    }
}
