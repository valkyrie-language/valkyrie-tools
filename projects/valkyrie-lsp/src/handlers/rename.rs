use tower_lsp::lsp_types::*;
use crate::state::ServerState;
use std::collections::HashMap;

/// 重命名处理器
pub struct RenameHandler;

impl RenameHandler {
    pub async fn handle(state: &ServerState, params: RenameParams) -> Option<WorkspaceEdit> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let new_name = params.new_name;

        // 1. 获取所有需要重命名的位置
        let refs_params = ReferenceParams {
            text_document_position: TextDocumentPositionParams { text_document: TextDocumentIdentifier { uri: uri.clone() }, position },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
            context: ReferenceContext { include_declaration: true },
        };

        let locations = super::ReferencesHandler::handle(state, refs_params).await?;

        // 2. 按 URI 分组
        let mut changes = HashMap::new();
        for loc in locations {
            let edits = changes.entry(loc.uri).or_insert_with(Vec::new);
            edits.push(TextEdit { range: loc.range, new_text: new_name.clone() });
        }

        Some(WorkspaceEdit { changes: Some(changes), document_changes: None, change_annotations: None })
    }
}
