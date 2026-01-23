use tower_lsp::lsp_types::*;
use valkyrie_ast::helper::ValkyrieNode;
use crate::state::{ServerState, DocumentState};
use super::utils::range_to_lsp_range;

/// 选择范围处理器
pub struct SelectionRangeHandler;

impl SelectionRangeHandler {
    pub async fn handle(state: &ServerState, params: SelectionRangeParams) -> Option<Vec<SelectionRange>> {
        let uri = params.text_document.uri.to_string();
        let doc = state.get_document(&uri)?;
        let ast = doc.ast.as_ref()?;

        let mut result = Vec::new();
        for position in params.positions {
            let offset = doc.position_to_offset(position) as u32;
            let mut ranges = Vec::new();
            Self::collect_selection_ranges(&ast.statements, offset, &doc, &mut ranges);

            if let Some(mut current) = ranges.pop() {
                while let Some(parent_range) = ranges.pop() {
                    current = SelectionRange {
                        range: current.range,
                        parent: Some(Box::new(SelectionRange { range: parent_range.range, parent: None })),
                    };
                }
                result.push(current);
            }
        }

        if result.is_empty() {
            None
        }
        else {
            Some(result)
        }
    }

    fn collect_selection_ranges(
        statements: &[valkyrie_ast::StatementKind],
        offset: u32,
        doc: &DocumentState,
        ranges: &mut Vec<SelectionRange>,
    ) {
        for stmt in statements {
            let span = stmt.get_range();
            if offset >= span.start && offset <= span.end {
                ranges.push(SelectionRange { range: range_to_lsp_range(&span, doc), parent: None });

                match stmt {
                    valkyrie_ast::StatementKind::Function(f) => {
                        if offset >= f.body.span.start && offset <= f.body.span.end {
                            Self::collect_selection_ranges(&f.body.terms, offset, doc, ranges);
                        }
                    }
                    valkyrie_ast::StatementKind::Class(c) => {
                        for term in &c.terms {
                            if let valkyrie_ast::ClassTerm::Method(m) = term {
                                if let Some(body) = &m.body {
                                    if offset >= body.span.start && offset <= body.span.end {
                                        Self::collect_selection_ranges(&body.terms, offset, doc, ranges);
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
                break;
            }
        }
    }
}
