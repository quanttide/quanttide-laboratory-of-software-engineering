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

fn collect_all_decls<'t>(root: &tree_sitter::Node<'t>, source: &str) -> HashMap<String, usize> {
    let mut decls = HashMap::new();
    let mut cursor = root.walk();
    loop {
        let n = cursor.node();
        if n.is_named() && n.kind() == "let_declaration" {
            if let Some(name) = n.child_by_field_name("pattern")
                .or_else(|| {
                    let mut cc = n.walk();
                    if cc.goto_first_child() {
                        loop {
                            let sub = cc.node();
                            if sub.is_named() && sub.kind() == "identifier" {
                                return Some(sub);
                            }
                            if !cc.goto_next_sibling() { break; }
                        }
                    }
                    None
                })
                .and_then(|nn| nn.utf8_text(source.as_bytes()).ok())
            {
                decls.insert(name.to_string(), n.start_position().row + 1);
            }
        }
        if cursor.goto_first_child() { continue; }
        loop { if cursor.goto_next_sibling() { break; } if !cursor.goto_parent() { break; } }
    }
    decls
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
