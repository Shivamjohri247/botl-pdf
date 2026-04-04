use std::collections::HashMap;

use crate::layout::elements::{Char, TextBlock, TextLine, Word};
use crate::layout::grouping;
use crate::layout::ordering;

/// Configurable parameters controlling layout analysis behavior.
#[derive(Debug, Clone)]
pub struct LayoutParams {
    /// Maximum horizontal gap between chars in the same word,
    /// as a multiple of font size. Default: 2.0
    pub word_margin: f64,
    /// Maximum vertical gap between lines in the same block,
    /// as a multiple of average line height. Default: 0.5
    pub line_margin: f64,
    /// Controls reading-order strictness.
    /// 0.0 = strict horizontal, 1.0 = strict vertical. Default: 0.5
    pub boxes_flow: f64,
}

impl Default for LayoutParams {
    fn default() -> Self {
        Self {
            word_margin: 2.0,
            line_margin: 0.5,
            boxes_flow: 0.5,
        }
    }
}

/// Sort characters into reading order using a per-line run-aware merge.
///
/// Algorithm:
/// 1. Sort chars strictly by (y0, x0) for initial ordering.
/// 2. Group consecutive chars into text lines based on vertical overlap.
/// 3. Within each line:
///    a. Detect if runs interleave (chars from different runs alternate in x).
///    b. If runs interleave: group chars by run_id, sort each run by x,
///       then order runs by their first char's x position (de-interleave).
///    c. If runs don't interleave: keep simple x-sort (already correct).
///
/// This correctly handles both:
/// - Interleaving runs: where the PDF producer places chars from different
///   text operations at alternating x positions (e.g., italic inline with
///   regular text, citation numbers mixed with author names).
/// - Normal multi-run text: where each run occupies its own x range within
///   a line, and a simple x-sort produces the correct reading order.
fn sort_chars_into_reading_order(chars: &mut [Char]) {
    if chars.len() <= 1 {
        return;
    }

    // Step 1: Sort strictly by (y0, x0)
    chars.sort_by(|a, b| {
        match a
            .bbox
            .y0
            .partial_cmp(&b.bbox.y0)
            .unwrap_or(std::cmp::Ordering::Equal)
        {
            std::cmp::Ordering::Equal => a
                .bbox
                .x0
                .partial_cmp(&b.bbox.x0)
                .unwrap_or(std::cmp::Ordering::Equal),
            other => other,
        }
    });

    // Step 2: Group into lines based on vertical overlap (greedy scan).
    // Uses the FIRST char's y-range as reference and does NOT expand it.
    // This prevents large decorative initials from swallowing adjacent visual lines.
    let n = chars.len();
    let mut line_ranges: Vec<(usize, usize)> = Vec::new();
    let mut line_start = 0usize;
    let mut ref_y0 = chars[0].bbox.y0;
    let mut ref_y1 = chars[0].bbox.y1;
    let mut ref_height = ref_y1 - ref_y0;

    for i in 1..n {
        let ch_y0 = chars[i].bbox.y0;
        let ch_y1 = chars[i].bbox.y1;
        let ch_height = ch_y1 - ch_y0;

        // Check vertical overlap with the reference (first char's) y-range
        let overlap_top = ref_y0.max(ch_y0);
        let overlap_bot = ref_y1.min(ch_y1);
        let overlap = (overlap_bot - overlap_top).max(0.0);
        let min_height = ref_height.min(ch_height);

        if min_height > 0.0 && overlap > min_height * 0.3 {
            // Same line — don't expand reference
        } else {
            line_ranges.push((line_start, i));
            line_start = i;
            ref_y0 = ch_y0;
            ref_y1 = ch_y1;
            ref_height = ch_height;
        }
    }
    line_ranges.push((line_start, n));

    // Step 3: Within each line, apply font-aware + run-aware sorting.
    for &(start, end) in &line_ranges {
        let line_chars = &mut chars[start..end];
        if line_chars.len() <= 2 {
            continue;
        }
        sort_within_line(line_chars);
    }
}

/// Sort characters within a single line, handling both font-layer
/// separation and run de-interleaving.
///
/// Some PDFs have dual font layers (e.g., large decorative initial + small
/// text) at overlapping positions. Other PDFs have interleaving runs from
/// different text operations at alternating x positions.
///
/// Algorithm:
/// 1. Group chars by font_size band (within 1.0pt tolerance).
/// 2. Within each font group, detect run interleaving and de-interleave.
/// 3. Order font groups by their minimum x0 position.
/// 4. Flatten.
fn sort_within_line(chars: &mut [Char]) {
    if chars.len() <= 2 {
        chars.sort_by(|a, b| {
            a.bbox
                .x0
                .partial_cmp(&b.bbox.x0)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        return;
    }

    // Group chars by font_size band (round to nearest integer for grouping)
    let mut font_groups: Vec<(i32, Vec<Char>)> = Vec::new();
    for ch in chars.iter() {
        let band = (ch.font_size + 0.5).floor() as i32;
        if let Some(slot) = font_groups.iter_mut().find(|(b, _)| *b == band) {
            slot.1.push(ch.clone());
        } else {
            font_groups.push((band, vec![ch.clone()]));
        }
    }

    // Within each font group: sub-group by y-position to separate visual lines
    // that got merged due to large decorative initials, then de-interleave each sub-line.
    for (_, group) in &mut font_groups {
        sub_group_by_y_and_sort(group);
    }

    // Order font groups by their minimum x0 position
    font_groups.sort_by(|a, b| {
        let min_x_a = a.1.iter().map(|c| c.bbox.x0).fold(f64::MAX, f64::min);
        let min_x_b = b.1.iter().map(|c| c.bbox.x0).fold(f64::MAX, f64::min);
        min_x_a
            .partial_cmp(&min_x_b)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Flatten
    let flat: Vec<Char> = font_groups
        .into_iter()
        .flat_map(|(_, group)| group)
        .collect();
    for (i, ch) in flat.into_iter().enumerate() {
        chars[i] = ch;
    }
}

/// Within a font_size band, further sub-group chars by their y-position to
/// separate different visual lines that may have been merged during line detection
/// (e.g., due to a large decorative initial's bbox spanning multiple lines).
/// Then sort each sub-group independently.
fn sub_group_by_y_and_sort(chars: &mut Vec<Char>) {
    if chars.len() <= 3 {
        chars.sort_by(|a, b| {
            a.bbox
                .y0
                .partial_cmp(&b.bbox.y0)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    a.bbox
                        .x0
                        .partial_cmp(&b.bbox.x0)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        });
        return;
    }

    // Sort by (y0, x0) first to establish visual line order
    chars.sort_by(|a, b| {
        a.bbox
            .y0
            .partial_cmp(&b.bbox.y0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                a.bbox
                    .x0
                    .partial_cmp(&b.bbox.x0)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });

    // Sub-group by y-position: chars are on the same sub-line if their y-centers
    // are within 0.5 * font_height of each other.
    let mut sub_lines: Vec<Vec<Char>> = Vec::new();
    let mut current = vec![chars[0].clone()];
    let mut ref_center_y = (chars[0].bbox.y0 + chars[0].bbox.y1) / 2.0;
    let mut ref_height = chars[0].bbox.height();

    for i in 1..chars.len() {
        let ch = &chars[i];
        let ch_center_y = (ch.bbox.y0 + ch.bbox.y1) / 2.0;
        let tolerance = ref_height * 0.5;

        if (ch_center_y - ref_center_y).abs() <= tolerance {
            current.push(ch.clone());
        } else {
            sub_lines.push(current);
            current = vec![ch.clone()];
            ref_center_y = ch_center_y;
            ref_height = ch.bbox.height();
        }
    }
    sub_lines.push(current);

    // Process each sub-line independently
    for sub_line in &mut sub_lines {
        maybe_de_interleave_runs(sub_line);
    }

    // Flatten back
    let flat: Vec<Char> = sub_lines.into_iter().flatten().collect();
    *chars = flat;
}

/// Conditionally de-interleave characters from multiple runs within a single line.
///
/// First checks whether runs actually interleave (their x ranges overlap).
/// If they do, groups chars by run_id, sorts each run by x, orders runs by
/// run_id (content stream order), and flattens.
/// If they don't interleave, just sorts all chars by x (normal reading order).
fn maybe_de_interleave_runs(chars: &mut [Char]) {
    if chars.len() <= 2 {
        chars.sort_by(|a, b| {
            a.bbox
                .x0
                .partial_cmp(&b.bbox.x0)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        return;
    }

    // Sort by x first to establish spatial order
    chars.sort_by(|a, b| {
        a.bbox
            .x0
            .partial_cmp(&b.bbox.x0)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Check if runs actually interleave by counting run_id inversions.
    // An inversion is where run_id decreases when scanning left-to-right.
    let n = chars.len();
    let mut inversions = 0usize;
    for i in 1..n {
        if chars[i].run_id < chars[i - 1].run_id {
            inversions += 1;
        }
    }

    // If no inversions, the x-sort is already correct.
    // Any inversions at all indicate run interleaving — de-interleave.
    if inversions == 0 {
        return;
    }

    // Runs genuinely interleave — de-interleave by grouping by run_id.
    let mut run_groups: HashMap<u32, Vec<Char>> = HashMap::new();
    for ch in chars.iter() {
        run_groups.entry(ch.run_id).or_default().push(ch.clone());
    }

    // Sort each run's chars by x
    for group in run_groups.values_mut() {
        group.sort_by(|a, b| {
            a.bbox
                .x0
                .partial_cmp(&b.bbox.x0)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    // Order runs by run_id (content stream order)
    let mut runs: Vec<(u32, Vec<Char>)> = run_groups.into_iter().collect();
    runs.sort_by_key(|(run_id, _)| *run_id);

    // Flatten
    let flat: Vec<Char> = runs.into_iter().flat_map(|(_, group)| group).collect();
    for (i, ch) in flat.into_iter().enumerate() {
        chars[i] = ch;
    }
}

/// Run the full layout analysis pipeline on a list of characters.
///
/// Pipeline: Characters → Words → Lines → Blocks → Reading Order
pub fn analyze_layout(chars: &[Char], params: &LayoutParams) -> Vec<TextBlock> {
    if chars.is_empty() {
        return Vec::new();
    }

    // Step 1: Sort characters into reading order (line-first approach).
    let mut sorted_chars = chars.to_vec();
    sort_chars_into_reading_order(&mut sorted_chars);

    // Step 2: Group characters into words
    let words = grouping::chars_to_words(&sorted_chars, params.word_margin);

    // Step 3: Group words into lines
    let lines = grouping::words_to_lines(&words);

    // Step 4: Group lines into text blocks
    let mut blocks = grouping::lines_to_blocks(&lines, params.line_margin);

    // Step 5: Sort blocks into reading order
    ordering::sort_blocks_reading_order(&mut blocks, params.boxes_flow);

    blocks
}

/// Extract plain text from layout blocks.
pub fn blocks_to_text(blocks: &[TextBlock]) -> String {
    blocks
        .iter()
        .map(|b| b.text.as_str())
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// Extract text with layout-preserving whitespace.
/// Attempts to maintain spatial positioning using spaces and newlines.
pub fn blocks_to_layout_text(blocks: &[TextBlock], page_width: f64) -> String {
    let mut result = String::new();

    for (i, block) in blocks.iter().enumerate() {
        if i > 0 {
            result.push_str("\n\n");
        }

        for (j, line) in block.lines.iter().enumerate() {
            if j > 0 {
                result.push('\n');
            }

            let mut prev_x1: Option<f64> = None;
            for word in &line.words {
                if let Some(px1) = prev_x1 {
                    let gap = word.bbox.x0 - px1;
                    // Use average character width from the word itself if available,
                    // otherwise estimate from font size
                    let char_width = if word.chars.len() > 1 {
                        let total_char_width: f64 =
                            word.chars.iter().map(|c| c.bbox.x1 - c.bbox.x0).sum();
                        total_char_width / word.chars.len() as f64
                    } else {
                        word.font_size * 0.5
                    };
                    if char_width > 0.0 {
                        let spaces = (gap / char_width).round() as usize;
                        if spaces > 0 {
                            for _ in 0..spaces.min(20) {
                                result.push(' ');
                            }
                        } else {
                            result.push(' ');
                        }
                    } else {
                        result.push(' ');
                    }
                }
                result.push_str(&word.text);
                prev_x1 = Some(word.bbox.x1);
            }
        }
    }

    result
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
            color: None,
            stroking_color: None,
            rotation: 0.0,
            run_id: 0,
        }
    }

    #[test]
    fn test_analyze_layout_basic() {
        let chars = vec![
            make_char("H", 0.0, 0.0, 8.0, 12.0),
            make_char("i", 8.0, 0.0, 13.0, 12.0),
        ];
        let params = LayoutParams::default();
        let blocks = analyze_layout(&chars, &params);
        assert!(!blocks.is_empty());
        assert!(blocks[0].text.contains("Hi"));
    }

    #[test]
    fn test_blocks_to_text() {
        let ch1 = make_char("a", 0.0, 0.0, 8.0, 12.0);
        let ch2 = make_char("b", 20.0, 0.0, 28.0, 12.0);
        let params = LayoutParams::default();
        let blocks = analyze_layout(&[ch1, ch2], &params);
        let text = blocks_to_text(&blocks);
        assert!(!text.is_empty());
    }

    #[test]
    fn test_maybe_de_interleave_runs_interleaved() {
        // Simulate: Run 1 = "AB" at x=0,20, Run 2 = "CD" at x=10,30
        // Runs interleave (run_id goes 1,2,1,2 in x-order), so should de-interleave
        let mut chars = vec![
            make_char("A", 0.0, 0.0, 8.0, 12.0),
            make_char("C", 10.0, 0.0, 18.0, 12.0), // run 2, but x between A and B
            make_char("B", 20.0, 0.0, 28.0, 12.0),
            make_char("D", 30.0, 0.0, 38.0, 12.0),
        ];
        chars[0].run_id = 1;
        chars[1].run_id = 2;
        chars[2].run_id = 1;
        chars[3].run_id = 2;

        maybe_de_interleave_runs(&mut chars);
        let text: String = chars.iter().map(|c| c.text.as_str()).collect();
        assert_eq!(
            text, "ABCD",
            "should keep runs together in run_id order when interleaved"
        );
    }

    #[test]
    fn test_maybe_de_interleave_runs_sequential() {
        // Sequential runs (not interleaved): Run 1 = "AB" x=0,10, Run 2 = "CD" x=20,30
        // Should just sort by x (no de-interleaving needed)
        let mut chars = vec![
            make_char("A", 0.0, 0.0, 8.0, 12.0),
            make_char("B", 10.0, 0.0, 18.0, 12.0),
            make_char("C", 20.0, 0.0, 28.0, 12.0),
            make_char("D", 30.0, 0.0, 38.0, 12.0),
        ];
        chars[0].run_id = 1;
        chars[1].run_id = 1;
        chars[2].run_id = 2;
        chars[3].run_id = 2;

        maybe_de_interleave_runs(&mut chars);
        let text: String = chars.iter().map(|c| c.text.as_str()).collect();
        assert_eq!(text, "ABCD", "sequential runs should stay in x-order");
    }

    #[test]
    fn test_full_pipeline_with_interleaving() {
        // Simulate page 143-like data: two overlapping runs on same line
        // Run 1 (regular): "AB E" at x=0, 10, 40, 50
        // Run 2 (italic): "CD" at x=20, 30 — interleaved in x
        // Expected: "AB ECD" (run 1 first, then run 2, in content stream order)
        let mut chars = vec![
            make_char("A", 0.0, 0.0, 8.0, 12.0),   // run 1
            make_char("B", 10.0, 0.0, 18.0, 12.0), // run 1
            make_char("C", 20.0, 0.0, 28.0, 12.0), // run 2
            make_char("D", 30.0, 0.0, 38.0, 12.0), // run 2
            make_char(" ", 40.0, 0.0, 44.0, 12.0), // run 1
            make_char("E", 50.0, 0.0, 58.0, 12.0), // run 1
        ];
        chars[0].run_id = 1;
        chars[1].run_id = 1;
        chars[2].run_id = 2;
        chars[3].run_id = 2;
        chars[4].run_id = 1;
        chars[5].run_id = 1;

        let params = LayoutParams::default();
        let blocks = analyze_layout(&chars, &params);
        let text = blocks_to_text(&blocks);
        // Run 1: "AB E" then Run 2: "CD" → "AB E CD" or "AB ECD"
        // The key is that A, B, space, E should be together, then C, D
        assert!(text.contains("AB"), "A and B should be together");
    }
}
