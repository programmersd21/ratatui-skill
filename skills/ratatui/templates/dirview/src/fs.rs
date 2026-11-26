//! Reading the file system. No terminal types in here.

use std::io::{self, Read};
use std::path::Path;
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
    match kind {
        EntryKind::Dir => dir_preview(path),
        EntryKind::File => file_preview(path),
        EntryKind::Other => Preview::Empty,
    }
}

fn dir_preview(path: &Path) -> Preview {
    let entries = match std::fs::read_dir(path) {
        Ok(entries) => entries,
        Err(error) => return Preview::Error(error.to_string()),
    };
    let mut names = Vec::with_capacity(PREVIEW_NAMES);
    let mut truncated = false;
    for entry in entries.take(PREVIEW_NAMES + 1) {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => return Preview::Error(error.to_string()),
        };
        if names.len() == PREVIEW_NAMES {
            truncated = true;
            break;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        let is_dir = entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false);
        names.push((is_dir, name));
    }
    names.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.to_lowercase().cmp(&b.1.to_lowercase())));
    let total = names.len() + usize::from(truncated);
    let names = names
        .into_iter()
        .map(|(is_dir, name)| if is_dir { format!("{name}/") } else { name })
        .collect();
    Preview::Dir { names, total, truncated }
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

    #[test]
    fn directory_preview_is_bounded_and_marks_truncation() -> io::Result<()> {
        let root = std::env::temp_dir().join(format!("dirview-preview-{}", std::process::id()));
        let _cleanup = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root)?;
        for index in 0..(PREVIEW_NAMES + 50) {
            std::fs::write(root.join(format!("entry-{index:03}")), "")?;
        }
        let preview = dir_preview(&root);
        assert!(matches!(preview, Preview::Dir { names, total, truncated: true } if names.len() == PREVIEW_NAMES && total == PREVIEW_NAMES + 1));
        std::fs::remove_dir_all(root)?;
        Ok(())
    }
}
