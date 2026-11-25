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
    let mut entries = match read_dir(path) {
        Ok(e) => e,
        Err(e) => return Preview::Error(e.to_string()),
    };
    let total = entries.len();
    entries.sort_by_cached_key(|e| (e.kind != EntryKind::Dir, e.name.to_lowercase()));
    let names = entries
        .into_iter()
        .take(PREVIEW_NAMES)
        .map(|e| match e.kind {
            EntryKind::Dir => format!("{}/", e.name),
            _ => e.name,
        })
        .collect();
    Preview::Dir { names, total }
}

fn file_preview(path: &Path) -> Preview {
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(e) => return Preview::Error(e.to_string()),
    };
    let mut buf = Vec::new();
    if let Err(e) = file.take(PREVIEW_BYTES).read_to_end(&mut buf) {
        return Preview::Error(e.to_string());
    }
    if buf.contains(&0) {
        return Preview::Binary;
    }
    let text = String::from_utf8_lossy(&buf);
    let mut lines: Vec<String> = text
        .lines()
        .take(PREVIEW_LINES + 1)
        .map(|l| sanitize(l).into_owned())
        .collect();
    let truncated = lines.len() > PREVIEW_LINES || buf.len() as u64 == PREVIEW_BYTES;
    lines.truncate(PREVIEW_LINES);
    Preview::Text { lines, truncated }
}
