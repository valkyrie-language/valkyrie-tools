use tower_lsp::lsp_types::*;
use tracing::debug;
use crate::state::ServerState;

/// 工作区符号处理器
pub struct WorkspaceSymbolHandler;

impl WorkspaceSymbolHandler {
    /// 处理工作区符号查询
    pub async fn handle(state: &ServerState, params: WorkspaceSymbolParams) -> Option<Vec<SymbolInformation>> {
        let query = params.query.to_lowercase();
        debug!("Workspace symbol query: {}", query);

        let mut result = Vec::new();

        // 遍历所有索引的符号
        for ns_map_ref in state.index.symbols.iter() {
            for symbol_ref in ns_map_ref.value().iter() {
                let symbol = symbol_ref.value();
                if query.is_empty() || symbol.name.to_lowercase().contains(&query) {
                    result.push(SymbolInformation {
                        name: symbol.name.clone(),
                        kind: symbol.kind,
                        tags: None,
                        deprecated: None,
                        location: Location { uri: Url::parse(&symbol.uri).unwrap(), range: symbol.range },
                        container_name: Some(ns_map_ref.key().clone()),
                    });
                }
            }
        }

        if result.is_empty() {
            None
        }
        else {
            Some(result)
        }
    }
}
