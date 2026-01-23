use oak_lsp::types::*;
use crate::state::{ServerState, DocumentState};

/// 折叠范围处理器
pub struct FoldingRangeHandler;

impl FoldingRangeHandler {
    pub async fn handle(state: &ServerState, uri: &str) -> Vec<FoldingRange> {
        let doc = match state.get_document(uri) {
            Some(d) => d,
            None => return vec![],
        };
        let ast = match doc.ast.as_ref() {
            Some(a) => a,
            None => return vec![],
        };

        let mut ranges = Vec::new();
        Self::collect_folding_ranges(ast, &doc, &mut ranges);

        ranges
    }

    fn collect_folding_ranges(
        ast: &valkyrie_ast::ProgramRoot,
        doc: &DocumentState,
        ranges: &mut Vec<FoldingRange>,
    ) {
        for stmt in &ast.statements {
            Self::collect_from_statement(stmt, doc, ranges);
        }
    }

    fn collect_from_statement(
        stmt: &valkyrie_ast::StatementKind,
        doc: &DocumentState,
        ranges: &mut Vec<FoldingRange>,
    ) {
        match stmt {
            valkyrie_ast::StatementKind::Class(cls) => {
                ranges.push(Self::span_to_folding_range(cls.span.clone(), doc));
                for term in &cls.terms {
                    if let valkyrie_ast::ClassTerm::Method(m) = term {
                        if let Some(body) = &m.body {
                            ranges.push(Self::span_to_folding_range(body.span.clone(), doc));
                        }
                    }
                }
            }
            valkyrie_ast::StatementKind::Function(func) => {
                ranges.push(Self::span_to_folding_range(func.body.span.clone(), doc));
            }
            valkyrie_ast::StatementKind::Trait(tr) => {
                ranges.push(Self::span_to_folding_range(tr.span.clone(), doc));
            }
            _ => {}
        }
    }

    fn span_to_folding_range(range: std::ops::Range<u32>, doc: &DocumentState) -> FoldingRange {
        let start = doc.offset_to_position(range.start as usize);
        let end = doc.offset_to_position(range.end as usize);
        FoldingRange {
            start_line: start.line,
            start_character: Some(start.character),
            end_line: end.line,
            end_character: Some(end.character),
            kind: Some(FoldingRangeKind::Region),
            collapsed_text: None,
        }
    }
}
