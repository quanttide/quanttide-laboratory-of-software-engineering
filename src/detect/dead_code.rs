#[cfg(test)]
mod tests {
    use super::*;

    fn parse_rust(code: &str) -> Option<(String, tree_sitter::Tree)> {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&tree_sitter_rust::LANGUAGE.into()).ok()?;
        let tree = parser.parse(code, None)?;
        Some((code.to_string(), tree))
    }

    #[test]
    fn test_dead_code_detects_unused() {
        if let Some((s, t)) = parse_rust("fn used() {} fn unused() {} fn main() { used(); }") {
            let dead = check_dead_code(&s, &t);
            assert_eq!(dead.len(), 1);
            assert_eq!(dead[0].name, "unused");
        }
    }

    #[test]
    fn test_dead_code_empty() {
        if let Some((s, t)) = parse_rust("") {
            assert!(check_dead_code(&s, &t).is_empty());
        }
    }

    #[test]
    fn test_walk_all_terminates() {
        if let Some((s, t)) = parse_rust("fn a() { fn b() {} }") {
            let mut count = 0;
            walk_all(&t.root_node(), &mut |_| count += 1);
            assert!(count > 0 && count < 100);
        }
    }
}

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
