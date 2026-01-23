//! Valkyrie LSP Backend Implementation
//!
//! 基于 Nyar 编译器基础设施的 LSP 后端实现

use serde_json::Value;
use std::sync::Arc;
use oak_lsp::{
    service::LanguageService,
    types::*,
    LspServer,
};
use oak_vfs::{MemoryVfs, Vfs, WritableVfs};
use tracing::{debug, error, info};

use crate::{capabilities::server_capabilities, diagnostics::DiagnosticsManager, handlers, state::ServerState};

/// Valkyrie LSP 后端
///
/// 这个结构体实现了 LSP 协议，并使用 Nyar 编译器基础设施
/// 来提供语言服务功能。
pub struct ValkyrieBackend {
    vfs: MemoryVfs,
    workspace: oak_lsp::workspace::WorkspaceManager,
    state: Arc<ServerState>,
    diagnostics: Arc<DiagnosticsManager>,
}

impl ValkyrieBackend {
    pub fn new() -> Self {
        Self {
            vfs: MemoryVfs::new(),
            workspace: oak_lsp::workspace::WorkspaceManager::new(),
            state: Arc::new(ServerState::new()),
            diagnostics: Arc::new(DiagnosticsManager::new()),
        }
    }

    /// 自定义方法：获取 AST
    pub async fn get_ast(&self, params: Value) -> anyhow::Result<Value> {
        let uri = params.get("uri").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing uri parameter"))?;

        debug!("Getting AST for {}", uri);

        match self.state.get_ast(uri) {
            Some(ast) => {
                // 将 AST 序列化为 JSON
                serde_json::to_value(&ast).map_err(|_| anyhow::anyhow!("Serialization error"))
            }
            None => Err(anyhow::anyhow!("AST not found")),
        }
    }

    /// 自定义方法：获取 HIR
    pub async fn get_hir(&self, params: Value) -> anyhow::Result<Value> {
        let uri = params.get("uri").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing uri parameter"))?;

        debug!("Getting HIR for {}", uri);

        match self.state.get_hir(uri) {
            Some(hir) => {
                // 将 HIR 序列化为 JSON
                serde_json::to_value(&hir).map_err(|_| anyhow::anyhow!("Serialization error"))
            }
            None => Ok(Value::Null),
        }
    }

    /// 自定义方法：查询符号
    pub async fn query_symbol(&self, params: Value) -> anyhow::Result<Value> {
        let uri = params.get("uri").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing uri parameter"))?;

        let position = params.get("position").ok_or_else(|| anyhow::anyhow!("Missing position parameter"))?;

        let position: Position =
            serde_json::from_value(position.clone()).map_err(|_| anyhow::anyhow!("Invalid position"))?;

        debug!("Querying symbol at {}:{}:{}", uri, position.line, position.character);

        match self.state.query_symbol_at_position(uri, position).await {
            Some(symbol_info) => serde_json::to_value(&symbol_info).map_err(|_| anyhow::anyhow!("Serialization error")),
            None => Ok(Value::Null),
        }
    }

    /// 自定义方法：获取测试
    pub async fn get_tests(&self, params: Value) -> anyhow::Result<Value> {
        let uri = params.get("uri").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing uri parameter"))?;
        let params = DocumentSymbolParams {
            text_document: TextDocumentIdentifier { uri: Url::parse(uri).map_err(|_| anyhow::anyhow!("Invalid URI"))? },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };

        match handlers::TestHandler::handle_get_tests(&self.state, params).await {
            Some(tests) => serde_json::to_value(tests).map_err(|_| anyhow::anyhow!("Serialization error")),
            None => Ok(Value::Null),
        }
    }

    /// 编译文档并更新诊断信息
    async fn compile_and_diagnose(&self, uri: &str, text: &str) {
        debug!("Compiling document: {}", uri);

        if let Err(e) = self.state.compile_document(uri, text) {
            error!("Failed to compile document {}: {}", uri, e);
        }
    }
}

impl LanguageService for ValkyrieBackend {
    type Lang = oak_valkyrie::ValkyrieLanguage;
    type Vfs = MemoryVfs;

    fn vfs(&self) -> &Self::Vfs {
        &self.vfs
    }

    fn workspace(&self) -> &oak_lsp::workspace::WorkspaceManager {
        &self.workspace
    }

    fn hover(&self, uri: &str, range: core::range::Range<usize>) -> impl std::future::Future<Output = Option<Hover>> + Send + '_ {
        async move {
            let position = self.offset_to_position(uri, range.start).await?;
            handlers::HoverHandler::handle(&self.state, uri, position).await.ok().flatten()
        }
    }
}

impl ValkyrieBackend {
    async fn offset_to_position(&self, uri: &str, offset: usize) -> Option<Position> {
        let source = self.vfs.get_source(uri)?;
        let line_map = oak_core::source::LineMap::from_source(&source);
        let (line, col) = line_map.offset_to_line_col_utf16(&source, offset);
        Some(Position { line, character: col })
    }
}

    async fn selection_range(&self, params: SelectionRangeParams) -> Result<Option<Vec<SelectionRange>>> {
        Ok(handlers::SelectionRangeHandler::handle(&self.state, params).await)
    }

    async fn folding_range(&self, params: FoldingRangeParams) -> Result<Option<Vec<FoldingRange>>> {
        Ok(handlers::FoldingRangeHandler::handle(&self.state, params).await)
    }

    async fn document_highlight(&self, params: DocumentHighlightParams) -> Result<Option<Vec<DocumentHighlight>>> {
        Ok(handlers::DocumentHighlightHandler::handle(&self.state, params).await)
    }

    async fn symbol(&self, params: WorkspaceSymbolParams) -> Result<Option<Vec<SymbolInformation>>> {
        Ok(handlers::WorkspaceSymbolHandler::handle(&self.state, params).await)
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        Ok(handlers::RenameHandler::handle(&self.state, params).await)
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        Ok(handlers::CodeActionHandler::handle(&self.state, params).await)
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        Ok(handlers::FormattingHandler::handle(&self.state, params).await)
    }

    async fn semantic_tokens_full(&self, params: SemanticTokensParams) -> Result<Option<SemanticTokensResult>> {
        Ok(handlers::SemanticTokensHandler::handle_full(&self.state, params).await)
    }

    async fn semantic_tokens_range(&self, params: SemanticTokensRangeParams) -> Result<Option<SemanticTokensRangeResult>> {
        Ok(handlers::SemanticTokensHandler::handle_range(&self.state, params).await)
    }

    async fn inlay_hint(&self, params: InlayHintParams) -> Result<Option<Vec<InlayHint>>> {
        Ok(handlers::InlayHintHandler::handle(&self.state, params).await)
    }
}
