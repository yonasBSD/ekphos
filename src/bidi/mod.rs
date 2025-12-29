//! Bidirectional text support using Unicode BiDi Algorithm (UAX #9)
//!
//! This module provides text direction detection and reordering for proper
//! display of RTL (Right-to-Left) and mixed-direction text.

use unicode_bidi::{bidi_class, BidiClass, BidiInfo, Level};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextDirection {
    #[default]
    Ltr,
    Rtl,
}

impl TextDirection {
    pub fn to_level(self) -> Level {
        match self {
            TextDirection::Ltr => Level::ltr(),
            TextDirection::Rtl => Level::rtl(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BidiLine {
    pub logical: String,
    pub visual: String,
    pub direction: TextDirection,
    pub logical_to_visual: Vec<usize>,
    pub visual_to_logical: Vec<usize>,
    pub has_rtl: bool,
}

impl BidiLine {
    fn new_ltr(text: String) -> Self {
        let len = text.chars().count();
        let indices: Vec<usize> = (0..len).collect();
        Self {
            logical: text.clone(),
            visual: text,
            direction: TextDirection::Ltr,
            logical_to_visual: indices.clone(),
            visual_to_logical: indices,
            has_rtl: false,
        }
    }

    pub fn visual_to_logical_col(&self, visual_col: usize) -> usize {
        if visual_col >= self.visual_to_logical.len() {
            self.logical.chars().count()
        } else {
            self.visual_to_logical[visual_col]
        }
    }

    pub fn logical_to_visual_col(&self, logical_col: usize) -> usize {
        if logical_col >= self.logical_to_visual.len() {
            self.visual.chars().count()
        } else {
            self.logical_to_visual[logical_col]
        }
    }
}

pub fn detect_direction(text: &str) -> TextDirection {
    for ch in text.chars() {
        match bidi_class(ch) {
            BidiClass::L => return TextDirection::Ltr,
            BidiClass::R | BidiClass::AL => return TextDirection::Rtl,
            _ => continue,
        }
    }
    TextDirection::Ltr
}

/// Check if text contains any RTL characters
pub fn has_rtl_chars(text: &str) -> bool {
    text.chars().any(|ch| {
        matches!(bidi_class(ch), BidiClass::R | BidiClass::AL | BidiClass::RLE | BidiClass::RLO | BidiClass::RLI)
    })
}

fn is_ascii_only(text: &str) -> bool {
    text.bytes().all(|b| b < 128)
}

/// Process a line of text for bidirectional display
///
/// This applies the Unicode Bidirectional Algorithm (UAX #9) to reorder
/// the text for visual display and creates position mappings.
pub fn process_line(text: &str) -> BidiLine {
    if text.is_empty() {
        return BidiLine::new_ltr(String::new());
    }

    if is_ascii_only(text) {
        return BidiLine::new_ltr(text.to_string());
    }

    if !has_rtl_chars(text) {
        return BidiLine::new_ltr(text.to_string());
    }
    process_line_bidi(text)
}

/// Full bidi processing for text with RTL characters
fn process_line_bidi(text: &str) -> BidiLine {
    let direction = detect_direction(text);
    let bidi_info = BidiInfo::new(text, Some(direction.to_level()));

    let para = &bidi_info.paragraphs[0];
    let line_range = para.range.clone();
    let levels = bidi_info.reordered_levels_per_char(para, line_range);

    let visual_to_logical = BidiInfo::reorder_visual(&levels);

    let chars: Vec<char> = text.chars().collect();
    let char_count = chars.len();

    let visual: String = visual_to_logical.iter().map(|&idx| chars[idx]).collect();

    let mut logical_to_visual: Vec<usize> = vec![0; char_count];
    for (visual_idx, &logical_idx) in visual_to_logical.iter().enumerate() {
        if logical_idx < char_count {
            logical_to_visual[logical_idx] = visual_idx;
        }
    }

    BidiLine {
        logical: text.to_string(),
        visual,
        direction,
        logical_to_visual,
        visual_to_logical,
        has_rtl: true,
    }
}

/// Process a line with a specific base direction override
pub fn process_line_with_direction(text: &str, direction: TextDirection) -> BidiLine {
    if text.is_empty() {
        return BidiLine {
            logical: String::new(),
            visual: String::new(),
            direction,
            logical_to_visual: vec![],
            visual_to_logical: vec![],
            has_rtl: false,
        };
    }

    if is_ascii_only(text) && direction == TextDirection::Ltr {
        return BidiLine::new_ltr(text.to_string());
    }

    let bidi_info = BidiInfo::new(text, Some(direction.to_level()));
    let para = &bidi_info.paragraphs[0];
    let line_range = para.range.clone();

    let levels = bidi_info.reordered_levels_per_char(para, line_range);

    let visual_to_logical = BidiInfo::reorder_visual(&levels);

    let chars: Vec<char> = text.chars().collect();
    let char_count = chars.len();

    let visual: String = visual_to_logical.iter().map(|&idx| chars[idx]).collect();

    let mut logical_to_visual: Vec<usize> = vec![0; char_count];
    for (visual_idx, &logical_idx) in visual_to_logical.iter().enumerate() {
        if logical_idx < char_count {
            logical_to_visual[logical_idx] = visual_idx;
        }
    }

    let has_rtl = has_rtl_chars(text);

    BidiLine {
        logical: text.to_string(),
        visual,
        direction,
        logical_to_visual,
        visual_to_logical,
        has_rtl,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_direction_ltr() {
        assert_eq!(detect_direction("Hello World"), TextDirection::Ltr);
        assert_eq!(detect_direction("123 Test"), TextDirection::Ltr);
    }

    #[test]
    fn test_detect_direction_rtl() {
        assert_eq!(detect_direction("مرحبا"), TextDirection::Rtl);
        assert_eq!(detect_direction("שלום"), TextDirection::Rtl);
    }

    #[test]
    fn test_detect_direction_mixed() {
        // First strong character determines direction
        assert_eq!(detect_direction("Hello مرحبا"), TextDirection::Ltr);
        assert_eq!(detect_direction("مرحبا Hello"), TextDirection::Rtl);
    }

    #[test]
    fn test_process_line_ascii() {
        let bidi = process_line("Hello World");
        assert_eq!(bidi.visual, "Hello World");
        assert!(!bidi.has_rtl);
        assert_eq!(bidi.direction, TextDirection::Ltr);
    }

    #[test]
    fn test_process_line_empty() {
        let bidi = process_line("");
        assert_eq!(bidi.visual, "");
        assert!(!bidi.has_rtl);
    }

    #[test]
    fn test_has_rtl_chars() {
        assert!(!has_rtl_chars("Hello World"));
        assert!(has_rtl_chars("مرحبا"));
        assert!(has_rtl_chars("Hello مرحبا World"));
    }

    #[test]
    fn test_position_mapping_ltr() {
        let bidi = process_line("abc");
        assert_eq!(bidi.logical_to_visual_col(0), 0);
        assert_eq!(bidi.logical_to_visual_col(1), 1);
        assert_eq!(bidi.logical_to_visual_col(2), 2);
        assert_eq!(bidi.visual_to_logical_col(0), 0);
        assert_eq!(bidi.visual_to_logical_col(1), 1);
        assert_eq!(bidi.visual_to_logical_col(2), 2);
    }

    #[test]
    fn test_position_mapping_out_of_bounds() {
        let bidi = process_line("abc");
        // Out of bounds should return length
        assert_eq!(bidi.logical_to_visual_col(10), 3);
        assert_eq!(bidi.visual_to_logical_col(10), 3);
    }
}
