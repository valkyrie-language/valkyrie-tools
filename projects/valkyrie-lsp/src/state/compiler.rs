use std::path::{Path, PathBuf};
use std::sync::Arc;
use tower_lsp::lsp_types::*;
use tracing::{debug, error, info};
use valkyrie_ast::ProgramRoot;
use valkyrie_ast::helper::ValkyrieNode;
use valkyrie_compiler::pipeline::ValkyrieCompiler;
use valkyrie_error::{SourceID, ValkyrieError};
use super::{ServerState, DocumentState, GlobalSymbol};
use crate::handlers::utils::range_to_lsp_range;

impl ServerState {
    /// 设置工作区根目录
    pub async fn set_workspace_root(&self, root: String) {
        *self.workspace_root.write() = Some(root.clone());
        if let Ok(url) = url::Url::parse(&root) {
            if let Ok(path) = url.to_file_path() {
                self.legion.write().set_workspace_root(path);
                self.scan_workspace().await;
            }
        }
    }

    /// 扫描工作区中的所有文件
    pub async fn scan_workspace(&self) {
        let packages = self.legion.read().get_all_packages();
        info!("Scanning workspace: found {} packages", packages.len());

        for pkg_dir in &packages {
            self.index_package(pkg_dir).await;
        }
    }

    /// 索引指定包中的所有文件
    pub async fn index_package(&self, pkg_dir: &std::path::Path) {
        let sources = self.legion.read().get_package_sources(pkg_dir);
        for source in sources {
            if let Ok(uri) = url::Url::from_file_path(&source) {
                let uri_str = uri.to_string();
                if !self.documents.contains_key(&uri_str) {
                    if let Ok(text) = std::fs::read_to_string(&source) {
                        if let Err(e) = self.compile_document(&uri_str, &text) {
                            error!("Failed to index {}: {}", uri_str, e);
                        }
                    }
                }
            }
        }
    }

    /// 编译文档
    pub fn compile_document(
        &self,
        uri: &str,
        text: &str,
    ) -> Result<Vec<ValkyrieError>, Box<dyn std::error::Error + Send + Sync>> {
        let new_hash = DocumentState::compute_hash(text);

        // 增量检查：如果 hash 没变，且已经有 AST，则跳过
        if let Some(doc) = self.documents.get(uri) {
            if doc.hash == new_hash && doc.ast.is_some() {
                debug!("Incremental: skipping compilation for {}", uri);
                return Ok(doc.diagnostics.clone());
            }
        }

        debug!("Compiling document: {}", uri);

        // 创建或更新文档状态
        let mut doc_state = DocumentState::new(uri.to_string(), 1, text.to_string());
        doc_state.hash = new_hash;
        let source_id = SourceID::default(); // TODO: 实际的 SourceID 生成
        doc_state.file_id = Some(source_id);

        // 使用 ValkyrieCompiler 进行编译
        let compiler = ValkyrieCompiler::new(text.to_string());

        match compiler.parse() {
            Ok(ast) => {
                doc_state.ast = Some(ast.clone());

                // 获取旧的命名空间用于失效缓存
                let mut _old_namespace = None;
                if let Some(old_doc) = self.documents.get(uri) {
                    if let Some(old_ast) = &old_doc.ast {
                        for stmt in &old_ast.statements {
                            if let valkyrie_ast::StatementKind::Namespace(n) = stmt {
                                _old_namespace = Some(n.path.to_string());
                                break;
                            }
                        }
                    }
                }

                // 尝试生成 HIR
                if let Ok(hir) = compiler.lower_hir(ast.clone()) {
                    doc_state.hir = Some(hir);
                }

                doc_state.diagnostics = Vec::new();

                // 缓存编译结果
                self.documents.insert(uri.to_string(), doc_state);

                // 重新索引并处理细粒度缓存失效
                self.reindex_document(uri);

                // 异步提取依赖关系以填充语义缓存
                let self_clone = self.clone();
                let uri_clone = uri.to_string();
                tokio::spawn(async move {
                    self_clone.extract_dependencies_from_ast(&uri_clone).await;
                });

                Ok(Vec::new())
            }
            Err(e) => {
                doc_state.diagnostics = vec![e.clone()];

                // 即使编译失败也要缓存状态
                self.documents.insert(uri.to_string(), doc_state);

                // 即使失败也尝试重新索引（可能有部分 AST）
                self.reindex_document(uri);

                Ok(vec![e])
            }
        }
    }

    /// 重新索引文档中的所有全局符号
    pub fn reindex_document(&self, uri: &str) {
        let doc = match self.documents.get(uri) {
            Some(d) => d,
            None => return,
        };
        let ast = match &doc.ast {
            Some(a) => a,
            None => return,
        };

        let mut current_namespace = String::new();
        let mut symbols = Vec::new();

        for stmt in &ast.statements {
            match stmt {
                valkyrie_ast::StatementKind::Namespace(n) => {
                    current_namespace = n.path.to_string();
                }
                valkyrie_ast::StatementKind::Function(f) => {
                    symbols.push(GlobalSymbol {
                        name: f.name.to_string(),
                        namespace: current_namespace.clone(),
                        kind: SymbolKind::FUNCTION,
                        uri: uri.to_string(),
                        range: range_to_lsp_range(&f.get_range(), &doc),
                        documentation: None, // TODO: 提取文档注释
                        hash: 0,             // TODO: 计算内容的哈希
                    });
                }
                valkyrie_ast::StatementKind::Class(c) => {
                    symbols.push(GlobalSymbol {
                        name: c.name.to_string(),
                        namespace: current_namespace.clone(),
                        kind: SymbolKind::CLASS,
                        uri: uri.to_string(),
                        range: range_to_lsp_range(&c.get_range(), &doc),
                        documentation: None,
                        hash: 0,
                    });
                }
                valkyrie_ast::StatementKind::Trait(t) => {
                    symbols.push(GlobalSymbol {
                        name: t.name.to_string(),
                        namespace: current_namespace.clone(),
                        kind: SymbolKind::INTERFACE,
                        uri: uri.to_string(),
                        range: range_to_lsp_range(&t.get_range(), &doc),
                        documentation: None,
                        hash: 0,
                    });
                }
                _ => {}
            }
        }

        // 失效旧的语义缓存
        self.semantic_cache.invalidate_namespace(&current_namespace);

        // 更新全局索引
        self.index.update_file_symbols(uri, &current_namespace, symbols);
    }

    /// 从 AST 中提取跨文档的依赖关系
    pub async fn extract_dependencies_from_ast(&self, uri: &str) {
        let ast = match self.documents.get(uri).and_then(|d| d.ast.clone()) {
            Some(a) => a,
            None => return,
        };

        let mut current_namespace = String::new();
        for stmt in &ast.statements {
            if let valkyrie_ast::StatementKind::Namespace(n) = stmt {
                current_namespace = n.path.to_string();
                break;
            }
        }

        // 遍历 AST 寻找外部引用
        // 这通常在 resolve_symbol 时按需填充，但这里可以做一些预热
        for stmt in &ast.statements {
            if let valkyrie_ast::StatementKind::Import(i) = stmt {
                let items = i.flatten();
                for item in items {
                    let ns = item.path.iter().map(|id| id.as_str()).collect::<Vec<_>>().join(".");
                    // 尝试预加载导入的包
                    if let Some(first) = item.path.first() {
                        let pkg_name = first.as_str();
                        let pkg_path = self.legion.read().resolve_package(pkg_name, uri);
                        if let Some(pkg_path) = pkg_path {
                            self.index_package(&pkg_path).await;
                        }
                    }
                    debug!("Found import dependency: {} -> {}", current_namespace, ns);
                }
            }
        }
    }

    /// 获取指定文档的 AST
    pub fn get_ast(&self, uri: &str) -> Option<ProgramRoot> {
        self.documents.get(uri).and_then(|doc| doc.ast.clone())
    }

    /// 获取指定文档的 HIR
    pub fn get_hir(&self, uri: &str) -> Option<valkyrie_types::hir::HirProgram> {
        self.documents.get(uri).and_then(|doc| doc.hir.clone())
    }
}
