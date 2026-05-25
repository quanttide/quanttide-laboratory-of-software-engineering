/// 跨函数反向切片实验

use std::collections::HashMap;

fn main() {
    let code = r#"
fn helper(x: i32) -> i32 {
    let y = x + 1;
    y
}

fn process(items: &[i32], threshold: i32) -> i32 {
    let mut sum = 0;
    for item in items {
        if *item > threshold {
            let v = helper(*item);
            sum += v;
        }
    }
    sum
}
"#;

    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&tree_sitter_rust::LANGUAGE.into()).unwrap();
    let tree = parser.parse(code, None).unwrap();
    let root = tree.root_node();

    let functions = collect_functions(&root, code);
    println!("=== 函数列表 ===");
    for (name, node) in &functions {
        println!("  {}: {}:{}-{}:{}", name,
            node.start_position().row + 1, node.start_position().column + 1,
            node.end_position().row + 1, node.end_position().column + 1);
    }

    let mut func_decls: HashMap<String, HashMap<String, usize>> = HashMap::new();
    let mut func_stmts: HashMap<String, Vec<tree_sitter::Node>> = HashMap::new();
    for (name, node) in &functions {
        func_decls.insert(name.clone(), find_declarations(node, code));
        func_stmts.insert(name.clone(), flatten_stmts(node));
    }

    // 打印所有函数语句
    for fname in ["helper", "process"] {
        println!("\n=== {} 函数语句 ===", fname);
        for s in func_stmts.get(fname).unwrap() {
            let line = s.start_position().row + 1;
            let text = s.utf8_text(code.as_bytes()).unwrap_or("?").to_string();
            let short = if text.len() > 60 { format!("{}...", &text[..57]) } else { text };
            println!("  L{} ({}) {}", line, s.kind(), short);
        }
    }

    // 跨函数反向切片
    let finding_line = 12; // sum += v
    println!("\n=== 跨函数反向切片：L{} ===", finding_line);
    let result = cross_function_slice(&functions, &func_decls, &func_stmts, code, &"process".to_string(), finding_line);
    for (i, (func, l, text)) in result.iter().enumerate() {
        let short = if text.len() > 60 { format!("{}...", &text[..57]) } else { text.to_string() };
        println!("  {}. {}:L{} {}", i + 1, func, l, short);
    }
}

// ===== 基础方法 =====

fn collect_functions<'t>(root: &tree_sitter::Node<'t>, source: &str) -> HashMap<String, tree_sitter::Node<'t>> {
    let mut funcs = HashMap::new();
    let mut cursor = root.walk();
    loop {
        let node = cursor.node();
        if node.is_named() && node.kind() == "function_item" {
            let name = node.child_by_field_name("name")
                .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                .unwrap_or("<anon>")
                .to_string();
            funcs.insert(name, node);
        }
        if cursor.goto_first_child() { continue; }
        loop {
            if cursor.goto_next_sibling() { break; }
            if !cursor.goto_parent() { return funcs; }
        }
    }
}

/// 递归展平语句：只收集 block 直接子节点中的语句，不进入表达式内部
fn flatten_stmts<'t>(node: &tree_sitter::Node<'t>) -> Vec<tree_sitter::Node<'t>> {
    let mut stmts = Vec::new();
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            let child = cursor.node();
            if child.is_named() {
                let kind = child.kind();
                if is_stmt_kind(kind) {
                    stmts.push(child);
                }
                if is_container(kind) {
                    stmts.extend(flatten_stmts(&child));
                }
            }
            if !cursor.goto_next_sibling() { break; }
        }
    }
    stmts
}

fn is_stmt_kind(k: &str) -> bool {
    matches!(k, "let_declaration" | "expression_statement" | "return_statement")
}

fn is_container(k: &str) -> bool {
    matches!(k, "block" | "for_expression" | "if_expression"
        | "while_expression" | "loop_expression" | "match_expression"
        | "expression_statement")
}

fn find_declarations(node: &tree_sitter::Node, source: &str) -> HashMap<String, usize> {
    let mut decls = HashMap::new();
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            let child = cursor.node();
            if child.is_named() && child.kind() == "let_declaration" {
                // 取第一个 identifier 子节点作为变量名
                let mut cc = child.walk();
                if cc.goto_first_child() {
                    loop {
                        let sub = cc.node();
                        if sub.is_named() && sub.kind() == "identifier" {
                            if let Ok(name) = sub.utf8_text(source.as_bytes()) {
                                decls.insert(name.to_string(), child.start_position().row + 1);
                                break;
                            }
                        }
                        if !cc.goto_next_sibling() { break; }
                    }
                }
            }
            if child.is_named() && is_container(child.kind()) {
                decls.extend(find_declarations(&child, source));
            }
            if !cursor.goto_next_sibling() { break; }
        }
    }
    decls
}

// ===== 返回行检测 =====

fn find_return_line(functions: &HashMap<String, tree_sitter::Node>, name: &str) -> Option<usize> {
    let node = functions.get(name)?;
    let stmts = flatten_stmts(node);
    // 优先找 return_statement
    for s in &stmts {
        if s.kind() == "return_statement" { return Some(s.start_position().row + 1); }
    }
    // 没有 return 则取函数体 block 的最后一条命名子节点（Rust 隐式返回）
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            let child = cursor.node();
            if child.is_named() && child.kind() == "block" {
                let mut cc = child.walk();
                let mut last: Option<tree_sitter::Node> = None;
                if cc.goto_first_child() {
                    loop {
                        let inner = cc.node();
                        if inner.is_named() { last = Some(inner); }
                        if !cc.goto_next_sibling() { break; }
                    }
                }
                return last.map(|n| n.start_position().row + 1);
            }
            if !cursor.goto_next_sibling() { break; }
        }
    }
    None
}

// ===== 跨函数切片 =====

fn cross_function_slice(
    functions: &HashMap<String, tree_sitter::Node>,
    func_decls: &HashMap<String, HashMap<String, usize>>,
    func_stmts: &HashMap<String, Vec<tree_sitter::Node>>,
    source: &str,
    start_func: &String,
    start_line: usize,
) -> Vec<(String, usize, String)> {
    let mut results = Vec::new();
    let mut visited = std::collections::HashSet::new();
    let mut stack = vec![(start_func.to_string(), start_line)];

    while let Some((func_name, line)) = stack.pop() {
        let Some(stmts) = func_stmts.get(&func_name) else { continue };
        let Some(decls) = func_decls.get(&func_name) else { continue };

        let slice = backward_slice(stmts, decls, source, line);
        for l in &slice {
            let key = format!("{}:{}", func_name, l);
            if visited.insert(key.clone()) {
                let text = match stmts.iter().find(|s| s.start_position().row + 1 == *l) {
                    Some(stmt) => stmt.utf8_text(source.as_bytes()).unwrap_or("?").to_string(),
                    None => format!("(implicit return line {})", l),
                };
                let short = if text.len() > 60 { format!("{}...", &text[..57]) } else { text };
                results.push((func_name.clone(), *l, short));

                // 检测函数调用 → 追溯被调用函数
                        if let Some(stmt) = stmts.iter().find(|s| s.start_position().row + 1 == *l) {
                                if let Some(callee) = find_callee_in_stmt(stmt, source) {
                                    if let Some(ret_line) = find_return_line(functions, &callee) {
                                        stack.push((callee, ret_line));
                                    }
                                }
                            }
            }
        }
    }

    results.sort_by_key(|(_, l, _)| *l);
    results
}

fn backward_slice(
    stmts: &[tree_sitter::Node],
    decls: &HashMap<String, usize>,
    source: &str,
    line: usize,
) -> Vec<usize> {
    let mut lines = vec![line];
    let mut stack = vec![line];

    while let Some(current_line) = stack.pop() {
        let Some(current) = stmts.iter().find(|s| s.start_position().row + 1 == current_line) else { continue };
        for var in extract_identifiers(current, source.as_bytes()) {
            if let Some(&decl_line) = decls.get(&var) {
                if !lines.contains(&decl_line) {
                    lines.push(decl_line);
                    stack.push(decl_line);
                }
            }
        }
    }

    lines.sort();
    lines
}

fn extract_identifiers(node: &tree_sitter::Node, source: &[u8]) -> Vec<String> {
    let mut names = Vec::new();
    let mut cursor = node.walk();
    loop {
        let n = cursor.node();
        if n.is_named() && n.kind() == "identifier" {
            if let Ok(name) = n.utf8_text(source) {
                names.push(name.to_string());
            }
        }
        if cursor.goto_first_child() { continue; }
        loop {
            if cursor.goto_next_sibling() { break; }
            if !cursor.goto_parent() { return names; }
        }
    }
}

fn find_callee_in_stmt(stmt: &tree_sitter::Node, source: &str) -> Option<String> {
    let mut cursor = stmt.walk();
    loop {
        let node = cursor.node();
        if node.is_named() && node.kind() == "call_expression" {
            // 提取函数名（call_expression 的第一个 identifier 子节点）
            let mut cc = node.walk();
            if cc.goto_first_child() {
                loop {
                    let child = cc.node();
                    if child.is_named() && child.kind() == "identifier" {
                        return child.utf8_text(source.as_bytes()).ok().map(|s| s.to_string());
                    }
                    if !cc.goto_next_sibling() { break; }
                }
            }
        }
        if cursor.goto_first_child() { continue; }
        loop {
            if cursor.goto_next_sibling() { break; }
            if !cursor.goto_parent() { return None; }
        }
    }
}
