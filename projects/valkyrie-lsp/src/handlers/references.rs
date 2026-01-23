use oak_lsp::types::{LocationRange, Position};
use crate::state::{ServerState, DocumentState};
use super::utils::{span_to_range_usize, range_to_lsp_range_usize};

/// 引用处理器
pub struct ReferencesHandler;

impl ReferencesHandler {
    pub async fn handle(state: &ServerState, uri: &str, position: Position) -> Vec<LocationRange> {
        // 1. 找到当前位置的符号
        let symbol = match state.query_symbol_at_position(uri, position).await {
            Some(s) => s,
            None => return vec![],
        };
        let name = symbol.name;

        let mut locations = Vec::new();

        // 2. 遍历所有打开的文档，在 AST 中搜索引用
        for doc_ref in state.documents.iter() {
            let doc_uri = doc_ref.key();
            let doc_state = doc_ref.value();
            if let Some(ast) = &doc_state.ast {
                Self::collect_references(&ast.statements, &name, doc_uri, doc_state, &mut locations);
            }
        }

        locations
    }

    fn collect_references(
        statements: &[valkyrie_ast::StatementKind],
        name: &str,
        uri: &str,
        doc: &DocumentState,
        locations: &mut Vec<LocationRange>,
    ) {
        for stmt in statements {
            match stmt {
                valkyrie_ast::StatementKind::Function(f) => {
                    if f.name.to_string() == name {
                        locations.push(LocationRange {
                            uri: uri.to_string(),
                            range: span_to_range_usize(f.name.span()),
                        });
                    }
                    Self::collect_references(&f.body.terms, name, uri, doc, locations);
                }
                valkyrie_ast::StatementKind::Class(c) => {
                    if c.name.to_string() == name {
                        locations.push(LocationRange {
                            uri: uri.to_string(),
                            range: span_to_range_usize(c.name.span),
                        });
                    }
                    for term in &c.terms {
                        match term {
                            valkyrie_ast::ClassTerm::Field(f) => {
                                if f.name.to_string() == name {
                                    locations.push(LocationRange {
                                        uri: uri.to_string(),
                                        range: span_to_range_usize(f.name.span),
                                    });
                                }
                            }
                            valkyrie_ast::ClassTerm::Method(m) => {
                                if m.name.to_string() == name {
                                    locations.push(LocationRange {
                                        uri: uri.to_string(),
                                        range: span_to_range_usize(m.name.span()),
                                    });
                                }
                                if let Some(body) = &m.body {
                                    Self::collect_references(&body.terms, name, uri, doc, locations);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                valkyrie_ast::StatementKind::Expression(e) => {
                    Self::collect_references_in_expr(&e.body, name, uri, doc, locations);
                }
                valkyrie_ast::StatementKind::Variable(v) => {
                    Self::collect_references_in_pattern(&v.pattern, name, uri, doc, locations);
                    if let Some(body) = &v.body {
                        Self::collect_references_in_expr(body, name, uri, doc, locations);
                    }
                }
                valkyrie_ast::StatementKind::Each(e) => {
                    Self::collect_references_in_pattern(&e.pattern, name, uri, doc, locations);
                    Self::collect_references_in_expr(&e.iterator, name, uri, doc, locations);
                    Self::collect_references(&e.body.terms, name, uri, doc, locations);
                }
                _ => {}
            }
        }
    }

    fn collect_references_in_pattern(
        pattern: &valkyrie_ast::CasePattern,
        name: &str,
        uri: &str,
        doc: &DocumentState,
        locations: &mut Vec<LocationRange>,
    ) {
        match pattern {
            valkyrie_ast::CasePattern::Symbol(s) => {
                if let valkyrie_ast::ArgumentKey::Symbol(node) = &**s {
                    if node.name.to_string() == name {
                        locations.push(LocationRange {
                            uri: uri.to_string(),
                            range: span_to_range_usize(node.span.clone()),
                        });
                    }
                }
            }
            valkyrie_ast::CasePattern::Tuple(t) => {
                for p in &t.terms {
                    Self::collect_references_in_pattern(p, name, uri, doc, locations);
                }
            }
            valkyrie_ast::CasePattern::Array(a) => {
                for p in &a.terms {
                    Self::collect_references_in_pattern(p, name, uri, doc, locations);
                }
            }
            valkyrie_ast::CasePattern::Class(c) => {
                for p in &c.terms {
                    Self::collect_references_in_pattern(p, name, uri, doc, locations);
                }
            }
            _ => {}
        }
    }

    fn collect_references_in_expr(
        expr: &valkyrie_ast::ExpressionKind,
        name: &str,
        uri: &str,
        doc: &DocumentState,
        locations: &mut Vec<LocationRange>,
    ) {
        match expr {
            valkyrie_ast::ExpressionKind::Symbol(s) => {
                if s.to_string() == name {
                    locations.push(LocationRange {
                        uri: uri.to_string(),
                        range: range_to_lsp_range_usize(&s.get_range()),
                    });
                }
            }
            valkyrie_ast::ExpressionKind::ApplyCall(call) => {
                Self::collect_references_in_expr(&call.caller, name, uri, doc, locations);
                for term in &call.arguments.terms {
                    Self::collect_references_in_expr(&term.value, name, uri, doc, locations);
                }
                if let Some(body) = &call.body {
                    Self::collect_references(&body.terms, name, uri, doc, locations);
                }
            }
            valkyrie_ast::ExpressionKind::DotCall(call) => {
                Self::collect_references_in_expr(&call.base, name, uri, doc, locations);
                if let valkyrie_ast::DotCallTerm::Symbol(p) = &call.term {
                    if p.to_string() == name {
                        locations.push(LocationRange {
                            uri: uri.to_string(),
                            range: span_to_range_usize(p.span.clone()),
                        });
                    }
                }
            }
            valkyrie_ast::ExpressionKind::Infix(infix) => {
                Self::collect_references_in_expr(&infix.lhs, name, uri, doc, locations);
                Self::collect_references_in_expr(&infix.rhs, name, uri, doc, locations);
            }
            valkyrie_ast::ExpressionKind::Lambda(lambda) => {
                for param in lambda.parameters.terms() {
                    if param.key.to_string() == name {
                        locations.push(LocationRange {
                            uri: uri.to_string(),
                            range: span_to_range_usize(param.key.span.clone()),
                        });
                    }
                }
                Self::collect_references(&lambda.body.terms, name, uri, doc, locations);
            }
            valkyrie_ast::ExpressionKind::Tuple(tuple) => {
                for term in &tuple.terms.terms {
                    Self::collect_references_in_expr(&term.value, name, uri, doc, locations);
                }
            }
            valkyrie_ast::ExpressionKind::Array(array) => {
                for term in &array.terms {
                    match term {
                        valkyrie_ast::RangeTermNode::Index { index, .. } => {
                            Self::collect_references_in_expr(index, name, uri, doc, locations);
                        }
                        valkyrie_ast::RangeTermNode::Range { head, tail, step } => {
                            if let Some(h) = head {
                                Self::collect_references_in_expr(h, name, uri, doc, locations);
                            }
                            if let Some(t) = tail {
                                Self::collect_references_in_expr(t, name, uri, doc, locations);
                            }
                            if let Some(s) = step {
                                Self::collect_references_in_expr(s, name, uri, doc, locations);
                            }
                        }
                    }
                }
            }
            valkyrie_ast::ExpressionKind::If(if_expr) => {
                for branch in &if_expr.branches {
                    Self::collect_references_in_expr(&branch.condition.body, name, uri, doc, locations);
                    Self::collect_references(&branch.body.terms, name, uri, doc, locations);
                }
                if let Some(else_branch) = &if_expr.else_body {
                    Self::collect_references(&else_branch.body.terms, name, uri, doc, locations);
                }
            }
            _ => {}
        }
    }
}
