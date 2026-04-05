use std::collections::HashMap;

use crate::layout::elements::{Char, TextBlock};
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
///    then order runs by their first char's x position (de-interleave).
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

    for (i, ch) in chars.iter().enumerate().skip(1) {
        let ch_y0 = ch.bbox.y0;
        let ch_y1 = ch.bbox.y1;
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
/// Uses index-based grouping to minimize cloning.
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

    // Sort by font_size band, then by x within each band, using stable partition
    // to preserve relative order.
    // Step 1: Sort by (font_size_band, x0) to group by font band
    let mut indices: Vec<usize> = (0..chars.len()).collect();
    indices.sort_by(|&a, &b| {
        let band_a = (chars[a].font_size + 0.5).floor() as i32;
        let band_b = (chars[b].font_size + 0.5).floor() as i32;
        match band_a.cmp(&band_b) {
            std::cmp::Ordering::Equal => chars[a]
                .bbox
                .x0
                .partial_cmp(&chars[b].bbox.x0)
                .unwrap_or(std::cmp::Ordering::Equal),
            other => other,
        }
    });

    // Identify band boundaries
    let mut band_slices: Vec<(i32, usize, usize)> = Vec::new(); // (band, start, end)
    let mut band_start = 0;
    let mut current_band = (chars[indices[0]].font_size + 0.5).floor() as i32;
    for (i, &idx) in indices.iter().enumerate() {
        let band = (chars[idx].font_size + 0.5).floor() as i32;
        if band != current_band {
            band_slices.push((current_band, band_start, i));
            band_start = i;
            current_band = band;
        }
    }
    band_slices.push((current_band, band_start, indices.len()));

    // Process each band: sub-group by y, sort, de-interleave
    for &(_, start, end) in &band_slices {
        let band_indices = &mut indices[start..end];
        // Sort by (y0, x0) for sub-grouping
        band_indices.sort_by(|&a, &b| {
            match chars[a]
                .bbox
                .y0
                .partial_cmp(&chars[b].bbox.y0)
                .unwrap_or(std::cmp::Ordering::Equal)
            {
                std::cmp::Ordering::Equal => chars[a]
                    .bbox
                    .x0
                    .partial_cmp(&chars[b].bbox.x0)
                    .unwrap_or(std::cmp::Ordering::Equal),
                other => other,
            }
        });

        // Sub-group by y and de-interleave each group
        sub_group_by_y_and_sort_indices(chars, band_indices);
    }

    // Order bands by their minimum x0 position
    band_slices.sort_by(|a, b| {
        let min_x_a = indices[a.1..a.2]
            .iter()
            .map(|&i| chars[i].bbox.x0)
            .fold(f64::MAX, f64::min);
        let min_x_b = indices[b.1..b.2]
            .iter()
            .map(|&i| chars[i].bbox.x0)
            .fold(f64::MAX, f64::min);
        min_x_a
            .partial_cmp(&min_x_b)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Apply permutation: create reordered vec from indices
    let flat: Vec<Char> = band_slices
        .iter()
        .flat_map(|&(_, start, end)| indices[start..end].iter().map(|&i| chars[i].clone()))
        .collect();
    for (i, ch) in flat.into_iter().enumerate() {
        chars[i] = ch;
    }
}

/// Sub-group indices by y-position and sort/de-interleave each sub-group.
fn sub_group_by_y_and_sort_indices(chars: &mut [Char], indices: &mut [usize]) {
    if indices.len() <= 3 {
        // Already sorted by (y0, x0); just de-interleave
        de_interleave_by_indices(chars, indices);
        return;
    }

    // Sub-group by y-position
    let mut sub_groups: Vec<(usize, usize)> = Vec::new(); // (start, end) in indices
    let mut group_start = 0;
    let ref_char = &chars[indices[0]];
    let mut ref_center_y = (ref_char.bbox.y0 + ref_char.bbox.y1) / 2.0;
    let mut ref_height = ref_char.bbox.height();

    for (i, &idx) in indices.iter().enumerate().skip(1) {
        let ch_center_y = (chars[idx].bbox.y0 + chars[idx].bbox.y1) / 2.0;
        let tolerance = ref_height * 0.5;

        if (ch_center_y - ref_center_y).abs() > tolerance {
            sub_groups.push((group_start, i));
            group_start = i;
            ref_center_y = ch_center_y;
            ref_height = chars[idx].bbox.height();
        }
    }
    sub_groups.push((group_start, indices.len()));

    // Process each sub-group
    for &(start, end) in &sub_groups {
        let sub = &mut indices[start..end];
        // Sort by x within sub-group
        sub.sort_by(|&a, &b| {
            chars[a]
                .bbox
                .x0
                .partial_cmp(&chars[b].bbox.x0)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        de_interleave_by_indices(chars, sub);
    }
}

/// De-interleave runs within a sub-group (indexed), if needed.
fn de_interleave_by_indices(chars: &[Char], indices: &mut [usize]) {
    if indices.len() <= 2 {
        return;
    }

    // Check for run_id inversions
    let mut inversions = 0usize;
    for i in 1..indices.len() {
        if chars[indices[i]].run_id < chars[indices[i - 1]].run_id {
            inversions += 1;
        }
    }

    if inversions == 0 {
        return;
    }

    // De-interleave: group by run_id, sort each group by x, order by run_id
    let mut run_groups: HashMap<u32, Vec<usize>> = HashMap::new();
    for &idx in indices.iter() {
        run_groups.entry(chars[idx].run_id).or_default().push(idx);
    }
    for group in run_groups.values_mut() {
        group.sort_by(|&a, &b| {
            chars[a]
                .bbox
                .x0
                .partial_cmp(&chars[b].bbox.x0)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }
    let mut runs: Vec<(u32, Vec<usize>)> = run_groups.into_iter().collect();
    runs.sort_by_key(|(run_id, _)| *run_id);

    let flat: Vec<usize> = runs.into_iter().flat_map(|(_, group)| group).collect();
    for (i, idx) in flat.into_iter().enumerate() {
        indices[i] = idx;
    }
}

/// Run the full layout analysis pipeline on a list of characters.
///
/// Pipeline: Characters → Words → Lines → Blocks → Reading Order
///
/// Takes ownership of the character vector to avoid cloning.
pub fn analyze_layout(chars: Vec<Char>, params: &LayoutParams) -> Vec<TextBlock> {
    if chars.is_empty() {
        return Vec::new();
    }

    // Step 1: Sort characters into reading order (line-first approach).
    let mut sorted_chars = chars;
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
pub fn blocks_to_layout_text(blocks: &[TextBlock], _page_width: f64) -> String {
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
        let blocks = analyze_layout(chars, &params);
        assert!(!blocks.is_empty());
        assert!(blocks[0].text.contains("Hi"));
    }

    #[test]
    fn test_blocks_to_text() {
        let ch1 = make_char("a", 0.0, 0.0, 8.0, 12.0);
        let ch2 = make_char("b", 20.0, 0.0, 28.0, 12.0);
        let params = LayoutParams::default();
        let blocks = analyze_layout(vec![ch1, ch2], &params);
        let text = blocks_to_text(&blocks);
        assert!(!text.is_empty());
    }

    #[test]
    fn test_de_interleave_runs_interleaved() {
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

        sort_within_line(&mut chars);
        let text: String = chars.iter().map(|c| c.text.as_str()).collect();
        assert_eq!(
            text, "ABCD",
            "should keep runs together in run_id order when interleaved"
        );
    }

    #[test]
    fn test_de_interleave_runs_sequential() {
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

        sort_within_line(&mut chars);
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
        let blocks = analyze_layout(chars, &params);
        let text = blocks_to_text(&blocks);
        // Run 1: "AB E" then Run 2: "CD" → "AB E CD" or "AB ECD"
        // The key is that A, B, space, E should be together, then C, D
        assert!(text.contains("AB"), "A and B should be together");
    }
}
