use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::{App, Focus, Mode, VimMode};

pub fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;

    // Calculate stats
    let (word_count, reading_time) = if let Some(note) = app.current_note() {
        let words: usize = note.content.split_whitespace().count();
        let minutes = (words as f64 / 200.0).ceil() as usize;
        (words, minutes)
    } else {
        (0, 0)
    };

    // Calculate percentage
    let percentage = if app.content_items.is_empty() {
        0
    } else {
        ((app.content_cursor + 1) * 100) / app.content_items.len()
    };

    let note_path = app
        .current_note()
        .and_then(|n| n.file_path.as_ref())
        .map(|p| {
            let path_str = p.to_string_lossy().to_string();
            if let Some(home) = dirs::home_dir() {
                let home_str = home.to_string_lossy().to_string();
                if path_str.starts_with(&home_str) {
                    return path_str.replacen(&home_str, "~", 1);
                }
            }
            path_str
        })
        .unwrap_or_else(|| "—".to_string());

    // Get mode indicator
    let mode_text = match app.mode {
        Mode::Normal => match app.focus {
            Focus::Sidebar => "sidebar",
            Focus::Content => "content",
            Focus::Outline => "outline",
        },
        Mode::Edit => match app.vim_mode {
            VimMode::Normal => "normal",
            VimMode::Insert => "insert",
            VimMode::Visual => "visual",
        },
    };

    let statusbar = &theme.statusbar;

    let brand = Span::styled(
        " ekphos ",
        Style::default()
            .fg(statusbar.brand)
            .add_modifier(Modifier::BOLD),
    );

    let separator1 = Span::styled(
        "›",
        Style::default().fg(statusbar.separator),
    );

    let mode = Span::styled(
        format!(" {} ", mode_text),
        Style::default().fg(statusbar.mode),
    );

    let separator2 = Span::styled(
        "›",
        Style::default().fg(statusbar.separator),
    );

    let file_path = Span::styled(
        format!(" {}", note_path),
        Style::default().fg(statusbar.foreground),
    );

    // Right side content
    let zen_indicator = if app.zen_mode {
        vec![
            Span::styled(
                "zen  ",
                Style::default().fg(theme.info).add_modifier(Modifier::BOLD),
            ),
        ]
    } else {
        vec![]
    };

    let stats = Span::styled(
        format!("{}w · {}m", word_count, reading_time),
        Style::default().fg(statusbar.mode),
    );

    let position = Span::styled(
        format!("  {}%", percentage),
        Style::default().fg(statusbar.mode),
    );

    let help = Span::styled(
        "  ? for help",
        Style::default().fg(statusbar.mode),
    );

    // Build layout
    let left_content = vec![brand, separator1, mode, separator2, file_path];
    let mut right_content = zen_indicator;
    right_content.extend(vec![stats, position, help]);

    let left_width: usize = left_content.iter().map(|s| s.content.len()).sum();
    let right_width: usize = right_content.iter().map(|s| s.content.len()).sum();
    let available_width = area.width as usize;
    let padding = available_width.saturating_sub(left_width + right_width + 1);

    let mut spans = left_content;
    spans.push(Span::styled(" ".repeat(padding), Style::default().bg(statusbar.background)));
    spans.extend(right_content);
    spans.push(Span::styled(" ", Style::default().bg(statusbar.background)));

    let status_line = Line::from(spans);
    let status_bar = Paragraph::new(status_line)
        .style(Style::default().bg(statusbar.background));

    f.render_widget(status_bar, area);
}
