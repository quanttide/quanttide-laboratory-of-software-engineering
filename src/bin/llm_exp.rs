/// 实验：reflect（切片）→ LLM 联动管道
/// 验证：finding 涉及可追溯的变量定义，reflect 能提供有效上下文

use std::path::Path;

#[tokio::main]
async fn main() {
    let api_key = match qtcloud_code_cli::llm::get_api_key_from_vault().await {
        Ok(key) => key,
        Err(e) => { eprintln!("Vault 读取失败: {}", e); std::process::exit(1); }
    };

    // 模拟 review 产出的 finding：过长函数中的复杂表达式
    let code = r#"
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

    // finding 在 L7: sum += v — 依赖 sum 和 v，reflect 应该追溯到 L3 和 L6
    let finding_line = 7;
    let finding_msg = "L7: sum += v; — 表达式过于复杂，建议拆分".to_string();

    // 1. reflect: 从 finding 位置做反向切片
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&tree_sitter_rust::LANGUAGE.into()).unwrap();
    let tree = parser.parse(code, None).unwrap();
    let slice = qtcloud_code_cli::reflect::slice::backward_slice(code, &tree, Path::new("f.rs"), finding_line);

    println!("=== finding ===");
    println!("位置: L{}", finding_line);
    println!("问题: {}", finding_msg);

    println!("\n=== reflect 切片（{} 条语句）===", slice.len());
    for s in &slice {
        let mark = if s.line == finding_line { " ← finding" } else { "" };
        println!("  L{} {}{}", s.line, s.text, mark);
    }

    // 验证 reflect 是否追溯到了定义
    if slice.len() <= 1 {
        eprintln!("\n❌ reflect 失败：未追溯到上游定义");
        std::process::exit(1);
    }
    println!("\n✅ reflect 成功：追溯了 {} 条语句", slice.len());

    // 2. LLM 增强
    let ctx: String = slice.iter().map(|s| format!("L{} {}", s.line, s.text)).collect::<Vec<_>>().join("\n");
    let prompt = format!("代码上下文（从 finding 反向追溯）:\n{}\n\nfinding: {}", ctx, finding_msg);

    println!("\n=== LLM 增强 ===");
    match qtcloud_code_cli::llm::enhance_finding(code, &prompt, &api_key).await {
        Ok(enh) => {
            println!("优先级: {}", enh.priority);
            println!("解释: {}", enh.explanation);
            println!("置信度: {}", enh.confidence);
            if enh.explanation.contains("sum") && enh.explanation.contains("v") {
                println!("\n✅ LLM 正确利用了 reflect 上下文（分析了 sum 和 v）");
            }
        }
        Err(e) => eprintln!("LLM 错误: {}", e),
    }
}
