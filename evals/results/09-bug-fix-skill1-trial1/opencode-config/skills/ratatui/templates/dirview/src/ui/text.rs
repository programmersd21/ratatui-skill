//! Text helpers: width-aware truncation, sanitizing, human-readable units.
//!
//! Always measure with `unicode-width`, never `str::len`: a CJK character is
//! two columns wide and one combining mark is zero.

use std::borrow::Cow;
use std::time::Duration;

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Replace control characters so untrusted text (file names, file contents)
/// cannot move the cursor, change colors or break the layout.
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

/// Keep the end of `s`, starting with an ellipsis. Right for file paths.
pub fn truncate_start(s: &str, max: usize) -> String {
    if s.width() <= max {
        return s.to_owned();
    }
    if max == 0 {
        return String::new();
    }
    let mut tail = Vec::new();
    let mut used = 0;
    for c in s.chars().rev() {
        let w = c.width().unwrap_or(0);
        if used + w > max - 1 {
            break;
        }
        tail.push(c);
        used += w;
    }
    let mut out = String::from('\u{2026}');
    out.extend(tail.into_iter().rev());
    out
}

/// "512B", "1.5K", "12M". Binary units, one decimal below ten.
pub fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "K", "M", "G", "T"];
    if bytes < 1024 {
        return format!("{bytes}B");
    }
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if value < 10.0 {
        format!("{value:.1}{}", UNITS[unit])
    } else {
        format!("{value:.0}{}", UNITS[unit])
    }
}

/// "just now", "5 min ago", "3 h ago", "2 days ago", "4 months ago".
pub fn human_age(age: Duration) -> String {
    let secs = age.as_secs();
    match secs {
        0..=59 => "just now".to_owned(),
        60..=3599 => format!("{} min ago", secs / 60),
        3600..=86_399 => format!("{} h ago", secs / 3600),
        86_400..=5_183_999 => format!("{} days ago", secs / 86_400),
        5_184_000..=63_071_999 => format!("{} months ago", secs / 2_592_000),
        _ => format!("{} years ago", secs / 31_536_000),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_end_respects_display_width() {
        assert_eq!(truncate_end("hello", 5), "hello");
        assert_eq!(truncate_end("hello world", 6), "hello\u{2026}");
        // Each CJK character is two columns wide.
        assert_eq!(truncate_end("日本語テキスト", 7), "日本語\u{2026}");
        assert_eq!(truncate_end("abc", 0), "");
    }

    #[test]
    fn truncate_start_keeps_the_tail() {
        assert_eq!(truncate_start("/home/user/projects", 9), "\u{2026}projects");
        assert_eq!(truncate_start("/short", 20), "/short");
    }

    #[test]
    fn truncated_text_never_exceeds_the_limit() {
        for max in 0..12 {
            assert!(truncate_end("日本語 and ascii", max).width() <= max);
            assert!(truncate_start("日本語 and ascii", max).width() <= max);
        }
    }

    #[test]
    fn sanitize_removes_escapes_and_tabs() {
        assert_eq!(sanitize("plain"), "plain");
        assert_eq!(sanitize("a\u{1b}[31mb"), "a\u{b7}[31mb");
        assert_eq!(sanitize("a\tb"), "a b");
        assert_eq!(sanitize("line\r"), "line\u{b7}");
    }

    #[test]
    fn sizes_are_compact() {
        assert_eq!(human_size(0), "0B");
        assert_eq!(human_size(1023), "1023B");
        assert_eq!(human_size(1536), "1.5K");
        assert_eq!(human_size(12 * 1024 * 1024), "12M");
    }

    #[test]
    fn ages_pick_a_sensible_unit() {
        assert_eq!(human_age(Duration::from_secs(5)), "just now");
        assert_eq!(human_age(Duration::from_secs(300)), "5 min ago");
        assert_eq!(human_age(Duration::from_secs(7200)), "2 h ago");
        assert_eq!(human_age(Duration::from_secs(3 * 86_400)), "3 days ago");
    }
}
