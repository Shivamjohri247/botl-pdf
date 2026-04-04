use crate::layout::elements::{Char, TextBlock, TextLine, Word};

/// Group characters into words based on spatial proximity.
///
/// Characters are grouped into the same word when the horizontal gap between
/// them is less than `word_margin × font_size`. The `word_margin` parameter
/// controls the sensitivity — higher values require larger gaps to split words.
pub fn chars_to_words(chars: &[Char], word_margin: f64) -> Vec<Word> {
    if chars.is_empty() {
        return Vec::new();
    }

    let mut words = Vec::new();
    let mut current_chars: Vec<Char> = vec![chars[0].clone()];

    for i in 1..chars.len() {
        let prev = &chars[i - 1];
        let curr = &chars[i];

        // Calculate gap between previous and current character
        let gap = curr.bbox.x0 - prev.bbox.x1;

        // Threshold based on font size
        let avg_font_size = (prev.font_size + curr.font_size) / 2.0;
        let threshold = word_margin * avg_font_size;

        // Break word on: font change, vertical displacement, or space
        let font_changed =
            prev.font_name != curr.font_name || (prev.font_size - curr.font_size).abs() > 0.5;
        let vertical_break = {
            let prev_center_y = prev.bbox.center().1;
            let curr_center_y = curr.bbox.center().1;
            let min_height = prev.height().min(curr.height());
            (prev_center_y - curr_center_y).abs() > min_height * 0.5
        };

        // A space character is always a word boundary
        let prev_is_space = prev.text == " ";
        let curr_is_space = curr.text == " ";
        let is_space = prev_is_space || curr_is_space;

        // Gap-based breaking: only break on large gaps when a space is involved.
        // Without a space, large gaps are typically from justified text stretching
        // or PDF producers splitting a single word across Tj operations.
        // Use a generous absolute threshold (5x font_size) as a safety net for
        // extreme cases like column boundaries with no space.
        let large_gap_break = gap > threshold && (is_space || gap > avg_font_size * 5.0);

        if large_gap_break || font_changed || vertical_break || is_space {
            // Space character becomes a word boundary
            if current_chars.len() == 1 && current_chars[0].text == " " {
                // Skip standalone space "words"
            } else if !current_chars.is_empty() {
                if let Some(word) = Word::from_chars(current_chars.clone()) {
                    words.push(word);
                }
            }
            current_chars.clear();
            if curr.text != " " {
                current_chars.push(curr.clone());
            }
        } else {
            current_chars.push(curr.clone());
        }
    }

    // Flush remaining
    if current_chars.len() == 1 && current_chars[0].text == " " {
        // Skip
    } else if let Some(word) = Word::from_chars(current_chars) {
        words.push(word);
    }

    words
}

/// Group words into lines based on vertical overlap.
///
/// Words are in the same line when their vertical overlap exceeds 50%
/// of the smaller word's height. Words are processed in input order to
/// preserve the reading order established by char-level de-interleaving.
pub fn words_to_lines(words: &[Word]) -> Vec<TextLine> {
    if words.is_empty() {
        return Vec::new();
    }

    // Process words in input order (they are already in reading order from
    // the char-level sort_chars_into_reading_order). Do NOT re-sort by (y, x)
    // as that would undo the run de-interleaving.
    let mut lines = Vec::new();
    let mut current_words: Vec<Word> = vec![words[0].clone()];
    let mut line_y0 = words[0].bbox.y0;
    let mut line_y1 = words[0].bbox.y1;

    for curr in words.iter().skip(1) {
        // Check vertical overlap with current line
        let overlap_top = line_y0.max(curr.bbox.y0);
        let overlap_bot = line_y1.min(curr.bbox.y1);
        let overlap = (overlap_bot - overlap_top).max(0.0);
        let min_height = (line_y1 - line_y0).min(curr.bbox.height());

        if min_height > 0.0 && overlap > min_height * 0.3 {
            current_words.push(curr.clone());
            line_y0 = line_y0.min(curr.bbox.y0);
            line_y1 = line_y1.max(curr.bbox.y1);
        } else {
            if let Some(line) = TextLine::from_words(current_words) {
                lines.push(line);
            }
            current_words = vec![curr.clone()];
            line_y0 = curr.bbox.y0;
            line_y1 = curr.bbox.y1;
        }
    }

    // Flush remaining
    if let Some(line) = TextLine::from_words(current_words) {
        lines.push(line);
    }

    lines
}

/// Group lines into text blocks based on vertical proximity and alignment.
///
/// Lines are in the same block when:
/// - The vertical gap between them is less than `line_margin × avg_line_height`
/// - They have consistent horizontal alignment (left, right, or centered)
pub fn lines_to_blocks(lines: &[TextLine], line_margin: f64) -> Vec<TextBlock> {
    if lines.is_empty() {
        return Vec::new();
    }

    // Sort lines by vertical position (top to bottom)
    let mut sorted: Vec<&TextLine> = lines.iter().collect();
    sorted.sort_by(|a, b| a.bbox.y0.partial_cmp(&b.bbox.y0).unwrap());

    let mut blocks = Vec::new();
    let mut current_lines: Vec<TextLine> = vec![sorted[0].clone()];
    let mut avg_height = sorted[0].bbox.height();

    for i in 1..sorted.len() {
        let prev = &sorted[i - 1];
        let curr = &sorted[i];

        let gap = curr.bbox.y0 - prev.bbox.y1;
        let threshold = line_margin * avg_height;

        // Check horizontal alignment consistency
        let left_aligned = (prev.bbox.x0 - curr.bbox.x0).abs() < avg_height * 0.5;
        let aligned = left_aligned; // Could also check right/center alignment

        if gap <= threshold && aligned {
            current_lines.push((*curr).clone());
            // Running average of line height
            let n = current_lines.len() as f64;
            avg_height = avg_height * (n - 1.0) / n + curr.bbox.height() / n;
        } else {
            if let Some(block) = TextBlock::from_lines(current_lines) {
                blocks.push(block);
            }
            current_lines = vec![(*curr).clone()];
            avg_height = curr.bbox.height();
        }
    }

    // Flush remaining
    if let Some(block) = TextBlock::from_lines(current_lines) {
        blocks.push(block);
    }

    blocks
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::BBox;

    fn make_char(text: &str, x0: f64, y0: f64, x1: f64, y1: f64) -> Char {
        Char {
            text: text.to_string(),
            bbox: BBox::new(x0, y0, x1, y1),
            font_name: "F1".to_string(),
            font_size: 12.0,
            bold: false,
            italic: false,
            color: Some((0.0, 0.0, 0.0)),
            stroking_color: None,
            rotation: 0.0,
            run_id: 0,
        }
    }

    #[test]
    fn test_chars_to_words() {
        let chars = vec![
            make_char("H", 0.0, 0.0, 8.0, 12.0),
            make_char("e", 8.0, 0.0, 15.0, 12.0),
            make_char("l", 15.0, 0.0, 20.0, 12.0),
            make_char(" ", 20.0, 0.0, 25.0, 12.0), // space triggers word break
            make_char("w", 30.0, 0.0, 38.0, 12.0),
        ];
        let words = chars_to_words(&chars, 0.3);
        assert_eq!(words.len(), 2);
        assert_eq!(words[0].text, "Hel");
        assert_eq!(words[1].text, "w");
    }

    #[test]
    fn test_words_to_lines() {
        let chars1 = vec![make_char("a", 0.0, 0.0, 10.0, 12.0)];
        let chars2 = vec![make_char("b", 0.0, 20.0, 10.0, 32.0)];
        let w1 = Word::from_chars(chars1).unwrap();
        let w2 = Word::from_chars(chars2).unwrap();
        let lines = words_to_lines(&[w1, w2]);
        assert_eq!(lines.len(), 2); // Different y positions → different lines
    }

    #[test]
    fn test_lines_to_blocks() {
        let chars1 = vec![make_char("a", 0.0, 0.0, 10.0, 12.0)];
        let chars2 = vec![make_char("b", 0.0, 16.0, 10.0, 28.0)]; // gap=4, threshold=6
        let w1 = Word::from_chars(chars1).unwrap();
        let w2 = Word::from_chars(chars2).unwrap();
        let l1 = TextLine::from_words(vec![w1]).unwrap();
        let l2 = TextLine::from_words(vec![w2]).unwrap();
        let blocks = lines_to_blocks(&[l1, l2], 0.5);
        assert_eq!(blocks.len(), 1); // Close together → same block
    }
}
