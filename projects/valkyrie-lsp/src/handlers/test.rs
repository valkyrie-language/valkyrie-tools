use tower_lsp::lsp_types::*;
use crate::state::ServerState;
use super::utils::span_to_range;

/// 测试处理器
pub struct TestHandler;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TestItem {
    pub id: String,
    pub label: String,
    pub uri: String,
    pub range: Range,
}

impl TestHandler {
    /// 获取文档中的所有测试
    pub async fn handle_get_tests(state: &ServerState, params: DocumentSymbolParams) -> Option<Vec<TestItem>> {
        let uri = params.text_document.uri.to_string();
        let doc = state.get_document(&uri)?;
        let ast = doc.ast.as_ref()?;

        let mut tests = Vec::new();
        for stmt in &ast.statements {
            if let valkyrie_ast::StatementKind::Function(func) = stmt {
                if func.annotations.attributes.get("test").is_some() {
                    tests.push(TestItem {
                        id: format!("{}:{}", uri, func.name),
                        label: func.name.to_string(),
                        uri: uri.clone(),
                        range: span_to_range(func.name.span(), &doc),
                    });
                }
            }
        }

        if tests.is_empty() {
            None
        }
        else {
            Some(tests)
        }
    }
}
