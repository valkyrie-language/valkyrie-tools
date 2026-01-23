use core::range::Range;

pub fn span_to_range_usize(span: valkyrie_error::SourceSpan) -> Range<usize> {
    Range { start: span.get_start() as usize, end: span.get_end() as usize }
}

pub fn range_to_lsp_range_usize(range: &std::ops::Range<u32>) -> Range<usize> {
    Range { start: range.start as usize, end: range.end as usize }
}
