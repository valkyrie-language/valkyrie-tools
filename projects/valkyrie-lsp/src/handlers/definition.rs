use tower_lsp::lsp_types::*;
use crate::state::ServerState;

pub struct DefinitionHandler;

impl DefinitionHandler {
    pub async fn handle(
        state: &ServerState,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>, Box<dyn std::error::Error + Send + Sync>> {
        let uri = params.text_document_position_params.text_document.uri.to_string();
        let position = params.text_document_position_params.position;

        if let Some(symbol) = state.query_symbol_at_position(&uri, position).await {
            // 目前只返回符号自身的定义位置
            return Ok(Some(GotoDefinitionResponse::Scalar(symbol.location)));
        }

        Ok(None)
    }
}
