use ratatui::style::{Color, Modifier, Style as RatatuiStyle};
use ratatui::text::Span;
use syntect::easy::HighlightLines;
use syntect::highlighting::{FontStyle, Style, ThemeSet};
use syntect::parsing::SyntaxSet;

pub struct Highlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    theme_name: String,
}

impl Highlighter {
    pub fn new(theme_name: &str) -> Self {
        let theme_set = ThemeSet::load_defaults();
        let valid_theme = if theme_set.themes.contains_key(theme_name) {
            theme_name.to_string()
        } else {
            "base16-ocean.dark".to_string()
        };
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set,
            theme_name: valid_theme,
        }
    }

    pub fn highlight_line(&self, line: &str, lang: &str) -> Vec<Span<'static>> {
        let syntax = self
            .syntax_set
            .find_syntax_by_token(lang)
            .or_else(|| self.syntax_set.find_syntax_by_extension(lang))
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

        let theme = &self.theme_set.themes[&self.theme_name];
        let mut highlighter = HighlightLines::new(syntax, theme);

        match highlighter.highlight_line(line, &self.syntax_set) {
            Ok(ranges) => ranges
                .into_iter()
                .map(|(style, text)| self.style_to_span(text, style))
                .collect(),
            Err(_) => vec![Span::raw(line.to_string())],
        }
    }

    fn style_to_span(&self, text: &str, style: Style) -> Span<'static> {
        let fg = Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b);

        let mut ratatui_style = RatatuiStyle::default().fg(fg);

        if style.font_style.contains(FontStyle::BOLD) {
            ratatui_style = ratatui_style.add_modifier(Modifier::BOLD);
        }
        if style.font_style.contains(FontStyle::ITALIC) {
            ratatui_style = ratatui_style.add_modifier(Modifier::ITALIC);
        }
        if style.font_style.contains(FontStyle::UNDERLINE) {
            ratatui_style = ratatui_style.add_modifier(Modifier::UNDERLINED);
        }

        Span::styled(text.to_string(), ratatui_style)
    }
}

impl Default for Highlighter {
    fn default() -> Self {
        Self::new("base16-ocean.dark")
    }
}
