use oak_lsp::types::{StructureItem, SymbolKind};
use tracing::debug;
use crate::state::{ServerState, DocumentState};
use super::utils::{range_to_lsp_range_usize, span_to_range_usize};

pub struct DocumentSymbolHandler;

impl DocumentSymbolHandler {
    /// 获取文档符号
    pub async fn handle(state: &ServerState, uri: &str) -> Vec<StructureItem> {
        debug!("Getting document symbols for: {}", uri);

        // 从 AST 中提取符号
        let doc = match state.get_document(uri) {
            Some(d) => d,
            None => return vec![],
        };
        let ast = match doc.ast.as_ref() {
            Some(a) => a,
            None => return vec![],
        };
        Self::extract_symbols_from_ast(ast, &doc)
    }

    /// 从 AST 中提取符号
    fn extract_symbols_from_ast(ast: &valkyrie_ast::ProgramRoot, doc: &DocumentState) -> Vec<StructureItem> {
        let mut symbols = Vec::new();

        for stmt in &ast.statements {
            if let Some(symbol) = Self::extract_symbol_from_statement(stmt, doc) {
                symbols.push(symbol);
            }
        }

        symbols
    }

    fn extract_symbol_from_statement(
        stmt: &valkyrie_ast::StatementKind,
        doc: &DocumentState,
    ) -> Option<StructureItem> {
        match stmt {
            valkyrie_ast::StatementKind::Namespace(ns) => Some(StructureItem {
                name: ns.path.to_string(),
                detail: Some("namespace".to_string()),
                role: oak_lsp::types::UniversalElementRole::Container,
                kind: SymbolKind::Namespace,
                range: range_to_lsp_range_usize(&ns.span),
                selection_range: range_to_lsp_range_usize(&ns.path.span.get_range()),
                children: vec![],
                deprecated: false,
            }),
            valkyrie_ast::StatementKind::Import(imp) => {
                let start = imp.span.get_start() as usize;
                let end = imp.span.get_end() as usize;
                let span = core::range::Range { start, end };
                Some(StructureItem {
                    name: "import".to_string(),
                    detail: Some("import statement".to_string()),
                    role: oak_lsp::types::UniversalElementRole::Metadata,
                    kind: SymbolKind::Module,
                    range: span,
                    selection_range: span,
                    children: vec![],
                    deprecated: false,
                })
            }
            valkyrie_ast::StatementKind::Class(cls) => {
                let mut children = Vec::new();
                for term in &cls.terms {
                    match term {
                        valkyrie_ast::ClassTerm::Field(f) => {
                            children.push(StructureItem {
                                name: f.name.to_string(),
                                detail: f.typing.as_ref().map(|t| format!("{:?}", t)),
                                role: oak_lsp::types::UniversalElementRole::Binding,
                                kind: SymbolKind::Field,
                                range: range_to_lsp_range_usize(&f.span),
                                selection_range: span_to_range_usize(f.name.span),
                                children: vec![],
                                deprecated: false,
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
                            children.push(StructureItem {
                                name: m.name.to_string(),
                                detail: Some(format!("fn {}", m.name)),
                                role: oak_lsp::types::UniversalElementRole::Definition,
                                kind: SymbolKind::Method,
                                range: range_to_lsp_range_usize(&m.span),
                                selection_range: span_to_range_usize(m.name.span),
                                children: m_children,
                                deprecated: false,
                            });
                        }
                        _ => {}
                    }
                }
                Some(StructureItem {
                    name: cls.name.to_string(),
                    detail: Some(format!("class {}", cls.name)),
                    role: oak_lsp::types::UniversalElementRole::Typing,
                    kind: SymbolKind::Class,
                    range: range_to_lsp_range_usize(&cls.span),
                    selection_range: span_to_range_usize(cls.name.span),
                    children,
                    deprecated: false,
                })
            }
            valkyrie_ast::StatementKind::Function(f) => {
                let mut children = Vec::new();
                if let Some(body) = &f.body {
                    for inner_stmt in &body.terms {
                        if let Some(child) = Self::extract_symbol_from_statement(inner_stmt, doc) {
                            children.push(child);
                        }
                    }
                }
                Some(StructureItem {
                    name: f.name.to_string(),
                    detail: Some(format!("fn {}", f.name)),
                    role: oak_lsp::types::UniversalElementRole::Definition,
                    kind: SymbolKind::Function,
                    range: range_to_lsp_range_usize(&f.span),
                    selection_range: span_to_range_usize(f.name.span),
                    children,
                    deprecated: false,
                })
            }
            _ => None,
        }
    }
}
