use tower_lsp::lsp_types::*;
use crate::state::ServerState;

/// 补全处理器
pub struct CompletionHandler;

impl CompletionHandler {
    pub async fn handle(
        state: &ServerState,
        params: CompletionParams,
    ) -> Result<Option<CompletionResponse>, Box<dyn std::error::Error + Send + Sync>> {
        let _uri = params.text_document_position.text_document.uri.to_string();
        let _position = params.text_document_position.position;

        let mut items = Vec::new();

        // 1. 添加关键字补全
        let keywords = vec![
            "namespace", "import", "class", "trait", "union", "fn", "let", "mut", "var", "if", "else", "match", "loop", "while", "for", "return", "break",
            "continue", "yield", "raise", "catch", "theorem", "forall", "exists", "where", "macro",
        ];

        for kw in keywords {
            items.push(CompletionItem {
                label: kw.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("keyword".to_string()),
                ..Default::default()
            });
        }

        // 2. 基础补全：从所有已编译文档的 AST 中提取所有符号
        for doc_ref in state.documents.iter() {
            let doc = doc_ref.value();
            if let Some(ast) = &doc.ast {
                for stmt in &ast.statements {
                    match stmt {
                        valkyrie_ast::StatementKind::Function(f) => {
                            items.push(CompletionItem {
                                label: f.name.to_string(),
                                kind: Some(CompletionItemKind::FUNCTION),
                                detail: Some(format!("fn {}", f.name)),
                                ..Default::default()
                            });
                        }
                        valkyrie_ast::StatementKind::Class(c) => {
                            items.push(CompletionItem {
                                label: c.name.to_string(),
                                kind: Some(CompletionItemKind::CLASS),
                                detail: Some(format!("class {}", c.name)),
                                ..Default::default()
                            });
                            // 同时也添加类成员
                            for term in &c.terms {
                                match term {
                                    valkyrie_ast::ClassTerm::Field(f) => {
                                        items.push(CompletionItem {
                                            label: f.name.to_string(),
                                            kind: Some(CompletionItemKind::FIELD),
                                            detail: Some(format!("field {}.{}", c.name, f.name)),
                                            ..Default::default()
                                        });
                                    }
                                    valkyrie_ast::ClassTerm::Method(m) => {
                                        items.push(CompletionItem {
                                            label: m.name.to_string(),
                                            kind: Some(CompletionItemKind::METHOD),
                                            detail: Some(format!("method {}.{}", c.name, m.name)),
                                            ..Default::default()
                                        });
                                    }
                                    _ => {}
                                }
                            }
                        }
                        valkyrie_ast::StatementKind::Namespace(n) => {
                            items.push(CompletionItem {
                                label: n.path.to_string(),
                                kind: Some(CompletionItemKind::MODULE),
                                detail: Some("namespace".to_string()),
                                ..Default::default()
                            });
                        }
                        valkyrie_ast::StatementKind::Trait(t) => {
                            items.push(CompletionItem {
                                label: t.name.to_string(),
                                kind: Some(CompletionItemKind::INTERFACE),
                                detail: Some("trait".to_string()),
                                ..Default::default()
                            });
                        }
                        valkyrie_ast::StatementKind::Variable(v) => {
                            // 从模式中提取变量名
                            if let valkyrie_ast::CasePattern::Symbol(id) = &v.pattern {
                                if let valkyrie_ast::ArgumentKey::Symbol(node) = &**id {
                                    items.push(CompletionItem {
                                        label: node.name.to_string(),
                                        kind: Some(CompletionItemKind::VARIABLE),
                                        detail: Some("variable".to_string()),
                                        ..Default::default()
                                    });
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        // 去重
        items.sort_by(|a, b| a.label.cmp(&b.label));
        items.dedup_by(|a, b| a.label == b.label && a.kind == b.kind);

        Ok(Some(CompletionResponse::Array(items)))
    }
}
