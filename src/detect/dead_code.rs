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

#[derive(Debug, Clone)]
pub struct DeadFunc {
    pub name: String,
    pub line: usize,
}

pub const RULE_ID: &str = "dead-code";
pub const DESCRIPTION: &str = "未被调用的函数";

/// 检测死代码：找出未被调用的函数
pub fn check_dead_code(source: &str, tree: &tree_sitter::Tree) -> Vec<DeadFunc> {
    let root = tree.root_node();
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

    walk_all(&root, &mut |n| {
        if n.is_named() && n.kind() == "call_expression" {
            if let Some(name) = n.child_by_field_name("function")
                .or_else(|| {
                    let mut cc = n.walk();
                    if cc.goto_first_child() {
                        loop {
                            let ch = cc.node();
                            if ch.is_named() && ch.kind() == "identifier" { return Some(ch); }
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
