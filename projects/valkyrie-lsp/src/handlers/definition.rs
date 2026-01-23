use oak_lsp::types::{LocationRange, Position};
use crate::state::ServerState;

pub struct DefinitionHandler;

impl DefinitionHandler {
    pub async fn handle(
        state: &ServerState,
        uri: &str,
        position: Position,
    ) -> anyhow::Result<Option<LocationRange>> {
        if let Some(symbol) = state.query_symbol_at_position(uri, position).await {
            // 目前只返回符号自身的定义位置
            return Ok(Some(symbol.location));
        }

        Ok(None)
    }
}
