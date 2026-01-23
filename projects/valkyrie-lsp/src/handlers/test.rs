use oak_lsp::types::*;
use crate::state::ServerState;
use super::utils::span_to_range_usize;

/// 测试处理器
pub struct TestHandler;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TestItem {
    pub id: String,
    pub label: String,
    pub uri: String,
    pub range: Range<usize>,
}

impl TestHandler {
    /// 获取文档中的所有测试
    pub async fn handle_get_tests(state: &ServerState, uri: &str) -> Option<Vec<TestItem>> {
        let doc = state.get_document(uri)?;
        let ast = doc.ast.as_ref()?;

        let mut tests = Vec::new();
        // 这里只是示例，具体的 AST 结构需要根据实际情况调整
        // 假设有一个方法可以遍历 AST 并找到测试函数
        // 暂时返回空列表，避免编译错误，因为我们没有 valkyrie_ast 的详细信息
        /*
        for stmt in &ast.statements {
            if let valkyrie_ast::StatementKind::Function(func) = stmt {
                if func.annotations.attributes.get("test").is_some() {
                    tests.push(TestItem {
                        id: format!("{}:{}", uri, func.name),
                        label: func.name.to_string(),
                        uri: uri.to_string(),
                        range: span_to_range_usize(func.name.span()),
                    });
                }
            }
        }
        */

        if tests.is_empty() {
            None
        }
        else {
            Some(tests)
        }
    }
}
