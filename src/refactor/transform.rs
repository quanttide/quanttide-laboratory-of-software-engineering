

fn walk_all<F: FnMut(tree_sitter::Node)>(node: &tree_sitter::Node, f: &mut F) {
    f(*node);
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            walk_all(&cursor.node(), f);
            if !cursor.goto_next_sibling() { break; }
        }
    }
}

/// 检测死代码：找出未被调用的函数
pub fn detect_dead_code(source: &str, tree: &tree_sitter::Tree) -> Vec<DeadFunc> {
    let root = tree.root_node();

    // 收集所有函数定义
    let mut funcs: Vec<(String, usize, bool)> = Vec::new();
    walk_all(&root, &mut |n| {
        if n.is_named() && n.kind() == "function_item" {
            if let Some(name) = n.child_by_field_name("name")
                .and_then(|nn| nn.utf8_text(source.as_bytes()).ok())
            {
                funcs.push((name.to_string(), n.start_position().row + 1, false));
            }
        }
    });

    // 第二遍标记调用
    walk_all(&root, &mut |n| {
        if n.is_named() && n.kind() == "call_expression" {
            if let Some(name) = n.child_by_field_name("function")
                .or_else(|| {
                    let mut cc = n.walk();
                    if cc.goto_first_child() {
                        loop {
                            let ch = cc.node();
                            if ch.is_named() && ch.kind() == "identifier" {
                                return Some(ch);
                            }
                            if !cc.goto_next_sibling() { break; }
                        }
                    }
                    None
                })
                .and_then(|nn| nn.utf8_text(source.as_bytes()).ok())
            {
                for func in &mut funcs {
                    if func.0 == name { func.2 = true; }
                }
            }
        }
    });

    funcs.into_iter()
        .filter(|(name, _, called)| !called && name != "main")
        .map(|(name, line, _)| DeadFunc { name, line })
        .collect()
}

#[derive(Debug, Clone)]
pub struct DeadFunc {
    pub name: String,
    pub line: usize,
}

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

    // 向上找到 statement 级别的节点
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

fn byte_for_line(source: &str, line: usize) -> usize {
    let mut byte = 0;
    let mut current = 1;
    for ch in source.chars() {
        if current >= line { return byte; }
        byte += ch.len_utf8();
        if ch == '\n' { current += 1; }
    }
    source.len()
}
