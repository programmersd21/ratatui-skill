//! Text helpers: width-aware truncation and sanitizing.
//!
//! Always measure with `unicode-width`, never `str::len`: a CJK character is
//! two columns wide and a combining mark is zero.

use std::borrow::Cow;

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Replace control characters so untrusted text (task text from a hand-edited
/// file) cannot move the cursor, change colors or break the layout.
pub fn sanitize(s: &str) -> Cow<'_, str> {
    if !s.chars().any(char::is_control) {
        return Cow::Borrowed(s);
    }
    Cow::Owned(
        s.chars()
            .map(|c| match c {
                '\t' => ' ',
                c if c.is_control() => '\u{b7}',
                c => c,
            })
            .collect(),
    )
}

/// Display width in terminal columns.
pub fn width(s: &str) -> usize {
    s.width()
}

/// Keep the start of `s`, ending with an ellipsis when it does not fit.
pub fn truncate_end(s: &str, max: usize) -> String {
    if s.width() <= max {
        return s.to_owned();
    }
    if max == 0 {
        return String::new();
    }
    let mut out = String::new();
    let mut used = 0;
    for c in s.chars() {
        let w = c.width().unwrap_or(0);
        if used + w > max - 1 {
            break;
        }
        out.push(c);
        used += w;
    }
    out.push('\u{2026}');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_end_respects_display_width() {
        assert_eq!(truncate_end("hello", 5), "hello");
        assert_eq!(truncate_end("hello world", 6), "hello\u{2026}");
        assert_eq!(truncate_end("日本語テキスト", 7), "日本語\u{2026}");
        assert_eq!(truncate_end("abc", 0), "");
    }

    #[test]
    fn truncated_text_never_exceeds_the_limit() {
        for max in 0..12 {
            assert!(truncate_end("日本語 and ascii", max).width() <= max);
        }
    }

    #[test]
    fn sanitize_removes_escapes_and_tabs() {
        assert_eq!(sanitize("plain"), "plain");
        assert_eq!(sanitize("a\u{1b}[31mb"), "a\u{b7}[31mb");
        assert_eq!(sanitize("a\tb"), "a b");
        assert_eq!(sanitize("line\r"), "line\u{b7}");
    }
}
