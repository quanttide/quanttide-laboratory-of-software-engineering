/// 实验：reflect→LLM 联动 — 过长函数职责分析
/// 规则引擎发现过长函数（28行），LLM 通过展平语句分析职责并建议拆分

#[tokio::main]
async fn main() {
    let api_key = match qtcloud_code_cli::llm::get_api_key_from_vault().await {
        Ok(key) => key,
        Err(e) => { eprintln!("Vault 读取失败: {}", e); std::process::exit(1); }
    };

    let code = r#"
fn process_order(raw: &str) -> Result<String, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("空输入".into());
    }
    let parts: Vec<&str> = trimmed.split(',').collect();
    if parts.len() < 3 {
        return Err("字段不足".into());
    }
    let name = parts[0].trim();
    if name.is_empty() {
        return Err("名称为空".into());
    }
    let price: f64 = match parts[1].trim().parse() {
        Ok(v) => v,
        Err(_) => return Err("价格格式错误".into()),
    };
    if price <= 0.0 {
        return Err("价格必须大于 0".into());
    }
    let qty: i32 = match parts[2].trim().parse() {
        Ok(v) => v,
        Err(_) => return Err("数量格式错误".into()),
    };
    if qty <= 0 {
        return Err("数量必须大于 0".into());
    }
    let total = price * qty as f64;
    Ok(format!("{}: {:.2} x {} = {:.2}", name, price, qty, total))
}
"#;

    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&tree_sitter_rust::LANGUAGE.into()).unwrap();
    let tree = parser.parse(code, None).unwrap();
    let root = tree.root_node();

    // 找到 process_order 函数
    let mut func = None;
    let mut c = root.walk();
    loop {
        let n = c.node();
        if n.is_named() && n.kind() == "function_item" {
            if let Some(name) = n.child_by_field_name("name")
                .and_then(|nn| nn.utf8_text(code.as_bytes()).ok())
            {
                if name == "process_order" { func = Some(n); break; }
            }
        }
        if c.goto_first_child() { continue; }
        loop { if c.goto_next_sibling() { break; } if !c.goto_parent() { break; } }
    }
    let func = func.unwrap();

    let stmts = qtcloud_code_cli::reflect::slice::flatten_stmts(&func);

    println!("=== finding ===");
    println!("process_order 函数共 28 行（超过 15 行阈值），建议拆分");

    println!("\n=== 展平语句（{} 条）===", stmts.len());
    let ctx: String = stmts.iter().enumerate().map(|(i, s)| {
        s.utf8_text(code.as_bytes()).map(|t| format!("{}. {}", i + 1, t)).unwrap_or_default()
    }).collect::<Vec<_>>().join("\n");
    println!("{}", ctx);

    if stmts.len() < 5 {
        eprintln!("\n❌ reflect 失败：语句展平不足");
        std::process::exit(1);
    }
    println!("\n✅ reflect 展平了 {} 条语句", stmts.len());

    // LLM 职责分析
    println!("\n=== LLM 职责分析 ===");
    let prompt = format!(
        "以下是一个过长函数的全部语句。请分析：\n\
         1. 这个函数在做几件不同的事？每件从第几句到第几句？\n\
         2. 建议拆成几个函数，每个函数名和职责？\n\n{}",
        ctx
    );
    match qtcloud_code_cli::llm::enhance_finding(code, &prompt, &api_key).await {
        Ok(enh) => {
            println!("优先级: {}", enh.priority);
            println!("{}", enh.explanation);
        }
        Err(e) => eprintln!("LLM 错误: {}", e),
    }
}
