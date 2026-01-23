use oak_lsp::types::*;
use crate::state::{ServerState, DocumentState};
use super::utils::{span_to_range, range_to_lsp_range_usize};

/// 文档高亮处理器
pub struct DocumentHighlightHandler;

impl DocumentHighlightHandler {
    pub async fn handle(state: &ServerState, uri: &str, position: Position) -> Vec<DocumentHighlight> {
        let doc = match state.get_document(uri) {
            Some(d) => d,
            None => return vec![],
        };
        let ast = match doc.ast.as_ref() {
            Some(a) => a,
            None => return vec![],
        };

        // 1. 找到当前位置的符号
        let symbol = match state.query_symbol_at_position(uri, position).await {
            Some(s) => s,
            None => return vec![],
        };
        let name = symbol.name;

        // 2. 在文档中搜索所有同名符号
        let mut highlights = Vec::new();
        Self::collect_highlights(&ast.statements, &name, &doc, &mut highlights);

        highlights
    }

    pub fn collect_highlights(
        statements: &[valkyrie_ast::StatementKind],
        name: &str,
        doc: &DocumentState,
        highlights: &mut Vec<DocumentHighlight>,
    ) {
        for stmt in statements {
            match stmt {
                valkyrie_ast::StatementKind::Function(f) => {
                    if f.name.to_string() == name {
                        highlights.push(DocumentHighlight {
                            range: span_to_range(f.name.span(), doc),
                            kind: Some(DocumentHighlightKind::Write),
                        });
                    }
                    Self::collect_highlights(&f.body.terms, name, doc, highlights);
                }
                valkyrie_ast::StatementKind::Class(c) => {
                    if c.name.to_string() == name {
                        highlights.push(DocumentHighlight {
                            range: span_to_range(c.name.span, doc),
                            kind: Some(DocumentHighlightKind::Write),
                        });
                    }
                    for term in &c.terms {
                        match term {
                            valkyrie_ast::ClassTerm::Field(f) => {
                                if f.name.to_string() == name {
                                    highlights.push(DocumentHighlight {
                                        range: span_to_range(f.name.span, doc),
                                        kind: Some(DocumentHighlightKind::Write),
                                    });
                                }
                            }
                            valkyrie_ast::ClassTerm::Method(m) => {
                                if m.name.to_string() == name {
                                    highlights.push(DocumentHighlight {
                                        range: span_to_range(m.name.span(), doc),
                                        kind: Some(DocumentHighlightKind::Write),
                                    });
                                }
                                if let Some(body) = &m.body {
                                    Self::collect_highlights(&body.terms, name, doc, highlights);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                valkyrie_ast::StatementKind::Variable(v) => {
                    Self::collect_highlights_in_pattern(&v.pattern, name, doc, highlights);
                    if let Some(body) = &v.body {
                        Self::collect_highlights_in_expr(body, name, doc, highlights);
                    }
                }
                valkyrie_ast::StatementKind::Expression(e) => {
                    Self::collect_highlights_in_expr(&e.body, name, doc, highlights);
                }
                valkyrie_ast::StatementKind::Each(e) => {
                    Self::collect_highlights_in_pattern(&e.pattern, name, doc, highlights);
                    Self::collect_highlights_in_expr(&e.iterator, name, doc, highlights);
                    Self::collect_highlights(&e.body.terms, name, doc, highlights);
                }
                _ => {}
            }
        }
    }

    fn collect_highlights_in_pattern(
        pattern: &valkyrie_ast::CasePattern,
        name: &str,
        doc: &DocumentState,
        highlights: &mut Vec<DocumentHighlight>,
    ) {
        match pattern {
            valkyrie_ast::CasePattern::Symbol(s) => {
                if let valkyrie_ast::ArgumentKey::Symbol(node) = &**s {
                    if node.name.to_string() == name {
                        highlights.push(DocumentHighlight {
                            range: span_to_range(node.span.clone(), doc),
                            kind: Some(DocumentHighlightKind::Write),
                        });
                    }
                }
            }
            valkyrie_ast::CasePattern::Tuple(t) => {
                for p in &t.terms {
                    Self::collect_highlights_in_pattern(p, name, doc, highlights);
                }
            }
            valkyrie_ast::CasePattern::Array(a) => {
                for p in &a.terms {
                    Self::collect_highlights_in_pattern(p, name, doc, highlights);
                }
            }
            valkyrie_ast::CasePattern::Class(c) => {
                for p in &c.terms {
                    Self::collect_highlights_in_pattern(p, name, doc, highlights);
                }
            }
            _ => {}
        }
    }

    fn collect_highlights_in_expr(
        expr: &valkyrie_ast::ExpressionKind,
        name: &str,
        doc: &DocumentState,
        highlights: &mut Vec<DocumentHighlight>,
    ) {
        match expr {
            valkyrie_ast::ExpressionKind::Symbol(s) => {
                if s.to_string() == name {
                    highlights.push(DocumentHighlight {
                        range: range_to_lsp_range_usize(&s.get_range(), doc),
                        kind: Some(DocumentHighlightKind::Read),
                    });
                }
            }
            valkyrie_ast::ExpressionKind::ApplyCall(call) => {
                Self::collect_highlights_in_expr(&call.caller, name, doc, highlights);
                for term in &call.arguments.terms {
                    Self::collect_highlights_in_expr(&term.value, name, doc, highlights);
                }
                if let Some(body) = &call.body {
                    Self::collect_highlights(&body.terms, name, doc, highlights);
                }
            }
            valkyrie_ast::ExpressionKind::DotCall(call) => {
                Self::collect_highlights_in_expr(&call.base, name, doc, highlights);
                if let valkyrie_ast::DotCallTerm::Symbol(p) = &call.term {
                    if p.to_string() == name {
                        highlights.push(DocumentHighlight {
                            range: span_to_range(p.span.clone(), doc),
                            kind: Some(DocumentHighlightKind::Read),
                        });
                    }
                }
            }
            valkyrie_ast::ExpressionKind::Infix(infix) => {
                Self::collect_highlights_in_expr(&infix.lhs, name, doc, highlights);
                Self::collect_highlights_in_expr(&infix.rhs, name, doc, highlights);
            }
            valkyrie_ast::ExpressionKind::Lambda(lambda) => {
                for param in lambda.parameters.terms() {
                    if param.key.to_string() == name {
                        highlights.push(DocumentHighlight {
                            range: span_to_range(param.key.span.clone(), doc),
                            kind: Some(DocumentHighlightKind::Write),
                        });
                    }
                }
                Self::collect_highlights(&lambda.body.terms, name, doc, highlights);
            }
            valkyrie_ast::ExpressionKind::Tuple(tuple) => {
                for term in &tuple.terms.terms {
                    Self::collect_highlights_in_expr(&term.value, name, doc, highlights);
                }
            }
            valkyrie_ast::ExpressionKind::Array(array) => {
                for term in &array.terms {
                    match term {
                        valkyrie_ast::RangeTermNode::Index { index, .. } => {
                            Self::collect_highlights_in_expr(index, name, doc, highlights);
                        }
                        valkyrie_ast::RangeTermNode::Range { head, tail, step } => {
                            if let Some(h) = head {
                                Self::collect_highlights_in_expr(h, name, doc, highlights);
                            }
                            if let Some(t) = tail {
                                Self::collect_highlights_in_expr(t, name, doc, highlights);
                            }
                            if let Some(s) = step {
                                Self::collect_highlights_in_expr(s, name, doc, highlights);
                            }
                        }
                    }
                }
            }
            valkyrie_ast::ExpressionKind::If(if_expr) => {
                for branch in &if_expr.branches {
                    Self::collect_highlights_in_expr(&branch.condition.body, name, doc, highlights);
                    Self::collect_highlights(&branch.body.terms, name, doc, highlights);
                }
                if let Some(else_body) = &if_expr.else_body {
                    Self::collect_highlights(&else_body.body.terms, name, doc, highlights);
                }
            }
            _ => {}
        }
    }
}
