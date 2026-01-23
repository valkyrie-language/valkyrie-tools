use tower_lsp::lsp_types::Range;
use crate::state::DocumentState;

pub fn span_to_range(span: valkyrie_error::SourceSpan, doc: &DocumentState) -> Range {
    let start = span.get_start() as usize;
    let end = span.get_end() as usize;
    Range { start: doc.offset_to_position(start), end: doc.offset_to_position(end) }
}

pub fn range_to_lsp_range(range: &std::ops::Range<u32>, doc: &DocumentState) -> Range {
    Range { start: doc.offset_to_position(range.start as usize), end: doc.offset_to_position(range.end as usize) }
}
