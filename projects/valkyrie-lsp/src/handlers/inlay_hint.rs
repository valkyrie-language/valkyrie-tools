use oak_lsp::types::*;
use valkyrie_ast::helper::ValkyrieNode;
use crate::state::{ServerState, DocumentState};

/// 内联提示处理器
pub struct InlayHintHandler;

impl InlayHintHandler {
    pub async fn handle(state: &ServerState, uri: &str) -> Vec<InlayHint> {
        let doc = match state.get_document(uri) {
            Some(d) => d,
            None => return vec![],
        };
        let ast = match doc.ast.as_ref() {
            Some(a) => a,
            None => return vec![],
        };

        let mut hints = Vec::new();
        Self::collect_inlay_hints(&ast.statements, &doc, state, uri, &mut hints).await;

        hints
    }

    #[async_recursion::async_recursion]
    async fn collect_inlay_hints(
        statements: &[valkyrie_ast::StatementKind],
        doc: &DocumentState,
        state: &ServerState,
        uri: &str,
        hints: &mut Vec<InlayHint>,
    ) {
        for stmt in statements {
            match stmt {
                valkyrie_ast::StatementKind::Variable(var) => {
                    // 为变量添加类型提示
                    if var.type_hint.is_none() {
                        let range = var.pattern.get_range();
                        let position = doc.offset_to_position(range.start as usize);

                        // 尝试从状态中查询真实的类型信息
                        let label = if let Some(info) = state.query_symbol_at_position(uri, position).await {
                            if let Some(ty) = info.type_info {
                                format!(": {}", ty.replace("var ", "").replace("function ", ""))
                            }
                            else {
                                ": Unknown".to_string()
                            }
                        }
                        else {
                            ": Unknown".to_string()
                        };

                        let position_end = doc.offset_to_position(range.end as usize);
                        hints.push(InlayHint {
                            position: position_end,
                            label: label,
                            kind: Some(InlayHintKind::Type),
                            text_edits: None,
                            tooltip: Some("Inferred type".to_string()),
                            padding_left: Some(true),
                            padding_right: None,
                            data: None,
                        });
                    }
                    if let Some(body) = &var.body {
                        Self::collect_expression_hints(body, doc, state, uri, hints).await;
                    }
                }
                valkyrie_ast::StatementKind::Function(func) => {
                    // 检查参数类型提示
                    for param in func.parameters.terms() {
                        if param.bound.is_none() {
                            let range = param.key.span;
                            let position = doc.offset_to_position(range.start as usize);

                            let label = if let Some(info) = state.query_symbol_at_position(uri, position).await {
                                if let Some(ty) = info.type_info {
                                    format!(": {}", ty)
                                }
                                else {
                                    ": Any".to_string()
                                }
                            }
                            else {
                                ": Any".to_string()
                            };

                            let position_end = doc.offset_to_position(range.end as usize);
                            hints.push(InlayHint {
                                position: position_end,
                                label: label,
                                kind: Some(InlayHintKind::Type),
                                text_edits: None,
                                tooltip: Some("Implicit type".to_string()),
                                padding_left: Some(true),
                                padding_right: None,
                                data: None,
                            });
                        }
                    }
                    // 检查返回值提示
                    if func.returns.typing.is_none() {
                        let position = doc.offset_to_position(func.name.span().end as usize);
                        hints.push(InlayHint {
                            position,
                            label: " -> Any".to_string(),
                            kind: Some(InlayHintKind::Type),
                            text_edits: None,
                            tooltip: Some("Inferred return type".to_string()),
                            padding_left: Some(true),
                            padding_right: None,
                            data: None,
                        });
                    }
                    Self::collect_inlay_hints(&func.body.terms, doc, state, uri, hints).await;
                }
                valkyrie_ast::StatementKind::Class(cls) => {
                    for term in &cls.terms {
                        match term {
                            valkyrie_ast::ClassTerm::Method(m) => {
                                if let Some(body) = &m.body {
                                    Self::collect_inlay_hints(&body.terms, doc, state, uri, hints).await;
                                }
                            }
                            valkyrie_ast::ClassTerm::Field(f) => {
                                if f.typing.is_none() {
                                    let position = doc.offset_to_position(f.name.span.end as usize);
                                    hints.push(InlayHint {
                                        position,
                                        label: ": Any".to_string(),
                                        kind: Some(InlayHintKind::Type),
                                        text_edits: None,
                                        tooltip: Some("Field type".to_string()),
                                        padding_left: Some(true),
                                        padding_right: None,
                                        data: None,
                                    });
                                }
                            }
                            _ => {}
                        }
                    }
                }
                valkyrie_ast::StatementKind::Expression(expr) => {
                    Self::collect_expression_hints(expr, doc, state, uri, hints).await;
                }
                valkyrie_ast::StatementKind::Each(each) => {
                    // 为 each 循环的迭代变量添加类型提示
                    let position = doc.offset_to_position(each.pattern.get_range().end as usize);
                    hints.push(InlayHint {
                        position,
                        label: ": Element".to_string(),
                        kind: Some(InlayHintKind::Type),
                        text_edits: None,
                        tooltip: Some("Iterated element type".to_string()),
                        padding_left: Some(true),
                        padding_right: None,
                        data: None,
                    });
                    Self::collect_expression_hints(&each.iterator, doc, state, uri, hints).await;
                    Self::collect_inlay_hints(&each.body.terms, doc, state, uri, hints).await;
                }
                _ => {}
            }
        }
    }

    #[async_recursion::async_recursion]
    async fn collect_expression_hints(
        expr: &valkyrie_ast::ExpressionKind,
        doc: &DocumentState,
        state: &ServerState,
        uri: &str,
        hints: &mut Vec<InlayHint>,
    ) {
        match expr {
            valkyrie_ast::ExpressionKind::ApplyCall(call) => {
                // 为函数调用添加参数名提示
                // 尝试获取被调用函数的签名信息
                let caller_pos = doc.offset_to_position(call.caller.get_range().start as usize);
                let mut param_names = Vec::new();

                if let Some(info) = state.query_symbol_at_position(uri, caller_pos).await {
                    if let Some(ty) = info.type_info {
                        // 简单解析类型字符串中的参数名，例如 "function(a: Int, b: String)"
                        if let Some(start) = ty.find('(') {
                            if let Some(end) = ty.rfind(')') {
                                let params_str = &ty[start + 1..end];
                                for p in params_str.split(',') {
                                    let name = p.split(':').next().unwrap_or("").trim();
                                    if !name.is_empty() {
                                        param_names.push(name.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
                
                // 添加参数名提示
                for (i, arg) in call.arguments.terms.iter().enumerate() {
                    if i < param_names.len() {
                        let position = doc.offset_to_position(arg.value.get_range().start as usize);
                        hints.push(InlayHint {
                            position,
                            label: format!("{}:", param_names[i]),
                            kind: Some(InlayHintKind::Parameter),
                            text_edits: None,
                            tooltip: Some("Parameter name".to_string()),
                            padding_left: None,
                            padding_right: Some(true),
                            data: None,
                        });
                    }
                    Self::collect_expression_hints(&arg.value, doc, state, uri, hints).await;
                }
                Self::collect_expression_hints(&call.caller, doc, state, uri, hints).await;
                if let Some(body) = &call.body {
                    Self::collect_inlay_hints(&body.terms, doc, state, uri, hints).await;
                }
            }
            valkyrie_ast::ExpressionKind::Infix(infix) => {
                Self::collect_expression_hints(&infix.lhs, doc, state, uri, hints).await;
                Self::collect_expression_hints(&infix.rhs, doc, state, uri, hints).await;
            }
            _ => {}
        }
    }
}
