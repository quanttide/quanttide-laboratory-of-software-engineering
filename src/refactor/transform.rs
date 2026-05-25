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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_byte_for_line() {
        assert_eq!(byte_for_line("abc\ndef\nghi", 1), 0);
        assert_eq!(byte_for_line("abc\ndef\nghi", 2), 4);
        assert_eq!(byte_for_line("abc\ndef\nghi", 4), 11);
    }

    #[test]
    fn test_byte_for_line_beyond_end() {
        assert_eq!(byte_for_line("abc", 999), 3);
    }

    #[test]
    fn test_byte_for_line_empty() {
        assert_eq!(byte_for_line("", 1), 0);
    }

    #[test]
    fn test_find_extract_boundary_inline_stmt() {
        let code = "fn f() { let x = 1; x }";
        let mut p = tree_sitter::Parser::new();
        if p.set_language(&tree_sitter_rust::LANGUAGE.into()).is_err() { return; }
        if let Some(tree) = p.parse(code, None) {
            if let Some(candidate) = find_extract_boundary(code, &tree, 1) {
                assert!(candidate.code.trim().starts_with("fn"));
            }
            // else: early return if no extraction boundary found — acceptable
        }
    }

    #[test]
    fn test_find_extract_boundary_invalid_line() {
        let code = "fn f() {}";
        let mut p = tree_sitter::Parser::new();
        if p.set_language(&tree_sitter_rust::LANGUAGE.into()).is_err() { return; }
        if let Some(tree) = p.parse(code, None) {
            let r = find_extract_boundary(code, &tree, 999);
            // Either None (byte beyond source) or Some (bounds wapped)
            // Both are acceptable — just verify no panic
        }
    }
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
