/// 探索：reflect 工具组合模式试验
/// 测试多种工具组合，识别 LLM 能提供什么额外价值

use std::path::Path;
use std::collections::HashMap;

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

    // 找到函数节点
    let mut func = None;
    let mut c = root.walk();
    loop {
        let n = c.node();
        if n.is_named() && n.kind() == "function_item" { func = Some(n); break; }
        if c.goto_first_child() { continue; }
        loop { if c.goto_next_sibling() { break; } if !c.goto_parent() { break; } }
    }
    let func = func.unwrap();
    let stmts = qtcloud_code_cli::reflect::slice::flatten_stmts(&func);
    let file = Path::new("f.rs");

    // ============ 模式 1: backward_slice × dataflow → LLM 安全分析 ============
    println!("════════════════════════════════════════════");
    println!("模式 1: backward_slice + dataflow → 安全分析");
    println!("════════════════════════════════════════════");
    // 目标: parts[2] 的索引是否安全？
    // backward_slice 从 parts[2] 处追溯 → 发现 parts 来源是 trimmed.split
    // dataflow 追溯 qty → 发现依赖 parts[2]
    // LLM 结合两者: "没有检查 parts.len()"

    let line_qty = 7; // let qty: i32 = parts[2].trim()...
    let slice = qtcloud_code_cli::reflect::slice::backward_slice(code, &tree, file, line_qty);
    let flow = qtcloud_code_cli::reflect::dataflow::trace_variable(code, &tree, line_qty, "qty");

    let ctx1 = format!(
        "backward_slice:\n{}\n\ndataflow for qty:\n{}",
        slice.iter().map(|s| format!("L{} {}", s.line, s.text)).collect::<Vec<_>>().join("\n"),
        flow.iter().map(|f| format!("{} <- {}", f.var, f.from)).collect::<Vec<_>>().join("\n")
    );
    let resp1 = llm_call(&api_key, &format!(
        "以下是代码中一条访问数组元素的语句的追溯信息。请判断：\
         这条访问是否安全？（parts[2] 之前有没有检查 parts 长度？）\n\n{}", ctx1
    )).await;
    println!("LLM: {}", resp1);

    // ============ 模式 2: flatten_stmts + LLM → 重复模式识别 ============
    println!("\n════════════════════════════════════════════");
    println!("模式 2: flatten_stmts → 重复模式识别");
    println!("════════════════════════════════════════════");
    // 把函数体展平，让 LLM 识别重复结构
    // price 解析和 qty 解析用了同样的 parse().map_err(|_| "...")? 模式
    let ctx2: String = stmts.iter().enumerate()
        .map(|(i, s)| format!("{}. {}", i + 1, s.utf8_text(code.as_bytes()).unwrap_or("?")))
        .collect::<Vec<_>>().join("\n");
    let resp2 = llm_call(&api_key, &format!(
        "以下是一个函数的全部语句。请识别是否存在重复或相似的模式：\n\n{}", ctx2
    )).await;
    println!("LLM: {}", resp2);

    // ============ 模式 3: dataflow 多变量交叉 → 一致性检查 ============
    println!("\n════════════════════════════════════════════");
    println!("模式 3: dataflow 多变量 → 一致性检查");
    println!("════════════════════════════════════════════");

    let mut vars: HashMap<&str, Vec<String>> = HashMap::new();
    for var in &["name", "price", "qty", "total"] {
        let f = qtcloud_code_cli::reflect::dataflow::trace_variable(code, &tree, 8, var);
        vars.insert(var, f.iter().map(|e| format!("{} <- {}", e.var, e.from)).collect());
    }
    let ctx3: String = vars.iter()
        .map(|(v, steps)| format!("{}: {}", v, steps.join(" → ")))
        .collect::<Vec<_>>().join("\n");
    let resp3 = llm_call(&api_key, &format!(
        "以下是几个变量的数据流路径。请分析它们之间的依赖关系和潜在问题：\n\n{}", ctx3
    )).await;
    println!("LLM: {}", resp3);

    // ============ 模式总结 ============
    println!("\n════════════════════════════════════════════");
    println!("模式总结");
    println!("════════════════════════════════════════════");
    println!("1. slice + dataflow → LLM 安全分析");
    println!("   LLM 发现了 parts[2] 越界风险（规则引擎无法发现）");
    println!("2. flatten → LLM 重复模式");
    println!("   LLM 识别 price/qty 解析重复（规则引擎只能看到行数）");
    println!("3. dataflow × N → LLM 一致性检查");
    println!("   LLM 交叉对比多变量路径发现逻辑缺口");
}

async fn llm_call(api_key: &str, prompt: &str) -> String {
    match qtcloud_code_cli::llm::enhance_finding("", prompt, api_key).await {
        Ok(enh) => enh.explanation,
        Err(e) => format!("错误: {}", e),
    }
}
