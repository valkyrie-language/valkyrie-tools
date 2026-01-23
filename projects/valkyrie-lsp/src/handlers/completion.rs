use oak_lsp::types::{CompletionItem, CompletionItemKind, Position};
use crate::state::ServerState;
use crate::errors::LspResult;

/// 补全处理器
pub struct CompletionHandler;

impl CompletionHandler {
    pub async fn handle(
        state: &ServerState,
        _uri: &str,
        _position: Position,
    ) -> LspResult<Vec<CompletionItem>> {
        let mut items = Vec::new();

        // 1. 添加关键字补全
        let keywords = vec![
            "namespace", "import", "class", "trait", "union", "fn", "let", "mut", "var", "if", "else", "match", "loop", "while", "for", "return", "break",
            "continue", "yield", "raise", "catch", "theorem", "forall", "exists", "where", "macro",
        ];

        for kw in keywords {
            items.push(CompletionItem {
                label: kw.to_string(),
                kind: Some(CompletionItemKind::Keyword),
                detail: Some("keyword".to_string()),
                documentation: None,
                insert_text: None,
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
                                kind: Some(CompletionItemKind::Function),
                                detail: Some(format!("fn {}", f.name)),
                                documentation: None,
                                insert_text: None,
                            });
                        }
                        valkyrie_ast::StatementKind::Class(c) => {
                            items.push(CompletionItem {
                                label: c.name.to_string(),
                                kind: Some(CompletionItemKind::Class),
                                detail: Some(format!("class {}", c.name)),
                                documentation: None,
                                insert_text: None,
                            });
                            // 同时也添加类成员
                            for term in &c.terms {
                                match term {
                                    valkyrie_ast::ClassTerm::Field(f) => {
                                        items.push(CompletionItem {
                                            label: f.name.to_string(),
                                            kind: Some(CompletionItemKind::Field),
                                            detail: Some(format!("field {}.{}", c.name, f.name)),
                                            documentation: None,
                                            insert_text: None,
                                        });
                                    }
                                    valkyrie_ast::ClassTerm::Method(m) => {
                                        items.push(CompletionItem {
                                            label: m.name.to_string(),
                                            kind: Some(CompletionItemKind::Method),
                                            detail: Some(format!("method {}.{}", c.name, m.name)),
                                            documentation: None,
                                            insert_text: None,
                                        });
                                    }
                                    _ => {}
                                }
                            }
                        }
                        valkyrie_ast::StatementKind::Namespace(n) => {
                            items.push(CompletionItem {
                                label: n.path.to_string(),
                                kind: Some(CompletionItemKind::Module),
                                detail: Some("namespace".to_string()),
                                documentation: None,
                                insert_text: None,
                            });
                        }
                        valkyrie_ast::StatementKind::Trait(t) => {
                            items.push(CompletionItem {
                                label: t.name.to_string(),
                                kind: Some(CompletionItemKind::Interface),
                                detail: Some("trait".to_string()),
                                documentation: None,
                                insert_text: None,
                            });
                        }
                        valkyrie_ast::StatementKind::Variable(v) => {
                            // 从模式中提取变量名
                            if let valkyrie_ast::CasePattern::Symbol(id) = &v.pattern {
                                if let valkyrie_ast::ArgumentKey::Symbol(node) = &**id {
                                    items.push(CompletionItem {
                                        label: node.name.to_string(),
                                        kind: Some(CompletionItemKind::Variable),
                                        detail: Some("variable".to_string()),
                                        documentation: None,
                                        insert_text: None,
                                    });
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        // 3. 去重
        items.sort_by(|a, b| a.label.cmp(&b.label));
        items.dedup_by(|a, b| a.label == b.label);

        Ok(items)
    }
}
