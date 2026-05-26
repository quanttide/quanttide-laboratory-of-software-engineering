/// 探索：证据链组织形式对 LLM 意外发现的影响
/// 同一段代码、同一个 bug，不同的证据呈现方式

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
    Ok(format!("{:.2}", total))
}
"#;

    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&tree_sitter_rust::LANGUAGE.into()).unwrap();
    let tree = parser.parse(code, None).unwrap();

    // 从 total 行做 backward_slice（完整追溯）
    let total_line = 8;
    let full_slice = qtcloud_code_cli::reflect::slice::backward_slice(code, &tree, Path::new("f.rs"), total_line);
    let full_ctx: String = full_slice.iter().map(|s| format!("L{} {}", s.line, s.text)).collect::<Vec<_>>().join("\n");

    // 模式 A：只给 finding 行，不给上下文
    // 模式 B：给整段代码
    // 模式 C：给 backward_slice 追溯链（完整证据）

    let cases = vec![
        ("A: 只有行号", format!("L{}: {}", total_line,
            full_slice.iter().find(|s| s.line == total_line).map(|s| s.text.as_str()).unwrap_or(""))),
        ("B: 完整代码", code.to_string()),
        ("C: 追溯链", full_ctx.clone()),
    ];

    for (label, ctx) in &cases {
        println!("\n══════ {} ══════", label);
        let prompt = format!("以下是一段 Rust 代码。请分析有什么问题：\n\n{}", ctx);
        match qtcloud_code_cli::llm::enhance_finding(code, &prompt, &api_key).await {
            Ok(enh) => {
                let conf = qtcloud_code_cli::llm::compute_confidence(&enh.explanation);
                println!("置信度: {} (锚定引用: {})", conf, count_refs(&enh.explanation));
                println!("{}", enh.explanation.lines().next().unwrap_or(""));
                let found_bug = enh.explanation.contains("越界") || enh.explanation.contains("长度")
                    || enh.explanation.contains("panic") || enh.explanation.contains("index");
                println!("发现越界 bug: {}", if found_bug { "✅" } else { "❌" });
            }
            Err(e) => eprintln!("错误: {}", e),
        }
    }
}

fn count_refs(text: &str) -> usize {
    let markers = ["L", "行", "line", "parts", "split", "trim", "parse", "index", "length"];
    let mut count = 0;
    for m in &markers {
        count += text.matches(m).count();
    }
    count
}
