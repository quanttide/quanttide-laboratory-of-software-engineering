/// 探索：backward_slice × dataflow × LLM
/// 对函数内每个变量做反向追溯→LLM 解释完整数据流动

use std::path::Path;

#[tokio::main]
async fn main() {
    let api_key = match qtcloud_code_cli::llm::get_api_key_from_vault().await {
        Ok(k) => k,
        Err(e) => { eprintln!("Vault 失败: {}", e); std::process::exit(1); }
    };

    let code = r#"
fn process_order(raw: &str) -> Result<String, String> {
    let trimmed = raw.trim();
    let parts: Vec<&str> = trimmed.split(',').collect();
    let name = parts[0].trim();
    let price: f64 = parts[1].trim().parse().map_err(|_| "价格错误")?;
    let qty: i32 = parts[2].trim().parse().map_err(|_| "数量错误")?;
    let total = price * qty as f64;
    Ok(format!("{}: {:.2} x {} = {:.2}", name, price, qty, total))
}
"#;

    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&tree_sitter_rust::LANGUAGE.into()).unwrap();
    let tree = parser.parse(code, None).unwrap();
    let root = tree.root_node();

    // 找到函数
    let mut func = None;
    let mut c = root.walk();
    loop {
        let n = c.node();
        if n.is_named() && n.kind() == "function_item" {
            func = Some(n); break;
        }
        if c.goto_first_child() { continue; }
        loop { if c.goto_next_sibling() { break; } if !c.goto_parent() { break; } }
    }
    let func = func.unwrap();
    let stmts = qtcloud_code_cli::reflect::slice::flatten_stmts(&func);

    // 对最后一行 total = price * qty 做切片，追溯每个变量
    let target = stmts.iter()
        .find(|s| s.utf8_text(code.as_bytes()).map(|t| t.contains("total")).unwrap_or(false))
        .unwrap();
    let target_line = target.start_position().row + 1;

    println!("=== 目标表达式 ===");
    println!("L{} {}", target_line, target.utf8_text(code.as_bytes()).unwrap_or("?"));

    println!("\n=== backward_slice 追溯 ===");
    let slice = qtcloud_code_cli::reflect::slice::backward_slice(code, &tree, Path::new("f.rs"), target_line);
    for s in &slice {
        println!("  L{} {}", s.line, s.text);
    }

    // 数据流：为每个变量单独追溯
    println!("\n=== dataflow 变量路径 ===");
    for var in ["price", "qty", "name", "total"] {
        let flow = qtcloud_code_cli::reflect::dataflow::trace_variable(code, &tree, target_line, var);
        if !flow.is_empty() {
            let path: Vec<String> = flow.iter().map(|f| format!("{}→{}", f.from, f.var)).collect();
            println!("  {}: {}", var, path.join(" → "));
        }
    }

    // LLM：用追溯结果解释完整数据流
    let ctx: String = slice.iter()
        .map(|s| format!("L{} {}", s.line, s.text))
        .collect::<Vec<_>>()
        .join("\n");

    println!("\n=== LLM 数据流解释 ===");
    let prompt = format!(
        "以下是一个函数内从原始输入到最终输出的数据流动。\
         请用自然语言描述数据是如何一步步变换的：\n\n{}",
        ctx
    );
    match qtcloud_code_cli::llm::enhance_finding(code, &prompt, &api_key).await {
        Ok(enh) => println!("{}", enh.explanation),
        Err(e) => eprintln!("LLM 错误: {}", e),
    }
}
