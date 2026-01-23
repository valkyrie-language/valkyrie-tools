use tower_lsp::lsp_types::*;
use crate::state::{ServerState, DocumentState};

/// 语义标记处理器
pub struct SemanticTokensHandler;

/// 内部标记表示
struct InternalToken {
    delta_line: u32,
    delta_start_character: u32,
    length: u32,
    token_type: u32,
    token_modifiers_bitset: u32,
}

impl SemanticTokensHandler {
    pub async fn handle_full(state: &ServerState, params: SemanticTokensParams) -> Option<SemanticTokensResult> {
        let uri = params.text_document.uri.to_string();
        let doc = state.get_document(&uri)?;
        let ast = doc.ast.as_ref()?;

        let mut tokens = Vec::new();
        Self::collect_tokens(&ast.statements, &doc, &mut tokens);

        // 排序并转换为相对增量格式
        tokens.sort_by(|a, b| a.delta_line.cmp(&b.delta_line).then(a.delta_start_character.cmp(&b.delta_start_character)));

        let mut last_line = 0;
        let mut last_start = 0;
        let mut data = Vec::new();

        for token in tokens {
            let delta_line = token.delta_line - last_line;
            let delta_start = if delta_line == 0 { token.delta_start_character - last_start } else { token.delta_start_character };

            data.push(tower_lsp::lsp_types::SemanticToken {
                delta_line,
                delta_start,
                length: token.length,
                token_type: token.token_type,
                token_modifiers_bitset: token.token_modifiers_bitset,
            });

            last_line = token.delta_line;
            last_start = token.delta_start_character;
        }

        Some(SemanticTokensResult::Tokens(SemanticTokens { result_id: None, data }))
    }

    pub async fn handle_range(state: &ServerState, params: SemanticTokensRangeParams) -> Option<SemanticTokensRangeResult> {
        let uri = params.text_document.uri.to_string();
        let doc = state.get_document(&uri)?;
        let ast = doc.ast.as_ref()?;

        let mut tokens = Vec::new();
        Self::collect_tokens(&ast.statements, &doc, &mut tokens);

        // 过滤不在范围内的标记
        let range = params.range;
        let tokens: Vec<_> = tokens.into_iter().filter(|t| {
            t.delta_line >= range.start.line && t.delta_line <= range.end.line
        }).collect();

        // 排序并转换为相对增量格式
        let mut tokens = tokens;
        tokens.sort_by(|a, b| a.delta_line.cmp(&b.delta_line).then(a.delta_start_character.cmp(&b.delta_start_character)));

        let mut last_line = 0;
        let mut last_start = 0;
        let mut data = Vec::new();

        for token in tokens {
            let delta_line = token.delta_line - last_line;
            let delta_start = if delta_line == 0 { token.delta_start_character - last_start } else { token.delta_start_character };

            data.push(tower_lsp::lsp_types::SemanticToken {
                delta_line,
                delta_start,
                length: token.length,
                token_type: token.token_type,
                token_modifiers_bitset: token.token_modifiers_bitset,
            });

            last_line = token.delta_line;
            last_start = token.delta_start_character;
        }

        Some(SemanticTokensRangeResult::Tokens(SemanticTokens { result_id: None, data }))
    }

    fn collect_tokens(statements: &[valkyrie_ast::StatementKind], doc: &DocumentState, tokens: &mut Vec<InternalToken>) {
        for stmt in statements {
            match stmt {
                valkyrie_ast::StatementKind::Function(f) => {
                    let pos = doc.offset_to_position(f.name.span().start as usize);
                    tokens.push(InternalToken {
                        delta_line: pos.line,
                        delta_start_character: pos.character,
                        length: f.name.to_string().len() as u32,
                        token_type: 3, // function
                        token_modifiers_bitset: 0,
                    });
                    Self::collect_tokens(&f.body.terms, doc, tokens);
                }
                valkyrie_ast::StatementKind::Class(c) => {
                    let pos = doc.offset_to_position(c.name.span.start as usize);
                    tokens.push(InternalToken {
                        delta_line: pos.line,
                        delta_start_character: pos.character,
                        length: c.name.to_string().len() as u32,
                        token_type: 0, // class
                        token_modifiers_bitset: 0,
                    });
                    for term in &c.terms {
                        match term {
                            valkyrie_ast::ClassTerm::Field(f) => {
                                let pos = doc.offset_to_position(f.name.span.start as usize);
                                tokens.push(InternalToken {
                                    delta_line: pos.line,
                                    delta_start_character: pos.character,
                                    length: f.name.to_string().len() as u32,
                                    token_type: 1, // property
                                    token_modifiers_bitset: 0,
                                });
                            }
                            valkyrie_ast::ClassTerm::Method(m) => {
                                let pos = doc.offset_to_position(m.name.span().start as usize);
                                tokens.push(InternalToken {
                                    delta_line: pos.line,
                                    delta_start_character: pos.character,
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
