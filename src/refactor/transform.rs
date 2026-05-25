use std::path::Path;

/// 函数提取的候选信息
#[derive(Debug, Clone)]
pub struct ExtractCandidate {
    pub start_line: usize,
    pub end_line: usize,
    pub code: String,
}

/// 根据行范围找到可提取的最小完整语句
pub fn find_extract_boundary(source: &str, tree: &tree_sitter::Tree, line: usize) -> Option<ExtractCandidate> {
    let root = tree.root_node();
    let byte = byte_for_line(source, line);
    let node = root.descendant_for_byte_range(byte, byte)?;
    let mut extract = node;
    loop {
        if let Some(parent) = extract.parent() {
            if is_safe_boundary(parent.kind()) { break; }
            extract = parent;
        } else { break; }
    }
    let start_line = extract.start_position().row + 1;
    let end_line = extract.end_position().row + 1;
    let code = extract.utf8_text(source.as_bytes()).ok()?.to_string();
    Some(ExtractCandidate { start_line, end_line, code })
}

fn is_safe_boundary(k: &str) -> bool {
    matches!(k, "block" | "function_item" | "for_expression" | "if_expression"
        | "while_expression" | "loop_expression" | "match_expression"
        | "expression_statement" | "source_file")
}

pub fn byte_for_line(source: &str, line: usize) -> usize {
    let mut byte = 0;
    let mut current = 1;
    for ch in source.chars() {
        if current >= line { return byte; }
        byte += ch.len_utf8();
        if ch == '\n' { current += 1; }
    }
    source.len()
}
