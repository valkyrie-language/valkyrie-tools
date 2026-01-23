use dashmap::DashMap;
use std::sync::Arc;
use oak_lsp::types::{Position, LocationRange};
use valkyrie_ast::ExpressionKind;
use valkyrie_ast::helper::ValkyrieNode;
use super::{ServerState, DocumentState, GlobalSymbol, SymbolInfo};
use crate::handlers::utils::range_to_lsp_range_usize;

impl ServerState {
    /// 查找符号定义，并记录依赖关系
    pub async fn resolve_symbol(
        &self,
        name: &str,
        uri: &str,
        caller_ns: &str,
        caller_name: Option<&str>,
    ) -> Option<Arc<GlobalSymbol>> {
        let ast = self.documents.get(uri).and_then(|d| d.ast.clone())?;
        let mut current_namespace = String::new();

        // 1. 获取当前命名空间
        for stmt in &ast.statements {
            if let valkyrie_ast::StatementKind::Namespace(n) = stmt {
                current_namespace = n.path.to_string();
                break;
            }
        }

        // 2. 检查语义缓存
        if let Some(ns_cache) = self.semantic_cache.cache.get(&current_namespace) {
            if let Some(symbol_info) = ns_cache.get(name) {
                // 缓存命中：记录依赖关系
                if let Some(c_name) = caller_name {
                    self.semantic_cache.update_dependencies(&symbol_info.namespace, &symbol_info.name, caller_ns, c_name);
                }
                // 从全局索引中返回完整的 GlobalSymbol
                if let Some(ns_map) = self.index.symbols.get(&symbol_info.namespace) {
                    if let Some(symbol) = ns_map.get(&symbol_info.name) {
                        return Some(symbol.clone());
                    }
                }
            }
        }

        // 3. 检查当前命名空间
        if let Some(ns_map) = self.index.symbols.get(&current_namespace) {
            if let Some(symbol) = ns_map.get(name) {
                if let Some(c_name) = caller_name {
                    self.semantic_cache.update_dependencies(&current_namespace, name, caller_ns, c_name);
                }
                // 存入语义缓存
                self.cache_symbol(&current_namespace, name, symbol.clone());
                return Some(symbol.clone());
            }
        }

        // 4. 检查导入
        for stmt in &ast.statements {
            if let valkyrie_ast::StatementKind::Import(i) = stmt {
                let items = i.flatten();
                for item in items {
                    use valkyrie_ast::ImportResolvedKind;
                    match item.kind {
                        ImportResolvedKind::Alias { item: import_item, name: alias } => {
                            let import_name = match import_item {
                                valkyrie_ast::ImportAliasItem::Normal(n) => n.name.to_string(),
                                _ => continue,
                            };
                            let final_name = alias
                                .map(|a| match a {
                                    valkyrie_ast::ImportAliasItem::Normal(n) => n.name.to_string(),
                                    _ => import_name.clone(),
                                })
                                .unwrap_or(import_name.clone());

                            if final_name == name {
                                // 找到匹配的导入项，在全局索引中查找
                                let ns = item.path.iter().map(|id| id.as_str()).collect::<Vec<_>>().join(".");
                                if let Some(ns_map) = self.index.symbols.get(&ns) {
                                    if let Some(symbol) = ns_map.get(&import_name) {
                                        if let Some(c_name) = caller_name {
                                            self.semantic_cache.update_dependencies(&ns, &import_name, caller_ns, c_name);
                                        }
                                        // 存入语义缓存
                                        self.cache_symbol(&current_namespace, name, symbol.clone());
                                        return Some(symbol.clone());
                                    }
                                }
                                else {
                                    // 尝试作为包解析
                                    if let Some(first) = item.path.first() {
                                        let pkg_name = first.as_str();
                                        let pkg_path = self.legion.read().resolve_package(pkg_name, uri);
                                        if let Some(pkg_path) = pkg_path {
                                            self.index_package(&pkg_path).await;
                                            if let Some(ns_map) = self.index.symbols.get(&ns) {
                                                if let Some(symbol) = ns_map.get(&import_name) {
                                                    if let Some(c_name) = caller_name {
                                                        self.semantic_cache.update_dependencies(&ns, &import_name, caller_ns, c_name);
                                                    }
                                                    // 存入语义缓存
                                                    self.cache_symbol(&current_namespace, name, symbol.clone());
                                                    return Some(symbol.clone());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        ImportResolvedKind::All => {
                            // 通配符导入：在目标命名空间查找
                            let ns = item.path.iter().map(|id| id.as_str()).collect::<Vec<_>>().join(".");
                            if let Some(ns_map) = self.index.symbols.get(&ns) {
                                if let Some(symbol) = ns_map.get(name) {
                                    if let Some(c_name) = caller_name {
                                        self.semantic_cache.update_dependencies(&ns, name, caller_ns, c_name);
                                    }
                                    // 存入语义缓存
                                    self.cache_symbol(&current_namespace, name, symbol.clone());
                                    return Some(symbol.clone());
                                }
                            }
                            else {
                                // 如果命名空间未找到，尝试作为包解析并按需索引
                                if let Some(first) = item.path.first() {
                                    let pkg_name = first.as_str();
                                    let pkg_path = self.legion.read().resolve_package(pkg_name, uri);
                                    if let Some(pkg_path) = pkg_path {
                                        self.index_package(&pkg_path).await;
                                        if let Some(ns_map) = self.index.symbols.get(&ns) {
                                            if let Some(symbol) = ns_map.get(name) {
                                                if let Some(c_name) = caller_name {
                                                    self.semantic_cache.update_dependencies(&ns, name, caller_ns, c_name);
                                                }
                                                // 存入语义缓存
                                                self.cache_symbol(&current_namespace, name, symbol.clone());
                                                return Some(symbol.clone());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        ImportResolvedKind::Empty | ImportResolvedKind::This | ImportResolvedKind::Ffi(_) => continue,
                    }
                }
            }
        }

        None
    }

    /// 查询指定位置的符号信息
    pub async fn query_symbol_at_position(&self, uri: &str, position: Position) -> Option<SymbolInfo> {
        let (doc, ast) = {
            let doc = self.documents.get(uri)?;
            let ast = doc.ast.clone()?;
            (doc.clone(), ast)
        };
        let offset = doc.position_to_offset(position) as u32;

        let mut current_namespace = String::new();
        for stmt in &ast.statements {
            if let valkyrie_ast::StatementKind::Namespace(n) = stmt {
                current_namespace = n.path.to_string();
                break;
            }
        }

        self.find_symbol_in_statements(&ast.statements, offset, uri, &doc, &current_namespace, None).await
    }

    #[async_recursion::async_recursion]
    async fn find_symbol_in_statements(
        &self,
        statements: &[valkyrie_ast::StatementKind],
        offset: u32,
        uri: &str,
        doc: &DocumentState,
        context_ns: &str,
        context_symbol: Option<&str>,
    ) -> Option<SymbolInfo> {
        for stmt in statements {
            let span = stmt.get_range();
            if offset >= span.start && offset <= span.end {
                match stmt {
                    valkyrie_ast::StatementKind::Function(f) => {
                        if offset >= f.name.span().start && offset <= f.name.span().end {
                            return Some(SymbolInfo {
                                name: f.name.to_string(),
                                namespace: context_ns.to_string(),
                                kind: "function".to_string(),
                                type_info: Some(format!("fn {}", f.name)),
                                documentation: None,
                                location: LocationRange {
                                    uri: uri.to_string(),
                                    range: range_to_lsp_range_usize(&f.get_range()),
                                },
                            });
                        }
                        if offset >= f.body.span.start && offset <= f.body.span.end {
                            return self.find_symbol_in_statements(&f.body.terms, offset, uri, doc, context_ns, Some(&f.name.to_string())).await;
                        }
                    }
                    valkyrie_ast::StatementKind::Class(c) => {
                        if offset >= c.name.span.start && offset <= c.name.span.end {
                            return Some(SymbolInfo {
                                name: c.name.to_string(),
                                namespace: context_ns.to_string(),
                                kind: "class".to_string(),
                                type_info: Some(format!("class {}", c.name)),
                                documentation: None,
                                location: LocationRange {
                                    uri: uri.to_string(),
                                    range: range_to_lsp_range_usize(&c.get_range()),
                                },
                            });
                        }
                        return self.find_symbol_in_class_terms(&c.terms, offset, uri, doc, context_ns, Some(&c.name.to_string())).await;
                    }
                    valkyrie_ast::StatementKind::Variable(v) => {
                        if let Some(body) = &v.body {
                            if offset >= body.get_range().start && offset <= body.get_range().end {
                                return self.find_symbol_in_expression(body, offset, uri, doc, context_ns, context_symbol).await;
                            }
                        }
                    }
                    valkyrie_ast::StatementKind::Expression(e) => {
                        return self.find_symbol_in_expression(&e.body, offset, uri, doc, context_ns, context_symbol).await;
                    }
                    _ => {}
                }
            }
        }
        None
    }

    #[async_recursion::async_recursion]
    async fn find_symbol_in_expression(
        &self,
        expr: &valkyrie_ast::ExpressionKind,
        offset: u32,
        uri: &str,
        doc: &DocumentState,
        context_ns: &str,
        context_symbol: Option<&str>,
    ) -> Option<SymbolInfo> {
        match expr {
            ExpressionKind::Symbol(s) => {
                if let Some(last) = s.path.last() {
                    if offset >= last.span.start && offset <= last.span.end {
                        let name = last.name.to_string();
                        if let Some(global) = self.resolve_symbol(&name, uri, context_ns, context_symbol).await {
                            return Some(SymbolInfo {
                                name: global.name.clone(),
                                namespace: global.namespace.clone(),
                                kind: format!("{:?}", global.kind),
                                type_info: None,
                                documentation: global.documentation.clone(),
                                location: LocationRange { uri: global.uri.clone(), range: global.range },
                            });
                        }
                    }
                }
                None
            }
            ExpressionKind::ApplyCall(c) => {
                if let Some(a) = Box::pin(self.find_symbol_in_expression(&c.caller, offset, uri, doc, context_ns, context_symbol)).await {
                    return Some(a);
                }
                for term in &c.arguments.terms {
                    if let Some(a) = Box::pin(self.find_symbol_in_expression(&term.value, offset, uri, doc, context_ns, context_symbol)).await {
                        return Some(a);
                    }
                }
                None
            }
            ExpressionKind::DotCall(c) => {
                if let Some(object) = Box::pin(self.find_symbol_in_expression(&c.base, offset, uri, doc, context_ns, context_symbol)).await {
                    return Some(object);
                }

                // 检查属性名/方法名
                match &c.term {
                    valkyrie_ast::DotCallTerm::Symbol(p) => {
                        if let Some(last) = p.path.last() {
                            if offset >= last.span.start && offset <= last.span.end {
                                // 尝试推断 base 的类型并查找成员
                                if let Some(base_info) = self.get_expression_type(&c.base, uri, context_ns, context_symbol).await {
                                    if let Some(member) = self.find_member_in_type(&base_info, &last.name.to_string(), uri) {
                                        return Some(member);
                                    }
                                }

                                // 如果没找到特定成员，返回符号本身
                                return Some(SymbolInfo {
                                    name: last.name.to_string(),
                                    namespace: context_ns.to_string(),
                                    kind: "member".to_string(),
                                    type_info: Some(format!("member {}", last.name)),
                                    documentation: None,
                                    location: LocationRange {
                                        uri: uri.to_string(),
                                        range: range_to_lsp_range_usize(&(last.span.start..last.span.end)),
                                    },
                                });
                            }
                        }
                    }
                    _ => {}
                }
                None
            }
            _ => None,
        }
    }

    /// 尝试推断表达式的类型信息（目前仅支持简单的符号查找）
    async fn get_expression_type(&self, expr: &valkyrie_ast::ExpressionKind, uri: &str, context_ns: &str, context_symbol: Option<&str>) -> Option<Arc<GlobalSymbol>> {
        match expr {
            ExpressionKind::Symbol(s) => {
                if let Some(last) = s.path.last() {
                    return self.resolve_symbol(&last.name.to_string(), uri, context_ns, context_symbol).await;
                }
                None
            }
            _ => None,
        }
    }

    /// 在指定类型中查找成员
    fn find_member_in_type(&self, type_info: &GlobalSymbol, member_name: &str, _uri: &str) -> Option<SymbolInfo> {
        // 只有类、接口等才有成员
        if type_info.kind != SymbolKind::Class && type_info.kind != SymbolKind::Interface {
            return None;
        }

        // 获取定义该类型的文件
        let doc = self.get_document(&type_info.uri)?;
        let ast = doc.ast.as_ref()?;

        // 在 AST 中查找该类型的定义
        for stmt in &ast.statements {
            match stmt {
                valkyrie_ast::StatementKind::Class(c) if c.name.to_string() == type_info.name => {
                    // 在类成员中查找
                    for term in &c.terms {
                        match term {
                            valkyrie_ast::ClassTerm::Field(f) if f.name.to_string() == member_name => {
                                return Some(SymbolInfo {
                                    name: f.name.to_string(),
                                    namespace: type_info.namespace.clone(),
                                    kind: "field".to_string(),
                                    type_info: f.typing.as_ref().map(|t| format!("field {}: {:?}", f.name, t)),
                                    documentation: None,
                                    location: LocationRange {
                                        uri: type_info.uri.clone(),
                                        range: range_to_lsp_range_usize(&(f.span.start..f.span.end)),
                                    },
                                });
                            }
                            valkyrie_ast::ClassTerm::Method(m) if m.name.to_string() == member_name => {
                                return Some(SymbolInfo {
                                    name: m.name.to_string(),
                                    namespace: type_info.namespace.clone(),
                                    kind: "method".to_string(),
                                    type_info: Some(format!("method {}", m.name)),
                                    documentation: None,
                                    location: LocationRange {
                                        uri: type_info.uri.clone(),
                                        range: range_to_lsp_range_usize(&(m.name.span().start..m.name.span().end)),
                                    },
                                });
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
        None
    }

    async fn find_symbol_in_class_terms(
        &self,
        terms: &[valkyrie_ast::ClassTerm],
        offset: u32,
        uri: &str,
        doc: &DocumentState,
        context_ns: &str,
        _context_symbol: Option<&str>,
    ) -> Option<SymbolInfo> {
        for term in terms {
            match term {
                valkyrie_ast::ClassTerm::Field(f) => {
                    if offset >= f.span.start && offset <= f.span.end {
                        return Some(SymbolInfo {
                            name: f.name.to_string(),
                            namespace: context_ns.to_string(),
                            kind: "field".to_string(),
                            type_info: f.typing.as_ref().map(|t| format!("field {}: {:?}", f.name, t)),
                            documentation: self.extract_doc(&f.annotations.documents),
                            location: Location {
                                uri: Url::parse(uri).ok()?,
                                range: range_to_lsp_range(&f.span, doc),
                            },
                        });
                    }
                }
                valkyrie_ast::ClassTerm::Method(m) => {
                    if offset >= m.span.start && offset <= m.span.end {
                        // 如果方法有主体，递归检查主体内部
                        if let Some(body) = &m.body {
                            if offset >= body.span.start && offset <= body.span.end {
                                if let Some(inner) = self.find_symbol_in_statements(&body.terms, offset, uri, doc, context_ns, Some(&m.name.to_string())).await {
                                    return Some(inner);
                                }
                            }
                        }
                        return Some(SymbolInfo {
                            name: m.name.to_string(),
                            namespace: context_ns.to_string(),
                            kind: "method".to_string(),
                            type_info: Some(format!("fn {}", m.name)),
                            documentation: None,
                            location: Location {
                                uri: Url::parse(uri).ok()?,
                                range: range_to_lsp_range(&m.span, doc),
                            },
                        });
                    }
                }
                _ => {}
            }
        }
        None
    }
}
