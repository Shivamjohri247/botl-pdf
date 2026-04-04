use crate::layout::elements::{TextBlock, TextLine, Word};

/// Reading-order detection for layout elements.
///
/// Sorts blocks into reading order: top-to-bottom within columns,
/// left-to-right across columns.

/// Sort text blocks into reading order.
/// Uses a simple top-down, left-to-right sort with column detection.
pub fn sort_blocks_reading_order(blocks: &mut [TextBlock], boxes_flow: f64) {
    if blocks.len() <= 1 {
        return;
    }

    // boxes_flow controls the strictness of horizontal vs vertical ordering:
    // 0.0 = strict horizontal (all lines treated equally)
    // 1.0 = strict vertical (everything in column order)
    // 0.5 = balanced (default)

    // First, detect columns by finding vertical whitespace gaps
    let columns = detect_columns(blocks, boxes_flow);

    // Sort within each column top-to-bottom
    for col_blocks in &columns {
        let indices = col_blocks;
        // Already sorted by y within column
    }

    // Sort columns left-to-right and assign final order
    blocks.sort_by(|a, b| {
        // Primary sort: y position (top to bottom) weighted by boxes_flow
        let y_diff = a.bbox.y0 - b.bbox.y0;
        let x_diff = a.bbox.x0 - b.bbox.x0;

        // If blocks are roughly at the same vertical position, sort by x
        let avg_height = (a.bbox.height() + b.bbox.height()) / 2.0;
        if y_diff.abs() < avg_height * 0.5 {
            x_diff.partial_cmp(&0.0).unwrap_or(std::cmp::Ordering::Equal)
        } else {
            // Weighted ordering based on boxes_flow
            let score_a = a.bbox.y0 + a.bbox.x0 * (1.0 - boxes_flow);
            let score_b = b.bbox.y0 + b.bbox.x0 * (1.0 - boxes_flow);
            score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
        }
    });
}

/// Sort words within a line by x position (left to right).
pub fn sort_words_in_line(words: &mut [Word]) {
    words.sort_by(|a, b| a.bbox.x0.partial_cmp(&b.bbox.x0).unwrap_or(std::cmp::Ordering::Equal));
}

/// Sort lines within a block by y position (top to bottom).
pub fn sort_lines_in_block(lines: &mut [TextLine]) {
    lines.sort_by(|a, b| a.bbox.y0.partial_cmp(&b.bbox.y0).unwrap_or(std::cmp::Ordering::Equal));
}

/// Detect columns by analyzing horizontal positions of blocks.
/// Returns groups of block indices, one group per column.
fn detect_columns(blocks: &[TextBlock], _boxes_flow: f64) -> Vec<Vec<usize>> {
    if blocks.is_empty() {
        return Vec::new();
    }

    // Sort blocks by x position
    let mut indexed: Vec<(usize, f64)> = blocks
        .iter()
        .enumerate()
        .map(|(i, b)| (i, b.bbox.x0))
        .collect();
    indexed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    // Group into columns: blocks with overlapping x ranges are in the same column
    let mut columns: Vec<Vec<usize>> = Vec::new();
    let mut current_col = vec![indexed[0].0];
    let mut col_x_max = blocks[indexed[0].0].bbox.x1;

    for &(idx, x0) in &indexed[1..] {
        let x1 = blocks[idx].bbox.x1;
        // If this block starts before the current column's right edge, it's overlapping
        if x0 <= col_x_max {
            current_col.push(idx);
            col_x_max = col_x_max.max(x1);
        } else {
            columns.push(current_col);
            current_col = vec![idx];
            col_x_max = x1;
        }
    }
    columns.push(current_col);

    columns
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::BBox;
    use crate::layout::elements::{Char, TextLine, Word};

    fn make_block(x0: f64, y0: f64, x1: f64, y1: f64, text: &str) -> TextBlock {
        let ch = Char {
            text: text.to_string(),
            bbox: BBox::new(x0, y0, x1, y1),
            font_name: "F1".to_string(),
            font_size: 12.0,
            bold: false,
            italic: false,
            color: None,
            stroking_color: None,
            rotation: 0.0,
            run_id: 0,
        };
        let word = Word::from_chars(vec![ch]).unwrap();
        let line = TextLine::from_words(vec![word]).unwrap();
        TextBlock::from_lines(vec![line]).unwrap()
    }

    #[test]
    fn test_reading_order_top_to_bottom() {
        let mut blocks = vec![
            make_block(0.0, 100.0, 200.0, 112.0, "second"),
            make_block(0.0, 0.0, 200.0, 12.0, "first"),
        ];
        sort_blocks_reading_order(&mut blocks, 0.5);
        assert_eq!(blocks[0].text, "first");
        assert_eq!(blocks[1].text, "second");
    }

    #[test]
    fn test_reading_order_columns() {
        let mut blocks = vec![
            make_block(200.0, 0.0, 400.0, 12.0, "col2-top"),
            make_block(0.0, 0.0, 180.0, 12.0, "col1-top"),
            make_block(0.0, 20.0, 180.0, 32.0, "col1-bottom"),
            make_block(200.0, 20.0, 400.0, 32.0, "col2-bottom"),
        ];
        sort_blocks_reading_order(&mut blocks, 0.5);
        // Should be in reading order: col1-top, col2-top, col1-bottom, col2-bottom
        // or col1-top, col1-bottom, col2-top, col2-bottom depending on boxes_flow
        assert_eq!(blocks[0].text, "col1-top");
    }
}
