use std::collections::{HashMap, HashSet};
use std::path::Path;
use crate::reflect::SliceEntry;

/// 构建反向程序切片：给定代码位置，找出所有影响该点的语句
pub fn backward_slice(
    source: &str,
    tree: &tree_sitter::Tree,
    file: &Path,
    line: usize,
) -> Vec<SliceEntry> {
    let root = tree.root_node();

    // 找到包含该行的函数
    let func = find_containing_function(&root, line);
    let Some(func) = func else { return vec![] };

    // 提取函数内所有语句（展平）
    let stmts = flatten_stmts(&func);

    // 构建声明表
    let decls = build_decls(&func, source);

    // 反向追溯
    let mut results = Vec::new();
    let mut visited = HashSet::new();
    let mut stack = vec![line];

    while let Some(current) = stack.pop() {
        if !visited.insert(current) { continue; }
        let Some(stmt) = stmts.iter().find(|s| s.start_position().row + 1 == current) else { continue };

        results.push(SliceEntry {
            file: file.to_string_lossy().to_string(),
            line: current,
            text: stmt.utf8_text(source.as_bytes()).unwrap_or("?").to_string(),
        });

        for var in extract_identifiers(stmt, source.as_bytes()) {
            if let Some(&decl_line) = decls.get(&var) {
                stack.push(decl_line);
            }
        }
    }

    results.sort_by_key(|e| e.line);
    results
}

/// 跨函数反向切片
pub fn cross_function_slice(
    source: &str,
    tree: &tree_sitter::Tree,
    file: &Path,
    start_line: usize,
) -> Vec<SliceEntry> {
    let root = tree.root_node();
    let funcs = collect_functions(&root, source);
    let mut all_results = Vec::new();
    let mut visited = HashSet::new();
    let mut stack: Vec<(String, usize)> = Vec::new();

    // 找到起始函数
    let start_func = find_containing_function_name(&root, start_line, source);
    let Some(start_name) = start_func else { return backward_slice(source, tree, file, start_line) };
    stack.push((start_name, start_line));

    while let Some((func_name, line)) = stack.pop() {
        let key = format!("{}:{}", func_name, line);
        if !visited.insert(key) { continue; }
        let Some(func_node) = funcs.get(&func_name) else { continue };
        let stmts = flatten_stmts(func_node);
        let decls = build_decls(func_node, source);

        let mut local = Vec::new();
        let mut local_visited = HashSet::new();
        let mut local_stack = vec![line];

        while let Some(current) = local_stack.pop() {
            if !local_visited.insert(current) { continue; }
            let Some(stmt) = stmts.iter().find(|s| s.start_position().row + 1 == current) else { continue };

            local.push(SliceEntry {
                file: file.to_string_lossy().to_string(),
                line: current,
                text: stmt.utf8_text(source.as_bytes()).unwrap_or("?").to_string(),
            });

            for var in extract_identifiers(stmt, source.as_bytes()) {
                if let Some(&decl_line) = decls.get(&var) {
                    local_stack.push(decl_line);
                }
            }

            // 检测函数调用
            if let Some(callee) = find_callee_in_stmt(stmt, source) {
                if let Some(ret_line) = find_return_line(func_node, &callee, source, tree) {
                    stack.push((callee, ret_line));
                }
            }
        }

        all_results.extend(local);
    }

    all_results.sort_by_key(|e| e.line);
    all_results
}

// ===== 内部工具 =====

fn find_containing_function<'t>(root: &tree_sitter::Node<'t>, line: usize) -> Option<tree_sitter::Node<'t>> {
    let mut cursor = root.walk();
    loop {
        let n = cursor.node();
        if n.is_named() && n.kind() == "function_item" {
            let start = n.start_position().row + 1;
            let end = n.end_position().row + 1;
            if start <= line && line <= end { return Some(n); }
        }
        if cursor.goto_first_child() { continue; }
        loop { if cursor.goto_next_sibling() { break; } if !cursor.goto_parent() { break; } }
    }
}

fn find_containing_function_name(root: &tree_sitter::Node, line: usize, source: &str) -> Option<String> {
    let func = find_containing_function(root, line)?;
    func.child_by_field_name("name")
        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
        .map(|s| s.to_string())
}

fn collect_functions<'t>(root: &tree_sitter::Node<'t>, source: &str) -> HashMap<String, tree_sitter::Node<'t>> {
    let mut funcs = HashMap::new();
    let mut cursor = root.walk();
    loop {
        let n = cursor.node();
        if n.is_named() && n.kind() == "function_item" {
            if let Some(name) = n.child_by_field_name("name")
                .and_then(|nn| nn.utf8_text(source.as_bytes()).ok())
            {
                funcs.insert(name.to_string(), n);
            }
        }
        if cursor.goto_first_child() { continue; }
        loop { if cursor.goto_next_sibling() { break; } if !cursor.goto_parent() { break; } }
    }
    funcs
}

fn flatten_stmts<'t>(node: &tree_sitter::Node<'t>) -> Vec<tree_sitter::Node<'t>> {
    let mut stmts = Vec::new();
    recurse_stmt(node, &mut stmts);
    stmts
}

fn recurse_stmt<'t>(node: &tree_sitter::Node<'t>, out: &mut Vec<tree_sitter::Node<'t>>) {
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            let child = cursor.node();
            if child.is_named() {
                let k = child.kind();
                if is_stmt(k) { out.push(child); }
                if is_container(k) { recurse_stmt(&child, out); }
            }
            if !cursor.goto_next_sibling() { break; }
        }
    }
}

fn is_stmt(k: &str) -> bool {
    matches!(k, "let_declaration" | "expression_statement" | "return_statement")
}

fn is_container(k: &str) -> bool {
    matches!(k, "block" | "for_expression" | "if_expression" | "while_expression"
        | "loop_expression" | "match_expression" | "expression_statement")
}

fn build_decls<'t>(node: &tree_sitter::Node<'t>, source: &str) -> HashMap<String, usize> {
    let mut decls = HashMap::new();
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            let child = cursor.node();
            if child.is_named() && child.kind() == "let_declaration" {
                if let Some(name) = child.child_by_field_name("pattern")
                    .or_else(|| {
                        // fallback: 第一个 identifier
                        let mut cc = child.walk();
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
                    .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                {
                    decls.insert(name.to_string(), child.start_position().row + 1);
                }
            }
            if child.is_named() && is_container(child.kind()) {
                for (k, v) in build_decls(&child, source) {
                    decls.entry(k).or_insert(v);
                }
            }
            if !cursor.goto_next_sibling() { break; }
        }
    }
    decls
}

fn extract_identifiers(node: &tree_sitter::Node, source: &[u8]) -> Vec<String> {
    let mut names = Vec::new();
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            let child = cursor.node();
            if child.is_named() && child.kind() == "identifier" {
                if let Ok(name) = child.utf8_text(source) {
                    names.push(name.to_string());
                }
            }
            if child.is_named() && child.kind() != "call_expression" {
                names.extend(extract_identifiers(&child, source));
            }
            if !cursor.goto_next_sibling() { break; }
        }
    }
    names
}

fn find_callee_in_stmt(stmt: &tree_sitter::Node, source: &str) -> Option<String> {
    let mut cursor = stmt.walk();
    loop {
        let n = cursor.node();
        if n.is_named() && n.kind() == "call_expression" {
            if let Some(callee) = n.child_by_field_name("function")
                .and_then(|c| c.utf8_text(source.as_bytes()).ok())
            {
                return Some(callee.to_string());
            }
            // fallback: 第一个 identifier
            let mut cc = n.walk();
            if cc.goto_first_child() {
                loop {
                    let ch = cc.node();
                    if ch.is_named() && ch.kind() == "identifier" {
                        if let Ok(name) = ch.utf8_text(source.as_bytes()) {
                            return Some(name.to_string());
                        }
                    }
                    if !cc.goto_next_sibling() { break; }
                }
            }
        }
        if cursor.goto_first_child() { continue; }
        loop { if cursor.goto_next_sibling() { break; } if !cursor.goto_parent() { break; } }
    }
    None
}

fn find_return_line(func_node: &tree_sitter::Node, _callee: &str, source: &str, tree: &tree_sitter::Tree) -> Option<usize> {
    let stmts = flatten_stmts(func_node);
    // 优先 return_statement
    for s in &stmts {
        if s.kind() == "return_statement" { return Some(s.start_position().row + 1); }
    }
    // 取 block 最后一个 expression_statement（隐式返回）
    let root = tree.root_node();
    let mut cursor = root.walk();
    loop {
        let n = cursor.node();
        if n == *func_node {
            let mut cc = n.walk();
            if cc.goto_first_child() {
                loop {
                    let child = cc.node();
                    if child.is_named() && child.kind() == "block" {
                        let mut bc = child.walk();
                        let mut last: Option<tree_sitter::Node> = None;
                        if bc.goto_first_child() {
                            loop {
                                let inner = bc.node();
                                if inner.is_named() { last = Some(inner); }
                                if !bc.goto_next_sibling() { break; }
                            }
                        }
                        return last.map(|n| n.start_position().row + 1);
                    }
                    if !cc.goto_next_sibling() { break; }
                }
            }
        }
        if cursor.goto_first_child() { continue; }
        loop { if cursor.goto_next_sibling() { break; } if !cursor.goto_parent() { break; } }
    }
}
