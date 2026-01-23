use dashmap::DashMap;
use parking_lot::RwLock;
use std::sync::Arc;
use tower_lsp::lsp_types::*;

pub mod document;
pub mod symbol;
pub mod cache;
pub mod compiler;
pub mod query;

pub use document::DocumentState;
pub use symbol::{SymbolInfo, GlobalSymbol, GlobalIndex};
pub use cache::SemanticCache;

use crate::legion::LegionManager;

/// LSP 服务器状态
#[derive(Clone)]
pub struct ServerState {
    /// 文档状态缓存
    pub(crate) documents: Arc<DashMap<String, DocumentState>>,

    /// 全局索引
    pub(crate) index: Arc<GlobalIndex>,

    /// 语义缓存
    pub(crate) semantic_cache: Arc<SemanticCache>,

    /// 客户端能力
    pub(crate) client_capabilities: Arc<RwLock<Option<TextDocumentClientCapabilities>>>,

    /// 工作区根目录
    pub(crate) workspace_root: Arc<RwLock<Option<String>>>,

    /// Legion 包管理器
    pub(crate) legion: Arc<RwLock<LegionManager>>,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            documents: Arc::new(DashMap::new()),
            index: Arc::new(GlobalIndex::new()),
            semantic_cache: Arc::new(SemanticCache::new()),
            client_capabilities: Arc::new(RwLock::new(None)),
            workspace_root: Arc::new(RwLock::new(None)),
            legion: Arc::new(RwLock::new(LegionManager::new())),
        }
    }

    /// 获取指定文档状态
    pub fn get_document(&self, uri: &str) -> Option<DocumentState> {
        self.documents.get(uri).map(|doc| doc.clone())
    }

    /// 移除文档状态
    pub async fn remove_document(&self, uri: &str) {
        self.documents.remove(uri);
    }

    /// 设置客户端能力
    pub async fn set_client_capabilities(&self, capabilities: TextDocumentClientCapabilities) {
        *self.client_capabilities.write() = Some(capabilities);
    }

    /// 获取工作区根目录
    pub async fn get_workspace_root(&self) -> Option<String> {
        self.workspace_root.read().clone()
    }

    pub(crate) fn cache_symbol(&self, namespace: &str, name: &str, symbol: Arc<GlobalSymbol>) {
        let info = SymbolInfo {
            name: symbol.name.clone(),
            namespace: symbol.namespace.clone(),
            kind: format!("{:?}", symbol.kind),
            type_info: None,
            documentation: symbol.documentation.clone(),
            location: Location { uri: Url::parse(&symbol.uri).unwrap(), range: symbol.range },
        };
        self.semantic_cache.cache.entry(namespace.to_string()).or_insert_with(DashMap::new).insert(name.to_string(), Arc::new(info));
    }

    pub(crate) fn extract_doc(&self, documents: &valkyrie_ast::DocumentationList) -> Option<String> {
        if documents.is_empty() {
            None
        }
        else {
            let mut s = String::new();
            for term in &documents.terms {
                s.push_str(&term.text);
                s.push('\n');
            }
            Some(s.trim().to_string())
        }
    }
}
