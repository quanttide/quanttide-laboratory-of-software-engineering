/// 探索：5 个新工具 → 全链路分析
/// forward_slice / call_graph / impact_analysis / code_search / type_info

fn main() {
    let code = r#"
fn parse_price(s: &str) -> f64 {
    s.trim().parse().unwrap_or(0.0)
}

fn calc_total(price: f64, qty: i32) -> f64 {
    let total = price * qty as f64;
    total
}

fn process_order(raw: &str) -> Result<String, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() { return Err("空".into()); }
    let parts: Vec<&str> = trimmed.split(',').collect();
    let _name = parts[0].trim();
    let price: f64 = match parts[1].trim().parse() {
        Ok(v) => v,
        Err(_) => return Err("价格错误".into()),
    };
    let qty: i32 = match parts[2].trim().parse() {
        Ok(v) => v,
        Err(_) => return Err("数量错误".into()),
    };
    let total = calc_total(price, qty);
    Ok(format!("{:.2}", total))
}
"#;

    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&tree_sitter_rust::LANGUAGE.into()).unwrap();
    let tree = parser.parse(code, None).unwrap();
    let root = tree.root_node();

    // 1. forward_slice：找 total 的所有使用
    println!("════════════════════════════════════════════");
    println!("1. forward_slice: total 的所有引用");
    println!("════════════════════════════════════════════");
    for s in qtcloud_code_cli::reflect::analysis::forward_slice(code, &tree, "f.rs", 5) {
        println!("  L{} {}", s.line, s.text);
    }

    // 2. call_graph
    println!("\n════════════════════════════════════════════");
    println!("2. call_graph: 函数调用关系");
    println!("════════════════════════════════════════════");
    let graph = qtcloud_code_cli::reflect::analysis::build_call_graph(code, &tree);
    for (name, node) in &graph {
        let callees = if node.callees.is_empty() { "—".to_string() } else { node.callees.join(", ") };
        let callers = if node.callers.is_empty() { "—".to_string() } else { node.callers.join(", ") };
        println!("  {} (L{}): 调用 {} | 被 {}", name, node.line, callees, callers);
    }

    // 3. impact_analysis：改 price 定义会怎样
    println!("\n════════════════════════════════════════════");
    println!("3. impact_analysis: 修改 L18 price 定义的连锁影响");
    println!("════════════════════════════════════════════");
    let impact = qtcloud_code_cli::reflect::analysis::impact_analysis(code, &tree, "f.rs", 18);
    println!("  变量 {} (L{}) 被以下位置引用:", impact.var_name, impact.def_line);
    for s in &impact.forward_usages {
        println!("    L{} {}", s.line, s.text);
    }
    if impact.forward_usages.is_empty() {
        println!("    (无引用)");
    }

    // 4. code_search：找所有错误处理
    println!("\n════════════════════════════════════════════");
    println!("4. code_search: 找所有 return Err");
    println!("════════════════════════════════════════════");
    let results = qtcloud_code_cli::reflect::analysis::code_search(code, &tree, "(call_expression) @call");
    for s in results.iter().filter(|s| s.text.contains("Err") || s.text.contains("return")) {
        println!("  L{} {}", s.line, s.text);
    }

    // 5. type_info
    println!("\n════════════════════════════════════════════");
    println!("5. type_info: 变量类型注解");
    println!("════════════════════════════════════════════");
    for t in qtcloud_code_cli::reflect::analysis::type_info(code, &tree) {
        let typ = t.type_annotation.as_deref().unwrap_or("(无注解)");
        println!("  L{} {}: {}", t.line, t.var, typ);
    }

    // 6. 组合发现
    println!("\n════════════════════════════════════════════");
    println!("6. 组合发现");
    println!("════════════════════════════════════════════");

    // impact + call_graph 交叉：price 被哪些函数使用？
    println!("  price 的 forward_slice + call_graph:");
    println!("    price 定义于 L19（parse_price 内）");
    let fs = qtcloud_code_cli::reflect::analysis::forward_slice(code, &tree, "f.rs", 19);
    if fs.is_empty() {
        println!("    price 在 parse_price 内是临时变量，不传出。好设计。");
    } else {
        println!("    price 被 {} 处引用", fs.len());
    }

    // forward_slice 反向验证：calc_total 的参数从哪来
    println!("\n  price 和 qty 在 calc_total 中使用，信息来源:");
    let total_fs = qtcloud_code_cli::reflect::analysis::forward_slice(code, &tree, "f.rs", 5);
    for s in &total_fs {
        println!("    L{} 引用 total", s.line);
    }

    // 验证 type_info 是否能发现类型缺口
    let types = qtcloud_code_cli::reflect::analysis::type_info(code, &tree);
    let untyped: Vec<_> = types.iter().filter(|t| t.type_annotation.is_none()).collect();
    if !untyped.is_empty() {
        println!("\n  未标注类型的变量:");
        for t in &untyped {
            println!("    L{} {}", t.line, t.var);
        }
    } else {
        println!("\n  所有变量都有类型注解");
    }
}
