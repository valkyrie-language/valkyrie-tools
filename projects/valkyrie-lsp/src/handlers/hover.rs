use oak_lsp::types::{Hover, Position};
use crate::state::ServerState;

pub struct HoverHandler;

impl HoverHandler {
    pub async fn handle(
        state: &ServerState,
        uri: &str,
        position: Position,
    ) -> anyhow::Result<Option<Hover>> {
        if let Some(symbol) = state.query_symbol_at_position(uri, position).await {
            let mut markdown = String::new();

            // 添加名称和类型
            markdown.push_str(&format!("### {}\n", symbol.name));
            if let Some(type_info) = symbol.type_info {
                markdown.push_str(&format!("`{}`\n\n", type_info));
            }

            // 添加文档信息
            if let Some(doc) = symbol.documentation {
                markdown.push_str("---\n");
                markdown.push_str(&doc);
            }

            return Ok(Some(Hover {
                contents: markdown,
                range: Some(symbol.location.range),
            }));
        }

        Ok(None)
    }
}
