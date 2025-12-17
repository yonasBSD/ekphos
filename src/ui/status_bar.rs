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
        let minutes = (words as f64 / 200.0).ceil() as usize; // ~200 words per minute
        (words, minutes)
    } else {
        (0, 0)
    };

    // Calculate percentage complete based on cursor position
    let percentage = if app.content_items.is_empty() {
        0
    } else {
        ((app.content_cursor + 1) * 100) / app.content_items.len()
    };

    // Get current note file path
    let note_path = app
        .current_note()
        .and_then(|n| n.file_path.as_ref())
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "No file".to_string());

    // Get current mode indicator
    let mode_indicator = match app.mode {
        Mode::Normal => match app.focus {
            Focus::Sidebar => "SIDEBAR",
            Focus::Content => "CONTENT",
            Focus::Outline => "OUTLINE",
        },
        Mode::Edit => match app.vim_mode {
            VimMode::Normal => "EDIT: NORMAL",
            VimMode::Insert => "EDIT: INSERT",
            VimMode::Visual => "EDIT: VISUAL",
        },
    };

    // Build status bar content
    let logo = Span::styled(
        " ◆ Ekphos ",
        Style::default()
            .fg(theme.black)
            .bg(theme.bright_blue)
            .add_modifier(Modifier::BOLD),
    );

    let mode = Span::styled(
        format!(" {} ", mode_indicator),
        Style::default()
            .fg(theme.black)
            .bg(theme.yellow),
    );

    let file_path = Span::styled(
        format!(" {} ", note_path),
        Style::default().fg(theme.foreground),
    );

    let separator = Span::styled(
        " │ ",
        Style::default().fg(theme.bright_black),
    );

    let reading = Span::styled(
        format!("{} words ~{}min", word_count, reading_time),
        Style::default().fg(theme.green),
    );

    let progress = Span::styled(
        format!(" {}% ", percentage),
        Style::default()
            .fg(theme.black)
            .bg(theme.magenta),
    );

    let help_key = Span::styled(
        " ? for help ",
        Style::default().fg(theme.white).bg(theme.bright_black),
    );

    // Calculate spacing for justify-between layout
    let left_content = vec![logo, Span::raw(" "), mode, Span::raw(" "), file_path];
    let right_content = vec![reading, separator.clone(), progress, Span::raw(" "), help_key];

    let left_width: usize = left_content.iter().map(|s| s.content.len()).sum();
    let right_width: usize = right_content.iter().map(|s| s.content.len()).sum();
    let available_width = area.width as usize;
    let padding = available_width.saturating_sub(left_width + right_width);

    let mut spans = left_content;
    spans.push(Span::styled(" ".repeat(padding), Style::default().bg(theme.bright_black)));
    spans.extend(right_content);

    let status_line = Line::from(spans);
    let status_bar = Paragraph::new(status_line)
        .style(Style::default().bg(theme.bright_black));

    f.render_widget(status_bar, area);
}
