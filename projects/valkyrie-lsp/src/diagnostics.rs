//! 诊断信息管理
//!
//! 将 Nyar 编译器的诊断信息转换为 LSP 格式

use oak_lsp::types::*;
use tracing::{debug, warn};
use valkyrie_error::{ReportKind, ValkyrieError};

/// 诊断信息管理器
pub struct DiagnosticsManager {
    // 可以添加诊断信息的缓存和过滤逻辑
}

impl DiagnosticsManager {
    pub fn new() -> Self {
        Self {}
    }

    /// 将 Nyar 编译器诊断信息转换为 LSP 诊断信息
    pub fn convert_to_lsp_diagnostics(&self, compiler_diagnostics: &[ValkyrieError], source: &str) -> Vec<Diagnostic> {
        compiler_diagnostics.iter().filter_map(|diag| self.convert_single_diagnostic(diag, source)).collect()
    }

    /// 转换单个诊断信息
    fn convert_single_diagnostic(&self, diag: &ValkyrieError, source: &str) -> Option<Diagnostic> {
        // 获取诊断信息的位置
        let range = self.extract_range_from_diagnostic(diag, source)?;

        // 确定诊断严重程度
        let severity = self.map_severity(diag);

        // 提取错误代码
        let code = self.extract_error_code(diag);

        // 构建 LSP 诊断信息
        Some(Diagnostic {
            range,
            severity: Some(severity),
            code,
            source: Some("valkyrie-lsp".to_string()),
            message: diag.to_string(),
        })
    }

    /// 从诊断信息中提取位置范围
    fn extract_range_from_diagnostic(&self, diag: &ValkyrieError, _source: &str) -> Option<Range<usize>> {
        let span = diag.span()?;
        let start_offset = span.get_start() as usize;
        let end_offset = span.get_end() as usize;

        Some(Range { start: start_offset, end: end_offset })
    }

    /// 映射诊断严重程度
    fn map_severity(&self, diag: &ValkyrieError) -> DiagnosticSeverity {
        match diag.level() {
            ReportKind::Error => DiagnosticSeverity::Error,
            _ => DiagnosticSeverity::Warning,
        }
    }

    /// 提取错误代码
    fn extract_error_code(&self, _diag: &ValkyrieError) -> Option<String> {
        // TODO: 从 ValkyrieError 中提取错误代码
        None
    }
}

/// 诊断统计信息
#[derive(Debug, Clone, Default)]
pub struct DiagnosticStats {
    /// 错误数量
    pub errors: usize,
    /// 警告数量
    pub warnings: usize,
    /// 信息数量
    pub infos: usize,
    /// 提示数量
    pub hints: usize,
}

/// 诊断过滤配置
#[derive(Debug, Clone, Default)]
pub struct DiagnosticFilterConfig {
    /// 是否忽略警告
    pub ignore_warnings: bool,
    /// 是否忽略提示
    pub ignore_hints: bool,
    /// 排除的错误代码
    pub excluded_codes: Vec<String>,
}
