use oak_lsp::types::*;
use crate::state::{ServerState, DocumentState};

/// 语义标记处理器
pub struct SemanticTokensHandler;

impl SemanticTokensHandler {
    pub async fn handle_full(state: &ServerState, uri: &str) -> Option<SemanticTokens> {
        let doc = state.get_document(uri)?;
        let ast = doc.ast.as_ref()?;

        let mut tokens = Vec::new();
        Self::collect_tokens(&ast.statements, &doc, &mut tokens);

        // 排序并转换为相对增量格式
        tokens.sort_by(|a, b| a.delta_line.cmp(&b.delta_line).then(a.delta_start.cmp(&b.delta_start)));

        let mut last_line = 0;
        let mut last_start = 0;
        let mut data = Vec::new();

        for token in tokens {
            let delta_line = token.delta_line - last_line;
            let delta_start = if delta_line == 0 { token.delta_start - last_start } else { token.delta_start };

            data.push(SemanticToken {
                delta_line,
                delta_start,
                length: token.length,
                token_type: token.token_type,
                token_modifiers_bitset: token.token_modifiers_bitset,
            });

            last_line = token.delta_line;
            last_start = token.delta_start;
        }

        Some(SemanticTokens { result_id: None, data })
    }

    fn collect_tokens(statements: &[valkyrie_ast::StatementKind], doc: &DocumentState, tokens: &mut Vec<SemanticToken>) {
        for stmt in statements {
            match stmt {
                valkyrie_ast::StatementKind::Function(f) => {
                    let pos = doc.offset_to_position(f.name.span().start as usize);
                    tokens.push(SemanticToken {
                        delta_line: pos.line,
                        delta_start: pos.character,
                        length: f.name.to_string().len() as u32,
                        token_type: 3, // function
                        token_modifiers_bitset: 0,
                    });
                    Self::collect_tokens(&f.body.terms, doc, tokens);
                }
                valkyrie_ast::StatementKind::Class(c) => {
                    let pos = doc.offset_to_position(c.name.span.start as usize);
                    tokens.push(SemanticToken {
                        delta_line: pos.line,
                        delta_start: pos.character,
                        length: c.name.to_string().len() as u32,
                        token_type: 0, // class
                        token_modifiers_bitset: 0,
                    });
                    for term in &c.terms {
                        match term {
                            valkyrie_ast::ClassTerm::Field(f) => {
                                let pos = doc.offset_to_position(f.name.span.start as usize);
                                tokens.push(SemanticToken {
                                    delta_line: pos.line,
                                    delta_start: pos.character,
                                    length: f.name.to_string().len() as u32,
                                    token_type: 1, // property
                                    token_modifiers_bitset: 0,
                                });
                            }
                            valkyrie_ast::ClassTerm::Method(m) => {
                                let pos = doc.offset_to_position(m.name.span().start as usize);
                                tokens.push(SemanticToken {
                                    delta_line: pos.line,
                                    delta_start: pos.character,
                                    length: m.name.to_string().len() as u32,
                                    token_type: 3, // function
                                    token_modifiers_bitset: 0,
                                });
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
