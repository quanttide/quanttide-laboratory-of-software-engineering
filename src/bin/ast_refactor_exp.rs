/// AST 变换实验：死代码检测 + 符号表（正确 tree walk）

fn main() {
    let code = r#"
fn used() -> i32 {
    42
}

fn unused() -> i32 {
    99
}

fn main() {
    let x = used();
    let y = x + 1;
    y
}
"#;

    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&tree_sitter_rust::LANGUAGE.into()).unwrap();
    let tree = parser.parse(code, None).unwrap();
    let root = tree.root_node();

    // 第一遍：收集所有函数定义
    let mut funcs: Vec<String> = Vec::new();
    let mut called: Vec<bool> = Vec::new();
    walk(&root, &mut |n| {
        if n.is_named() && n.kind() == "function_item" {
            if let Some(name) = n.child_by_field_name("name")
                .and_then(|nn| nn.utf8_text(code.as_bytes()).ok())
            {
                funcs.push(name.to_string());
                called.push(false);
            }
        }
    });

    // 第二遍：标记被调用的函数
    walk(&root, &mut |n| {
        if n.is_named() && n.kind() == "call_expression" {
            if let Some(name) = n.child_by_field_name("function")
                .and_then(|nn| nn.utf8_text(code.as_bytes()).ok())
            {
                let name_str = name.to_string();
                for (i, fname) in funcs.iter().enumerate() {
                    if *fname == name_str { called[i] = true; }
                }
            }
        }
    });

    println!("=== 死代码检测 ===");
    for (i, name) in funcs.iter().enumerate() {
        let status = if called[i] { "✓" } else if name == "main" { "★ (入口)" } else { "⚠ 未使用" };
        println!("  {} — {}", name, status);
    }

    // 字节偏移测试
    println!("\n=== 字节偏移 ===");
    for line in [1, 2, 14] {
        let byte = byte_for_line(code, line);
        let line_text = code[byte..].lines().next().unwrap_or("(end)");
        println!("  L{} → byte {}: {}", line, byte, line_text);
    }
}

fn byte_for_line(source: &str, line: usize) -> usize {
    source.split('\n').take(line - 1).map(|s| s.len() + 1).sum()
}

fn walk<F: FnMut(tree_sitter::Node)>(node: &tree_sitter::Node, f: &mut F) {
    f(*node);
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            walk(&cursor.node(), f);
            if !cursor.goto_next_sibling() { break; }
        }
    }
}
