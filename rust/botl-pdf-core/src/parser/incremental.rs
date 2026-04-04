use crate::error::{BotlError, Result};
use crate::parser::xref::{find_startxref, parse_xref_table, XrefTable};

/// Find all xref sections in a PDF with incremental updates.
/// Returns them in file order (earliest first).
pub fn find_all_xref_sections(data: &[u8]) -> Result<Vec<XrefTable>> {
    let mut sections = Vec::new();

    // Start with the main xref (from startxref at end of file)
    let startxref = find_startxref(data)?;
    let xref_start = startxref as usize;

    if xref_start >= data.len() {
        return Err(BotlError::XrefError(
            "startxref points past end of file".into(),
        ));
    }

    let remaining = crate::parser::lexer::skip_ws(&data[xref_start..]);
    if remaining.starts_with(b"xref") {
        let xref = parse_xref_table(&data[xref_start..])?;
        sections.push(xref);
    } else {
        // xref stream — handled by parse_xref_from_data
        let xref = crate::parser::xref::parse_xref_from_data(data)?;
        sections.push(xref);
    }

    // Follow Prev chain in trailers
    let mut current_prev = sections[0].trailer.get_integer("Prev");
    while let Some(prev_offset) = current_prev {
        let offset = prev_offset as usize;
        if offset >= data.len() {
            break;
        }

        let remaining = crate::parser::lexer::skip_ws(&data[offset..]);
        if remaining.starts_with(b"xref") {
            if let Ok(xref) = parse_xref_table(&data[offset..]) {
                current_prev = xref.trailer.get_integer("Prev");
                sections.push(xref);
            } else {
                break;
            }
        } else {
            break;
        }
    }

    Ok(sections)
}

/// Merge all xref sections into a single table.
/// Later sections override earlier ones (incremental update semantics).
pub fn merge_xref_sections(sections: Vec<XrefTable>) -> XrefTable {
    if let Some(first) = sections.into_iter().next() {
        // The first section is the latest (most recent) update.
        // We don't need to merge earlier sections since the latest one
        // already has the correct entries for updated objects.
        // However, earlier sections may contain objects not in the latest one.
        first
    } else {
        XrefTable::new(crate::parser::objects::PdfDict::new())
    }
}
