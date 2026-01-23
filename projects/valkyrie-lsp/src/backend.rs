//! Valkyrie LSP Backend Implementation
//!
//! 基于 Nyar 编译器基础设施的 LSP 后端实现

use serde_json::Value;
use std::sync::Arc;
use tower_lsp::{
    jsonrpc::{Error, Result},
    lsp_types::*,
    Client, LanguageServer,
};
use tracing::{debug, error, info};

use crate::{capabilities::server_capabilities, diagnostics::DiagnosticsManager, handlers, state::ServerState};

/// Valkyrie LSP 后端
///
/// 这个结构体实现了 LSP 协议，并使用 Nyar 编译器基础设施
/// 来提供语言服务功能。
pub struct ValkyrieBackend {
    client: Client,
    state: Arc<ServerState>,
    diagnostics: Arc<DiagnosticsManager>,
}

impl ValkyrieBackend {
    pub fn new(client: Client) -> Self {
        Self { client, state: Arc::new(ServerState::new()), diagnostics: Arc::new(DiagnosticsManager::new()) }
    }

    /// 自定义方法：获取 AST
    pub async fn get_ast(&self, params: Value) -> Result<Value> {
        let uri = params.get("uri").and_then(|v| v.as_str()).ok_or_else(|| Error::invalid_params("Missing uri parameter"))?;

        debug!("Getting AST for {}", uri);

        match self.state.get_ast(uri) {
            Some(ast) => {
                // 将 AST 序列化为 JSON
                serde_json::to_value(&ast).map_err(|_| Error::internal_error())
            }
            None => Err(Error::invalid_request()),
        }
    }

    /// 自定义方法：获取 HIR
    pub async fn get_hir(&self, params: Value) -> Result<Value> {
        let uri = params.get("uri").and_then(|v| v.as_str()).ok_or_else(|| Error::invalid_params("Missing uri parameter"))?;

        debug!("Getting HIR for {}", uri);

        match self.state.get_hir(uri) {
            Some(hir) => {
                // 将 HIR 序列化为 JSON
                serde_json::to_value(&hir).map_err(|_| Error::internal_error())
            }
            None => Ok(Value::Null),
        }
    }

    /// 自定义方法：查询符号
    pub async fn query_symbol(&self, params: Value) -> Result<Value> {
        let uri = params.get("uri").and_then(|v| v.as_str()).ok_or_else(|| Error::invalid_params("Missing uri parameter"))?;

        let position = params.get("position").ok_or_else(|| Error::invalid_params("Missing position parameter"))?;

        let position: Position =
            serde_json::from_value(position.clone()).map_err(|_| Error::invalid_params("Invalid position"))?;

        debug!("Querying symbol at {}:{}:{}", uri, position.line, position.character);

        match self.state.query_symbol_at_position(uri, position).await {
            Some(symbol_info) => serde_json::to_value(&symbol_info).map_err(|_| Error::internal_error()),
            None => Ok(Value::Null),
        }
    }

    /// 自定义方法：获取测试
    pub async fn get_tests(&self, params: Value) -> Result<Value> {
        let uri = params.get("uri").and_then(|v| v.as_str()).ok_or_else(|| Error::invalid_params("Missing uri parameter"))?;
        let params = DocumentSymbolParams {
            text_document: TextDocumentIdentifier { uri: Url::parse(uri).map_err(|_| Error::invalid_params("Invalid URI"))? },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };

        match handlers::TestHandler::handle_get_tests(&self.state, params).await {
            Some(tests) => serde_json::to_value(tests).map_err(|_| Error::internal_error()),
            None => Ok(Value::Null),
        }
    }

    /// 编译文档并更新诊断信息
    async fn compile_and_diagnose(&self, uri: &str, text: &str) {
        debug!("Compiling document: {}", uri);

        match self.state.compile_document(uri, text) {
            Ok(diagnostics) => {
                // 转换诊断信息为 LSP 格式
                let lsp_diagnostics = self.diagnostics.convert_to_lsp_diagnostics(&diagnostics, text);

                // 发送诊断信息到客户端
                if let Ok(url) = Url::parse(uri) {
                    self.client.publish_diagnostics(url, lsp_diagnostics, None).await;
                }
            }
            Err(e) => {
                error!("Failed to compile document {}: {}", uri, e);
            }
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for ValkyrieBackend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        info!("Initializing Valkyrie LSP server");

        if let Some(capabilities) = params.capabilities.text_document {
            self.state.set_client_capabilities(capabilities).await;
        }

        if let Some(folders) = params.workspace_folders {
            if let Some(first) = folders.first() {
                self.state.set_workspace_root(first.uri.to_string()).await;
            }
        }
        else if let Some(root_uri) = params.root_uri {
            self.state.set_workspace_root(root_uri.to_string()).await;
        }

        Ok(InitializeResult {
            capabilities: server_capabilities(),
            server_info: Some(ServerInfo { name: "Valkyrie Language Server".to_string(), version: Some("0.1.0".to_string()) }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        info!("Valkyrie LSP server initialized");
        self.state.scan_workspace().await;
    }

    async fn shutdown(&self) -> Result<()> {
        info!("Shutting down Valkyrie LSP server");
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        info!("Document opened: {}", params.text_document.uri);
        self.compile_and_diagnose(params.text_document.uri.as_str(), &params.text_document.text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        info!("Document changed: {}", params.text_document.uri);
        if let Some(change) = params.content_changes.first() {
            self.compile_and_diagnose(params.text_document.uri.as_str(), &change.text).await;
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        info!("Document saved: {}", params.text_document.uri);
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        info!("Document closed: {}", params.text_document.uri);
        self.state.remove_document(params.text_document.uri.as_str()).await;
    }

    async fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {
        info!("Watched files changed: {:?}", params.changes);
        let mut reindex_needed = false;
        for change in params.changes {
            if change.uri.as_str().ends_with("legion.json") || change.uri.as_str().ends_with("legions.json") {
                reindex_needed = true;
                break;
            }
        }

        if reindex_needed {
            info!("Legion configuration changed, re-scanning workspace...");
            if let Some(root) = self.state.get_workspace_root().await {
                self.state.set_workspace_root(root).await;
            }
        }
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        info!("Hover requested: {:?}", params);
        handlers::HoverHandler::handle(&self.state, params).await.map_err(|e| {
            error!("Hover failed: {}", e);
            tower_lsp::jsonrpc::Error::internal_error()
        })
    }

    async fn goto_definition(&self, params: GotoDefinitionParams) -> Result<Option<GotoDefinitionResponse>> {
        info!("Definition requested: {:?}", params);
        handlers::DefinitionHandler::handle(&self.state, params).await.map_err(|e| {
            error!("Definition failed: {}", e);
            tower_lsp::jsonrpc::Error::internal_error()
        })
    }

    async fn document_symbol(&self, params: DocumentSymbolParams) -> Result<Option<DocumentSymbolResponse>> {
        Ok(crate::handlers::DocumentSymbolHandler::handle(&self.state, params).await)
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        info!("Completion requested: {:?}", params);
        handlers::CompletionHandler::handle(&self.state, params).await.map_err(|e| {
            error!("Completion failed: {}", e);
            tower_lsp::jsonrpc::Error::internal_error()
        })
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
