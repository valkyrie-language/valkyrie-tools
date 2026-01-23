//! 诊断信息管理
//!
//! 将 Nyar 编译器的诊断信息转换为 LSP 格式

use tower_lsp::lsp_types::*;
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
            code: code.map(NumberOrString::String),
            code_description: None,
            source: Some("valkyrie-lsp".to_string()),
            message: diag.to_string(),
            related_information: self.extract_related_information(diag),
            tags: self.extract_diagnostic_tags(diag),
            data: None,
        })
    }

    /// 从诊断信息中提取位置范围
    fn extract_range_from_diagnostic(&self, diag: &ValkyrieError, source: &str) -> Option<Range> {
        let span = diag.span()?;
        let start_offset = span.get_start() as usize;
        let end_offset = span.get_end() as usize;

        Some(Range { start: self.offset_to_position(start_offset, source), end: self.offset_to_position(end_offset, source) })
    }

    /// 将偏移量转换为 LSP Position
    fn offset_to_position(&self, offset: usize, source: &str) -> Position {
        let mut line = 0;
        let mut character = 0;

        for (i, c) in source.char_indices() {
            if i >= offset {
                break;
            }

            if c == '\n' {
                line += 1;
                character = 0;
            }
            else {
                character += 1;
            }
        }

        Position::new(line, character)
    }

    /// 映射诊断严重程度
    fn map_severity(&self, diag: &ValkyrieError) -> DiagnosticSeverity {
        match diag.level() {
            ReportKind::Error => DiagnosticSeverity::ERROR,
            _ => DiagnosticSeverity::WARNING,
        }
    }

    /// 提取错误代码
    fn extract_error_code(&self, _diag: &ValkyrieError) -> Option<String> {
        // TODO: 从 ValkyrieError 中提取错误代码
        None
    }

    /// 提取相关信息
    fn extract_related_information(&self, _diag: &ValkyrieError) -> Option<Vec<DiagnosticRelatedInformation>> {
        None
    }

    /// 提取诊断标签
    fn extract_diagnostic_tags(&self, _diag: &ValkyrieError) -> Option<Vec<DiagnosticTag>> {
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
