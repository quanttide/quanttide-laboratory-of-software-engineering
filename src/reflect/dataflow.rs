use std::collections::{HashMap, HashSet};
use crate::reflect::FlowEntry;

/// 追踪变量的数据流路径：从使用点追溯到源头
pub fn trace_variable(
    source: &str,
    tree: &tree_sitter::Tree,
    start_line: usize,
    var: &str,
) -> Vec<FlowEntry> {
    let root = tree.root_node();
    let decls = collect_all_decls(&root, source);
    let mut path = Vec::new();
    let mut visited = HashSet::new();
    let mut stack = vec![(var.to_string(), start_line)];

    while let Some((current_var, _line)) = stack.pop() {
        if !visited.insert(current_var.clone()) { continue; }

        // 找到声明行
        let decl_line = match decls.get(&current_var) {
            Some(&l) => l,
            None => continue,
        };

        // 找到声明语句
        let stmt_text = extract_stmt_at_line(&root, decl_line, source);
        let from = if let Some(ref text) = stmt_text {
            extract_rhs(text)
        } else {
            String::new()
        };

        path.push(FlowEntry {
            var: current_var.clone(),
            from: from.clone(),
            line: decl_line,
        });

        // 从 RHS 提取上游变量继续追踪
        for upstream in extract_upstream_vars(&from) {
            stack.push((upstream, decl_line));
        }
    }

    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_variable_empty() {
        // 空源码不应 panic
        let mut parser = tree_sitter::Parser::new();
        if parser.set_language(&tree_sitter_rust::LANGUAGE.into()).is_err() { return; }
        if let Some(tree) = parser.parse("", None) {
            let r = trace_variable("", &tree, 1, "x");
            assert!(r.is_empty());
        }
    }

    #[test]
    fn test_trace_variable_finds_decl() {
        let code = "fn f() {\nlet x = 1;\nlet y = x;\ny\n}";
        let mut p = tree_sitter::Parser::new();
        if p.set_language(&tree_sitter_rust::LANGUAGE.into()).is_err() { return; }
        if let Some(tree) = p.parse(code, None) {
            let r = trace_variable(code, &tree, 3, "y");
            assert!(!r.is_empty(), "should find y's declaration");
            assert!(r.iter().any(|e| e.var == "y"), "should trace y");
            // y depends on x, should also find x
            assert!(r.iter().any(|e| e.var == "x"), "should trace x as upstream of y");
        }
    }

    #[test]
    fn test_trace_variable_unknown() {
        let code = "fn f() { let x = 1; x }";
        let mut p = tree_sitter::Parser::new();
        if p.set_language(&tree_sitter_rust::LANGUAGE.into()).is_err() { return; }
        if let Some(tree) = p.parse(code, None) {
            let r = trace_variable(code, &tree, 1, "nonexistent");
            assert!(r.is_empty(), "unknown var should return empty");
        }
    }

    #[test]
    fn test_walk_all_terminates() {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&tree_sitter_rust::LANGUAGE.into()).unwrap();
        if let Some(tree) = parser.parse("fn f() { let x = 1; }", None) {
            let mut count = 0;
            walk_all(&tree.root_node(), &mut |_| count += 1);
            assert!(count > 0 && count < 200, "walk_all count: {}", count);
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

fn collect_all_decls(root: &tree_sitter::Node, source: &str) -> HashMap<String, usize> {
    let mut decls = HashMap::new();
    walk_all(root, &mut |n| {
        if n.is_named() && n.kind() == "let_declaration" {
            if let Some(name) = extract_let_name(&n, source) {
                decls.insert(name.to_string(), n.start_position().row + 1);
            }
        }
    });
    decls
}

fn extract_let_name<'t>(node: &tree_sitter::Node<'t>, source: &str) -> Option<String> {
    if let Some(name) = node.child_by_field_name("pattern")
        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
    {
        return Some(name.to_string());
    }
    // fallback: 第一个 identifier
    let mut result = None;
    walk_all(node, &mut |n| {
        if result.is_some() { return; }
        if n.is_named() && n.kind() == "identifier" {
            if let Ok(name) = n.utf8_text(source.as_bytes()) {
                result = Some(name.to_string());
            }
        }
    });
    result
}

fn extract_stmt_at_line(root: &tree_sitter::Node, line: usize, source: &str) -> Option<String> {
    let mut cursor = root.walk();
    loop {
        let n = cursor.node();
        if n.is_named() {
            let start = n.start_position().row + 1;
            let end = n.end_position().row + 1;
            if start <= line && line <= end && is_stmt_or_cont(n.kind()) {
                if start == line {
                    return n.utf8_text(source.as_bytes()).ok().map(|s| s.to_string());
                }
            }
            if cursor.goto_first_child() { continue; }
        }
        loop { if cursor.goto_next_sibling() { break; } if !cursor.goto_parent() { break; } }
    }
}

fn is_stmt_or_cont(k: &str) -> bool {
    matches!(k, "let_declaration" | "expression_statement" | "return_statement"
        | "block" | "for_expression" | "if_expression")
}

fn extract_rhs(stmt: &str) -> String {
    if let Some(eq) = stmt.find('=') {
        let rhs = stmt[eq + 1..].trim_end_matches(';').trim().to_string();
        rhs
    } else {
        stmt.to_string()
    }
}

fn extract_upstream_vars(rhs: &str) -> Vec<String> {
    let mut vars = Vec::new();
    for word in rhs.split(|c: char| !c.is_alphanumeric() && c != '_') {
        if !word.is_empty() && word.chars().all(|c| c.is_alphanumeric() || c == '_')
            && word != &word.to_uppercase()  // 跳过常量
        {
            vars.push(word.to_string());
        }
    }
    vars
}
