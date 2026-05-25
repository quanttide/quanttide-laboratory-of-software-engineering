/// 实验：reflect（切片）→ LLM 联动管道

use std::path::Path;

#[tokio::main]
async fn main() {
    let api_key = match qtcloud_code_cli::llm::get_api_key_from_vault().await {
        Ok(key) => key,
        Err(e) => { eprintln!("Vault 读取失败: {}", e); std::process::exit(1); }
    };

    // 模拟 review 产出的 finding
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

    let finding_line = 6;  // let v = helper(*item);
    let finding_msg = format!("L{}: let v = helper(*item); — 函数 helper 被调用但未在当前文件中定义", finding_line);

    // 1. reflect: 从 finding 位置做反向切片，获取上下文
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&tree_sitter_rust::LANGUAGE.into()).unwrap();
    let tree = parser.parse(code, None).unwrap();
    let slice = qtcloud_code_cli::reflect::slice::backward_slice(code, &tree, Path::new("f.rs"), finding_line);

    println!("=== finding ===");
    println!("位置: L{}", finding_line);
    println!("问题: {}", finding_msg);

    println!("\n=== reflect 切片（{} 条语句）===", slice.len());
    let ctx: String = slice.iter().map(|s| format!("L{} {}", s.line, s.text)).collect::<Vec<_>>().join("\n");
    println!("{}", ctx);

    // 2. LLM 增强：切片上下文 + finding → 优先级/解释/置信度
    let prompt = format!("代码上下文：\n{}\n\n问题：L{} {}", ctx, finding_line, finding_msg);

    println!("\n=== LLM 增强 ===");
    match qtcloud_code_cli::llm::enhance_finding(code, &prompt, &api_key).await {
        Ok(enh) => {
            println!("优先级: {}", enh.priority);
            println!("解释: {}", enh.explanation);
            println!("置信度: {}", enh.confidence);
        }
        Err(e) => eprintln!("LLM 错误: {}", e),
    }
}
