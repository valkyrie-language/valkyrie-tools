use tower_lsp::lsp_types::*;
use valkyrie_ast::helper::ValkyrieNode;
use crate::state::ServerState;

/// 签名帮助处理器
pub struct SignatureHelpHandler;

impl SignatureHelpHandler {
    pub async fn handle(state: &ServerState, params: SignatureHelpParams) -> Option<SignatureHelp> {
        let uri = params.text_document_position_params.text_document.uri.to_string();
        let position = params.text_document_position_params.position;
        let doc = state.get_document(&uri)?;
        let offset = doc.position_to_offset(position) as u32;
        let ast = doc.ast.as_ref()?;

        // 找到当前位置的函数调用
        let mut call_node = None;
        Self::find_call_at(&ast.statements, offset, &mut call_node);

        if let Some(call) = call_node {
            let caller_name = match &call.caller {
                valkyrie_ast::ExpressionKind::Symbol(s) => s.to_string(),
                _ => "function".to_string(),
            };

            // 尝试从索引中获取真实的参数名称
            let caller_pos = doc.offset_to_position(call.caller.get_range().start as usize);
            let mut param_names = Vec::new();
            if let Some(info) = state.query_symbol_at_position(&uri, caller_pos).await {
                if let Some(ty) = info.type_info {
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

            let mut parameters = Vec::new();
            for (i, _) in call.arguments.terms.iter().enumerate() {
                let name = if i < param_names.len() { param_names[i].clone() } else { format!("arg{}", i) };
                parameters.push(ParameterInformation { label: ParameterLabel::Simple(name), documentation: None });
            }

            let active_parameter = call.arguments.terms.iter().position(|t| t.span.get_range().contains(&offset)).map(|p| p as u32);

            Some(SignatureHelp {
                signatures: vec![SignatureInformation {
                    label: format!(
                        "{}({})",
                        caller_name,
                        parameters
                            .iter()
                            .map(|p| {
                                if let ParameterLabel::Simple(s) = &p.label {
                                    s.as_str()
                                }
                                else {
                                    ""
                                }
                            })
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                    documentation: Some(Documentation::String(format!("Signature help for {}", caller_name))),
                    parameters: Some(parameters),
                    active_parameter,
                }],
                active_signature: Some(0),
                active_parameter: active_parameter.or(Some(0)),
            })
        }
        else {
            None
        }
    }

    fn find_call_at<'a>(statements: &'a [valkyrie_ast::StatementKind], offset: u32, found: &mut Option<&'a valkyrie_ast::ApplyCallNode>) {
        for stmt in statements {
            match stmt {
                valkyrie_ast::StatementKind::Function(func) => {
                    Self::find_call_at(&func.body.terms, offset, found);
                }
                valkyrie_ast::StatementKind::Expression(expr) => {
                    Self::find_call_in_expr(&expr.body, offset, found);
                }
                valkyrie_ast::StatementKind::Variable(var) => {
                    if let Some(body) = &var.body {
                        Self::find_call_in_expr(body, offset, found);
                    }
                }
                _ => {}
            }
            if found.is_some() {
                return;
            }
        }
    }

    fn find_call_in_expr<'a>(expr: &'a valkyrie_ast::ExpressionKind, offset: u32, found: &mut Option<&'a valkyrie_ast::ApplyCallNode>) {
        match expr {
            valkyrie_ast::ExpressionKind::ApplyCall(call) => {
                if call.get_range().contains(&offset) {
                    *found = Some(call);
                }
                for term in &call.arguments.terms {
                    Self::find_call_in_expr(&term.value, offset, found);
                }
                Self::find_call_in_expr(&call.caller, offset, found);
            }
            valkyrie_ast::ExpressionKind::Infix(infix) => {
                Self::find_call_in_expr(&infix.lhs, offset, found);
                Self::find_call_in_expr(&infix.rhs, offset, found);
            }
            _ => {}
        }
    }
}
