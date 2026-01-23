use dashmap::DashMap;
use std::sync::Arc;
use oak_lsp::types::{LocationRange, SymbolKind, Range};

/// 符号查询结果
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SymbolInfo {
    pub name: String,
    pub namespace: String, // 所属命名空间
    pub kind: String,
    pub type_info: Option<String>,
    pub documentation: Option<String>,
    pub location: LocationRange,
}

/// 全局符号信息
#[derive(Debug, Clone)]
pub struct GlobalSymbol {
    pub name: String,
    pub namespace: String,
    pub kind: SymbolKind,
    pub uri: String,
    pub range: Range<usize>,
    pub documentation: Option<String>,
    /// 符号定义的哈希，用于细粒度失效检查
    pub hash: u64,
}

/// 全局符号索引
pub struct GlobalIndex {
    /// 命名空间 -> 符号名 -> 符号信息
    /// 键为 "std.math" 这种格式
    pub symbols: DashMap<String, DashMap<String, Arc<GlobalSymbol>>>,
    /// 文件 URI -> 该文件定义的符号列表
    pub file_symbols: DashMap<String, Vec<Arc<GlobalSymbol>>>,
}

impl GlobalIndex {
    pub fn new() -> Self {
        Self { symbols: DashMap::new(), file_symbols: DashMap::new() }
    }

    pub fn update_file_symbols(&self, uri: &str, namespace: &str, symbols: Vec<GlobalSymbol>) {
        // 先清理旧的
        if let Some((_, old_symbols)) = self.file_symbols.remove(uri) {
            for symbol in old_symbols {
                if let Some(ns_map) = self.symbols.get(&symbol.namespace) {
                    ns_map.remove(&symbol.name);
                }
            }
        }

        // 添加新的
        let mut arc_symbols = Vec::with_capacity(symbols.len());
        let ns_map = self.symbols.entry(namespace.to_string()).or_insert_with(DashMap::new);

        for symbol in symbols {
            let arc_symbol = Arc::new(symbol);
            ns_map.insert(arc_symbol.name.clone(), arc_symbol.clone());
            arc_symbols.push(arc_symbol);
        }

        self.file_symbols.insert(uri.to_string(), arc_symbols);
    }
}
