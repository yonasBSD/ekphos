use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::{App, Focus, Mode};
use crate::vim::VimMode as VimModeNew;

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

    // Get mode indicator and command info for edit mode
    let (mode_text, pending_info, command_input) = match app.mode {
        Mode::Normal => {
            let mode = match app.focus {
                Focus::Sidebar => "sidebar",
                Focus::Content => "content",
                Focus::Outline => "outline",
            };
            (mode.to_string(), String::new(), None)
        }
        Mode::Edit => {
            // Get detailed vim mode info
            let vim = &app.vim;
            let mode_name = vim.mode.display_name().to_lowercase();

            // Build pending info string
            let mut pending_parts = Vec::new();

            // Recording indicator
            if vim.macros.is_recording() {
                pending_parts.push("recording".to_string());
            }

            // Count prefix
            if let Some(count) = vim.count {
                pending_parts.push(format!("{}", count));
            }

            // Operator pending
            if let VimModeNew::OperatorPending { operator, count } = &vim.mode {
                if let Some(c) = count {
                    pending_parts.push(format!("{}", c));
                }
                pending_parts.push(format!("{}", operator.char()));
            }

            // Pending g (for gg)
            if vim.pending_g {
                pending_parts.push("g".to_string());
            }

            // Pending z (for zz, zt, zb)
            if vim.pending_z {
                pending_parts.push("z".to_string());
            }

            // Pending find (f, F, t, T)
            if vim.pending_find.is_some() {
                pending_parts.push("f/t".to_string());
            }

            // Awaiting replace char
            if vim.awaiting_replace {
                pending_parts.push("r".to_string());
            }

            // Pending text object scope (i/a)
            if let Some(scope) = &vim.pending_text_object_scope {
                let ch = match scope {
                    crate::vim::TextObjectScope::Inner => 'i',
                    crate::vim::TextObjectScope::Around => 'a',
                };
                pending_parts.push(format!("{}", ch));
            }

            // Pending mark
            if let Some(mark) = &vim.pending_mark {
                let ch = match mark {
                    crate::vim::PendingMark::Set => 'm',
                    crate::vim::PendingMark::GotoExact => '`',
                    crate::vim::PendingMark::GotoLine => '\'',
                };
                pending_parts.push(format!("{}", ch));
            }

            // Pending macro
            if let Some(mac) = &vim.pending_macro {
                let ch = match mac {
                    crate::vim::PendingMacro::Record => 'q',
                    crate::vim::PendingMacro::Play => '@',
                };
                pending_parts.push(format!("{}", ch));
            }

            // Selected register
            if let Some(reg) = vim.registers.get_selected() {
                pending_parts.push(format!("\"{}", reg));
            }

            let pending = pending_parts.join("");

            // Command mode input
            let cmd_input = if matches!(vim.mode, VimModeNew::Command) {
                Some(format!(":{}", vim.command_buffer))
            } else {
                None
            };

            (mode_name, pending, cmd_input)
        }
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

    // Pending info (operators, count, etc.)
    let pending = if !pending_info.is_empty() {
        vec![
            Span::styled(
                "›",
                Style::default().fg(statusbar.separator),
            ),
            Span::styled(
                format!(" {} ", pending_info),
                Style::default().fg(theme.warning).add_modifier(Modifier::BOLD),
            ),
        ]
    } else {
        vec![]
    };

    let separator2 = Span::styled(
        "›",
        Style::default().fg(statusbar.separator),
    );

    // Command input or file path
    let path_or_command = if let Some(cmd) = command_input {
        Span::styled(
            format!(" {}", cmd),
            Style::default().fg(theme.primary).add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(
            format!(" {}", note_path),
            Style::default().fg(statusbar.foreground),
        )
    };

    // Right side content
    // Recording indicator
    let recording_indicator = if app.mode == Mode::Edit && app.vim.macros.is_recording() {
        vec![
            Span::styled(
                "● REC  ",
                Style::default().fg(theme.error).add_modifier(Modifier::BOLD),
            ),
        ]
    } else {
        vec![]
    };

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
    let mut left_content = vec![brand, separator1, mode];
    left_content.extend(pending);
    left_content.push(separator2);
    left_content.push(path_or_command);

    let mut right_content = recording_indicator;
    right_content.extend(zen_indicator);
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
