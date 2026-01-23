use oak_lsp::types::{LocationRange, Position};
use crate::state::ServerState;
use crate::errors::LspResult;

pub struct DefinitionHandler;

impl DefinitionHandler {
    pub async fn handle(
        state: &ServerState,
        uri: &str,
        position: Position,
    ) -> LspResult<Option<LocationRange>> {
        if let Some(symbol) = state.query_symbol_at_position(uri, position).await {
            // 目前只返回符号自身的定义位置
            return Ok(Some(symbol.location));
        }

        Ok(None)
    }
}
