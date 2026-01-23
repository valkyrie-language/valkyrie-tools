use tower_lsp::lsp_types::*;
use crate::state::ServerState;

/// 代码操作处理器
pub struct CodeActionHandler;

impl CodeActionHandler {
    pub async fn handle(_state: &ServerState, params: CodeActionParams) -> Option<CodeActionResponse> {
        let mut actions = Vec::new();

        // 基础代码操作示例：整理导入
        if let Some(kind) = &params.context.only {
            if kind.contains(&CodeActionKind::SOURCE_ORGANIZE_IMPORTS) {
                actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                    title: "Organize Imports".to_string(),
                    kind: Some(CodeActionKind::SOURCE_ORGANIZE_IMPORTS),
                    edit: Some(WorkspaceEdit { changes: None, document_changes: None, change_annotations: None }),
                    ..Default::default()
                }));
            }
        }

        if actions.is_empty() {
            None
        }
        else {
            Some(actions)
        }
    }
}
