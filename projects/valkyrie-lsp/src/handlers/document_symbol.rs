use tower_lsp::lsp_types::*;
use tracing::debug;
use crate::state::{ServerState, DocumentState};
use super::utils::{range_to_lsp_range, span_to_range};

pub struct DocumentSymbolHandler;

impl DocumentSymbolHandler {
    /// 获取文档符号
    pub async fn handle(state: &ServerState, params: DocumentSymbolParams) -> Option<DocumentSymbolResponse> {
        let uri = params.text_document.uri.to_string();
        debug!("Getting document symbols for: {}", uri);

        // 从 AST 中提取符号
        let doc = state.get_document(&uri)?;
        let symbols = Self::extract_symbols_from_ast(doc.ast.as_ref()?, &doc);

        if symbols.is_empty() {
            None
        }
        else {
            Some(DocumentSymbolResponse::Nested(symbols))
        }
    }

    /// 从 AST 中提取符号
    fn extract_symbols_from_ast(ast: &valkyrie_ast::ProgramRoot, doc: &DocumentState) -> Vec<DocumentSymbol> {
        let mut symbols = Vec::new();

        for stmt in &ast.statements {
            if let Some(symbol) = Self::extract_symbol_from_statement(stmt, doc) {
                symbols.push(symbol);
            }
        }

        symbols
    }

    #[allow(deprecated)]
    fn extract_symbol_from_statement(
        stmt: &valkyrie_ast::StatementKind,
        doc: &DocumentState,
    ) -> Option<DocumentSymbol> {
        match stmt {
            valkyrie_ast::StatementKind::Namespace(ns) => Some(DocumentSymbol {
                name: ns.path.to_string(),
                detail: Some("namespace".to_string()),
                kind: SymbolKind::NAMESPACE,
                tags: None,
                range: range_to_lsp_range(&ns.span, doc),
                selection_range: range_to_lsp_range(&ns.path.span.get_range(), doc),
                children: None,
                deprecated: None,
            }),
            valkyrie_ast::StatementKind::Import(imp) => {
                let start = imp.span.get_start() as u32;
                let end = imp.span.get_end() as u32;
                let span = std::ops::Range { start, end };
                Some(DocumentSymbol {
                    name: "import".to_string(),
                    detail: Some("import statement".to_string()),
                    kind: SymbolKind::MODULE,
                    tags: None,
                    range: range_to_lsp_range(&span, doc),
                    selection_range: range_to_lsp_range(&span, doc),
                    children: None,
                    deprecated: None,
                })
            }
            valkyrie_ast::StatementKind::Class(cls) => {
                let mut children = Vec::new();
                for term in &cls.terms {
                    match term {
                        valkyrie_ast::ClassTerm::Field(f) => {
                            children.push(DocumentSymbol {
                                name: f.name.to_string(),
                                detail: f.typing.as_ref().map(|t| format!("{:?}", t)),
                                kind: SymbolKind::FIELD,
                                tags: None,
                                range: range_to_lsp_range(&f.span, doc),
                                selection_range: span_to_range(f.name.span, doc),
                                children: None,
                                deprecated: None,
                            });
                        }
                        valkyrie_ast::ClassTerm::Method(m) => {
                            let mut m_children = Vec::new();
                            if let Some(body) = &m.body {
                                for inner_stmt in &body.terms {
                                    if let Some(child) = Self::extract_symbol_from_statement(inner_stmt, doc) {
                                        m_children.push(child);
                                    }
                                }
                            }
                            children.push(DocumentSymbol {
                                name: m.name.to_string(),
                                detail: Some(format!("fn {}", m.name)),
                                kind: SymbolKind::METHOD,
                                tags: None,
                                range: range_to_lsp_range(&m.span, doc),
                                selection_range: span_to_range(m.name.span(), doc),
                                children: if m_children.is_empty() { None } else { Some(m_children) },
                                deprecated: None,
                            });
                        }
                        _ => {}
                    }
                }
                Some(DocumentSymbol {
                    name: cls.name.to_string(),
                    detail: Some(format!("{:?}", cls.kind)),
                    kind: SymbolKind::CLASS,
                    tags: None,
                    range: range_to_lsp_range(&cls.span, doc),
                    selection_range: span_to_range(cls.name.span, doc),
                    children: if children.is_empty() { None } else { Some(children) },
                    deprecated: None,
                })
            }
            valkyrie_ast::StatementKind::Trait(tr) => {
                let mut children = Vec::new();
                for term in &tr.body {
                    match term {
                        valkyrie_ast::TraitTerm::Method(m) => {
                            children.push(DocumentSymbol {
                                name: m.name.to_string(),
                                detail: Some(format!("fn {}", m.name)),
                                kind: SymbolKind::METHOD,
                                tags: None,
                                range: range_to_lsp_range(&m.span, doc),
                                selection_range: span_to_range(m.name.span(), doc),
                                children: None,
                                deprecated: None,
                            });
                        }
                        _ => {}
                    }
                }
                Some(DocumentSymbol {
                    name: tr.name.to_string(),
                    detail: Some("trait".to_string()),
                    kind: SymbolKind::INTERFACE,
                    tags: None,
                    range: range_to_lsp_range(&tr.span, doc),
                    selection_range: span_to_range(tr.name.span, doc),
                    children: if children.is_empty() { None } else { Some(children) },
                    deprecated: None,
                })
            }
            valkyrie_ast::StatementKind::Union(un) => {
                let mut children = Vec::new();
                for term in &un.body {
                    match term {
                        valkyrie_ast::UnionTerm::Variant(v) => {
                            children.push(DocumentSymbol {
                                name: v.name.to_string(),
                                detail: Some("variant".to_string()),
                                kind: SymbolKind::ENUM_MEMBER,
                                tags: None,
                                range: range_to_lsp_range(&v.span, doc),
                                selection_range: span_to_range(v.name.span, doc),
                                children: None,
                                deprecated: None,
                            });
                        }
                        valkyrie_ast::UnionTerm::Method(m) => {
                            children.push(DocumentSymbol {
                                name: m.name.to_string(),
                                detail: Some(format!("fn {}", m.name)),
                                kind: SymbolKind::METHOD,
                                tags: None,
                                range: range_to_lsp_range(&m.span, doc),
                                selection_range: span_to_range(m.name.span(), doc),
                                children: None,
                                deprecated: None,
                            });
                        }
                        _ => {}
                    }
                }
                Some(DocumentSymbol {
                    name: un.name.to_string(),
                    detail: Some("union".to_string()),
                    kind: SymbolKind::ENUM,
                    tags: None,
                    range: range_to_lsp_range(&un.span, doc),
                    selection_range: span_to_range(un.name.span, doc),
                    children: if children.is_empty() { None } else { Some(children) },
                    deprecated: None,
                })
            }
            valkyrie_ast::StatementKind::Function(func) => {
                let full_range = func.keyword.get_start()..func.body.span.end;
                let mut children = Vec::new();
                for inner_stmt in &func.body.terms {
                    if let Some(child) = Self::extract_symbol_from_statement(inner_stmt, doc) {
                        children.push(child);
                    }
                }
                Some(DocumentSymbol {
                    name: func.name.to_string(),
                    detail: Some(format!("fn {}", func.name)),
                    kind: SymbolKind::FUNCTION,
                    tags: None,
                    range: range_to_lsp_range(&full_range, doc),
                    selection_range: span_to_range(func.name.span(), doc),
                    children: if children.is_empty() { None } else { Some(children) },
                    deprecated: None,
                })
            }
            valkyrie_ast::StatementKind::Variable(var) => {
                let mut children = Vec::new();
                Self::extract_symbols_from_pattern(&var.pattern, doc, &mut children);

                Some(DocumentSymbol {
                    name: Self::pattern_to_string(&var.pattern),
                    detail: Some("variable".to_string()),
                    kind: SymbolKind::VARIABLE,
                    tags: None,
                    range: range_to_lsp_range(&var.span, doc),
                    selection_range: range_to_lsp_range(&var.span, doc),
                    children: if children.is_empty() { None } else { Some(children) },
                    deprecated: None,
                })
            }
            _ => None,
        }
    }

    fn pattern_to_string(pattern: &valkyrie_ast::CasePattern) -> String {
        match pattern {
            valkyrie_ast::CasePattern::Symbol(key) => {
                if let valkyrie_ast::ArgumentKey::Symbol(id) = &**key {
                    id.to_string()
                }
                else {
                    "_".to_string()
                }
            }
            valkyrie_ast::CasePattern::Tuple(node) => {
                let mut s = String::new();
                if let Some(id) = &node.bind {
                    s.push_str(&id.to_string());
                    s.push_str(" <- ");
                }
                if let Some(name) = &node.name {
                    s.push_str(&name.to_string());
                }
                s.push('(');
                for (i, term) in node.terms.iter().enumerate() {
                    if i > 0 {
                        s.push_str(", ");
                    }
                    s.push_str(&Self::pattern_to_string(term));
                }
                s.push(')');
                s
            }
            valkyrie_ast::CasePattern::Class(node) => {
                let mut s = String::new();
                if let Some(id) = &node.bind {
                    s.push_str(&id.to_string());
                    s.push_str(" <- ");
                }
                if let Some(name) = &node.name {
                    s.push_str(&name.to_string());
                }
                s.push('{');
                for (i, term) in node.terms.iter().enumerate() {
                    if i > 0 {
                        s.push_str(", ");
                    }
                    s.push_str(&Self::pattern_to_string(term));
                }
                s.push('}');
                s
            }
            valkyrie_ast::CasePattern::Array(node) => {
                let mut s = String::new();
                if let Some(id) = &node.bind {
                    s.push_str(&id.to_string());
                    s.push_str(" <- ");
                }
                s.push('[');
                for (i, term) in node.terms.iter().enumerate() {
                    if i > 0 {
                        s.push_str(", ");
                    }
                    s.push_str(&Self::pattern_to_string(term));
                }
                s.push(']');
                s
            }
            valkyrie_ast::CasePattern::Union(node) => {
                let mut s = String::new();
                if let Some(id) = &node.bind {
                    s.push_str(&id.to_string());
                    s.push_str(" <- ");
                }
                for (i, term) in node.terms.iter().enumerate() {
                    if i > 0 {
                        s.push_str(" | ");
                    }
                    s.push_str(&Self::pattern_to_string(term));
                }
                s
            }
            valkyrie_ast::CasePattern::Atom(node) => node.identifier.to_string(),
            valkyrie_ast::CasePattern::Wildcard(_) => "_".to_string(),
            valkyrie_ast::CasePattern::Pair(node) => {
                let key_str = match &node.key {
                    valkyrie_ast::ArgumentKey::Nothing => "".to_string(),
                    valkyrie_ast::ArgumentKey::Symbol(id) => id.to_string(),
                };
                format!("{}: {}", key_str, Self::pattern_to_string(&node.value))
            }
        }
    }

    #[allow(deprecated)]
    fn extract_symbols_from_pattern(
        pattern: &valkyrie_ast::CasePattern,
        doc: &DocumentState,
        symbols: &mut Vec<DocumentSymbol>,
    ) {
        match pattern {
            valkyrie_ast::CasePattern::Symbol(key) => {
                if let valkyrie_ast::ArgumentKey::Symbol(id) = &**key {
                    symbols.push(DocumentSymbol {
                        name: id.to_string(),
                        detail: Some("variable".to_string()),
                        kind: SymbolKind::VARIABLE,
                        tags: None,
                        range: span_to_range(id.span, doc),
                        selection_range: span_to_range(id.span, doc),
                        children: None,
                        deprecated: None,
                    });
                }
            }
            valkyrie_ast::CasePattern::Tuple(node) => {
                if let Some(id) = &node.bind {
                    symbols.push(DocumentSymbol {
                        name: id.to_string(),
                        detail: Some("variable".to_string()),
                        kind: SymbolKind::VARIABLE,
                        tags: None,
                        range: span_to_range(id.span, doc),
                        selection_range: span_to_range(id.span, doc),
                        children: None,
                        deprecated: None,
                    });
                }
                for term in &node.terms {
                    Self::extract_symbols_from_pattern(term, doc, symbols);
                }
            }
            valkyrie_ast::CasePattern::Class(node) => {
                if let Some(id) = &node.bind {
                    symbols.push(DocumentSymbol {
                        name: id.to_string(),
                        detail: Some("variable".to_string()),
                        kind: SymbolKind::VARIABLE,
                        tags: None,
                        range: span_to_range(id.span, doc),
                        selection_range: span_to_range(id.span, doc),
                        children: None,
                        deprecated: None,
                    });
                }
                for term in &node.terms {
                    Self::extract_symbols_from_pattern(term, doc, symbols);
                }
            }
            valkyrie_ast::CasePattern::Union(node) => {
                if let Some(id) = &node.bind {
                    symbols.push(DocumentSymbol {
                        name: id.to_string(),
                        detail: Some("variable".to_string()),
                        kind: SymbolKind::VARIABLE,
                        tags: None,
                        range: span_to_range(id.span, doc),
                        selection_range: span_to_range(id.span, doc),
                        children: None,
                        deprecated: None,
                    });
                }
                for term in &node.terms {
                    Self::extract_symbols_from_pattern(term, doc, symbols);
                }
            }
            valkyrie_ast::CasePattern::Array(node) => {
                if let Some(id) = &node.bind {
                    symbols.push(DocumentSymbol {
                        name: id.to_string(),
                        detail: Some("variable".to_string()),
                        kind: SymbolKind::VARIABLE,
                        tags: None,
                        range: span_to_range(id.span, doc),
                        selection_range: span_to_range(id.span, doc),
                        children: None,
                        deprecated: None,
                    });
                }
                for term in &node.terms {
                    Self::extract_symbols_from_pattern(term, doc, symbols);
                }
            }
            valkyrie_ast::CasePattern::Atom(node) => {
                symbols.push(DocumentSymbol {
                    name: node.identifier.to_string(),
                    detail: Some("variable".to_string()),
                    kind: SymbolKind::VARIABLE,
                    tags: None,
                    range: span_to_range(node.identifier.span, doc),
                    selection_range: span_to_range(node.identifier.span, doc),
                    children: None,
                    deprecated: None,
                });
            }
            valkyrie_ast::CasePattern::Wildcard(_) => {}
            valkyrie_ast::CasePattern::Pair(node) => {
                Self::extract_symbols_from_pattern(&node.value, doc, symbols);
            }
        }
    }
}
