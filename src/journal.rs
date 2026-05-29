//! Journal mode.
//!
//! Pressing `t` opens today's daily note (`journal.<date>.md`) in the notes
//! directory — creating it from a small dated template if it doesn't exist
//! yet, or just opening it if today's entry is already there. The date is the
//! user's *local* date (via `chrono`), so the filename rolls over at local
//! midnight rather than UTC.

use chrono::Local;

/// Filename for today's journal entry, e.g. `journal.2024-05-29.md`.
pub fn today_filename() -> String {
    format!("journal.{}.md", Local::now().format("%Y-%m-%d"))
}

/// Initial content for a fresh journal entry, with the date filled in.
pub fn new_entry_content() -> String {
    let now = Local::now();
    let date = now.format("%Y-%m-%d");
    let weekday = now.format("%A");
    format!(
        "---\ntitle: Journal — {date}\ntags: [journal]\ndate: {date}\n---\n\n# {date} · {weekday}\n\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filename_matches_journal_date_md() {
        let name = today_filename();
        assert!(name.starts_with("journal."));
        assert!(name.ends_with(".md"));
        // journal.YYYY-MM-DD.md
        let date = &name["journal.".len()..name.len() - ".md".len()];
        assert_eq!(date.len(), 10, "expected YYYY-MM-DD, got {date:?}");
        assert_eq!(date.as_bytes()[4], b'-');
        assert_eq!(date.as_bytes()[7], b'-');
    }

    #[test]
    fn entry_content_has_frontmatter_and_heading() {
        let content = new_entry_content();
        assert!(content.starts_with("---\n"));
        assert!(content.contains("tags: [journal]"));
        assert!(content.contains("\n# "));
    }
}
