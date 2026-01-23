use tower_lsp::lsp_types::*;
use crate::state::ServerState;
use valkyrie_ast::helper::{PrettyPrint, PrettyProvider};

/// 格式化处理器
pub struct FormattingHandler;

impl FormattingHandler {
    pub async fn handle(state: &ServerState, params: DocumentFormattingParams) -> Option<Vec<TextEdit>> {
        let uri = params.text_document.uri.to_string();
        let doc = state.get_document(&uri)?;
        let ast = doc.ast.as_ref()?;

        // 获取格式化选项
        let tab_size = params.options.tab_size as usize;
        let insert_spaces = params.options.insert_spaces;

        let theme = PrettyProvider::new(80);
        let tree = ast.pretty(&theme);

        // 目前 pretty-print 库可能不支持动态设置缩进大小
        // 我们暂时使用 80 作为推荐宽度，并进行后处理
        let mut buf = Vec::new();
        let _ = tree.render(80, &mut buf);
        let mut formatted = String::from_utf8_lossy(&buf).to_string();

        // 调整缩进
        formatted = Self::adjust_indentation(formatted, tab_size, insert_spaces);

        // 处理其他格式化选项
        if params.options.trim_trailing_whitespace.unwrap_or(false) {
            let tree_ends_with_newline = formatted.ends_with('\n');
            formatted = formatted.lines().map(|line| line.trim_end()).collect::<Vec<_>>().join("\n");
            // 保持末尾换行符状态
            if tree_ends_with_newline && !formatted.ends_with('\n') {
                formatted.push('\n');
            }
        }

        if params.options.insert_final_newline.unwrap_or(false) {
            if !formatted.ends_with('\n') {
                formatted.push('\n');
            }
        }
        else if params.options.trim_final_newlines.unwrap_or(false) {
            while formatted.ends_with("\n\n") {
                formatted.pop();
            }
        }

        if formatted == doc.text {
            return None;
        }

        Some(vec![TextEdit {
            range: Range { start: Position { line: 0, character: 0 }, end: doc.offset_to_position(doc.text.len()) },
            new_text: formatted,
        }])
    }

    fn adjust_indentation(text: String, tab_size: usize, insert_spaces: bool) -> String {
        if tab_size == 4 && insert_spaces {
            return text;
        }

        let mut result = String::with_capacity(text.len());
        let has_trailing_newline = text.ends_with('\n');

        for (i, line) in text.lines().enumerate() {
            if i > 0 {
                result.push('\n');
            }

            let mut spaces = 0;
            for c in line.chars() {
                if c == ' ' {
                    spaces += 1;
                }
                else {
                    break;
                }
            }

            let content = &line[spaces..];
            if spaces > 0 {
                let levels = spaces / 4;
                let rem = spaces % 4;
                if insert_spaces {
                    result.push_str(&" ".repeat(levels * tab_size + rem));
                }
                else {
                    result.push_str(&"\t".repeat(levels));
                    result.push_str(&" ".repeat(rem));
                }
            }
            result.push_str(content);
        }

        if has_trailing_newline {
            result.push('\n');
        }

        result
    }
}
