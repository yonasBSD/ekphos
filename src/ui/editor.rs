use ratatui::{
    layout::Rect,
    style::Style,
    widgets::Paragraph,
    Frame,
};

use crate::app::App;

pub fn render_editor(f: &mut Frame, app: &mut App, area: Rect) {
    let inner_width = area.width.saturating_sub(2) as usize;
    let inner_height = area.height.saturating_sub(2) as usize;

    app.update_editor_scroll(inner_height);

    f.render_widget(&app.textarea, area);

    let theme = &app.theme;
    let (cursor_row, cursor_col) = app.textarea.cursor();
    let lines = app.textarea.lines();
    let scroll_top = app.editor_scroll_top;

    /* Check if the cursor exists within a line that contains overflow.
    If it does, render the overflow indicators.

    NOTE: A sticky overflow indicator that remains visible during scrolling
    would be ideal, but this would likely require a custom textarea implementation. */
    if cursor_row >= scroll_top && cursor_row < scroll_top + inner_height {
        if let Some(line) = lines.get(cursor_row) {
            let line_len = line.chars().count();
            let y = area.y + 1 + (cursor_row - scroll_top) as u16;

            let h_scroll = if cursor_col > inner_width.saturating_sub(1) {
                cursor_col.saturating_sub(inner_width.saturating_sub(1))
            } else {
                0
            };

            let has_left_overflow = h_scroll > 0;
            let has_right_overflow = line_len > h_scroll + inner_width;

            if has_left_overflow {
                let indicator = Paragraph::new("«│")
                    .style(Style::default().fg(theme.yellow));
                f.render_widget(indicator, Rect::new(area.x + 1, y, 2, 1));
            }

            if has_right_overflow {
                let indicator = Paragraph::new("│»")
                    .style(Style::default().fg(theme.yellow));
                f.render_widget(indicator, Rect::new(area.x + area.width - 3, y, 2, 1));
            }
        }
    }
}
