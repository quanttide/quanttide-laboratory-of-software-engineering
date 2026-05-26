/// 证据链排序实验：正向 vs 反向追溯链对 LLM 发现的影响
use std::path::Path;

const CODE: &str = r#"fn process_order(input: &str) -> Result<String, String> {
    let trimmed = input.trim();
    let parts: Vec<&str> = trimmed.split(',').collect();
    let name = parts[0].trim();
    let price: f64 = parts[1].trim().parse().map_err(|e| format!("bad price: {}", e))?;
    let qty: f64 = parts[2].trim().parse().map_err(|e| format!("bad qty: {}", e))?;
    let total = price * qty;
    Ok(format!("{}: {:.2}", name, total))
}"#;

#[tokio::main]
async fn main() {
    let api_key = match qtcloud_code_cli::llm::get_api_key_from_vault().await {
        Ok(k) => k,
        Err(e) => { eprintln!("Vault 失败: {}", e); std::process::exit(1); }
    };

    // 解析代码
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&tree_sitter_rust::LANGUAGE.into()).unwrap();
    let tree = parser.parse(CODE, None).unwrap();

    // 获取 L6 (parts[2]) 的反向切片
    let slice = qtcloud_code_cli::reflect::slice::backward_slice(CODE, &tree, Path::new("sample.rs"), 6);
    eprintln!("=== 反向切片（共 {} 条）===", slice.len());
    for s in &slice {
        eprintln!("  L{}: {}", s.line, s.text);
    }

    // 拼接正向链（按行号升序）
    let mut fwd: Vec<_> = slice.clone();
    fwd.sort_by_key(|s| s.line);
    let fwd_text: String = fwd.iter()
        .map(|s| format!("L{} {}", s.line, s.text))
        .collect::<Vec<_>>()
        .join("\n");

    // 拼接反向链（按行号降序）
    let mut rev: Vec<_> = slice.clone();
    rev.sort_by_key(|s| std::cmp::Reverse(s.line));
    let rev_text: String = rev.iter()
        .map(|s| format!("L{} {}", s.line, s.text))
        .collect::<Vec<_>>()
        .join("\n");

    // 正向 prompt
    let fwd_prompt = format!(
        "你是一个代码审查助手。分析以下代码中从输入到输出的完整数据流，找出潜在问题。\n\n\
        代码：\n```rust\n{}\n```\n\n\
        以下是按执行顺序整理的数据流追溯链：\n```\n{}\n```\n\n\
        请输出 JSON：\
        {{\"issues\": [{{\"line\": 数字, \"severity\": \"high/medium/low\", \"description\": \"中文描述\"}}], \
        \"summary\": \"总体评价（中文）\"}}",
        CODE, fwd_text
    );

    // 反向 prompt
    let rev_prompt = format!(
        "你是一个代码审查助手。分析以下代码中从输出到输入的反向追溯链，找出潜在问题。\n\n\
        代码：\n```rust\n{}\n```\n\n\
        以下是从输出反向追溯到输入的数据流链：\n```\n{}\n```\n\n\
        请输出 JSON：\
        {{\"issues\": [{{\"line\": 数字, \"severity\": \"high/medium/low\", \"description\": \"中文描述\"}}], \
        \"summary\": \"总体评价（中文）\"}}",
        CODE, rev_text
    );

    println!("===== 正向链实验 =====");
    let fwd_result = call_llm(&fwd_prompt, &api_key).await;
    println!("{}", fwd_result);

    println!("\n===== 反向链实验 =====");
    let rev_result = call_llm(&rev_prompt, &api_key).await;
    println!("{}", rev_result);

    // 对比
    println!("\n===== 对比 =====");
    let fwd_confidence = qtcloud_code_cli::llm::compute_confidence(&fwd_result);
    let rev_confidence = qtcloud_code_cli::llm::compute_confidence(&rev_result);
    println!("正向链置信度: {}", fwd_confidence);
    println!("反向链置信度: {}", rev_confidence);
    println!("\n正向链发现 {} 个问题, 反向链发现 {} 个问题",
        count_issues(&fwd_result), count_issues(&rev_result));
}

async fn call_llm(prompt: &str, api_key: &str) -> String {
    use serde_json::json;

    let client = reqwest::Client::new();
    let body = json!({
        "model": "deepseek-chat",
        "messages": [
            {"role": "system", "content": "你是一个专业的代码审查助手，回答格式为 JSON。"},
            {"role": "user", "content": prompt}
        ],
        "temperature": 0.1,
        "max_tokens": 1024,
    });

    let resp = client
        .post("https://api.deepseek.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&body)
        .send()
        .await;

    match resp {
        Ok(r) => {
            let text = r.text().await.unwrap_or_default();
            let parsed: serde_json::Value = serde_json::from_str(&text).unwrap_or(json!({}));
            let content = parsed["choices"][0]["message"]["content"]
                .as_str()
                .unwrap_or("(parse error)")
                .to_string();
            // Strip markdown code fence if present
            content
                .trim()
                .strip_prefix("```json").unwrap_or(&content)
                .strip_prefix("```").unwrap_or(&content)
                .trim()
                .trim_end_matches("```")
                .trim()
                .to_string()
        }
        Err(e) => format!("{{\"error\": \"{}\"}}", e),
    }
}

fn count_issues(json: &str) -> usize {
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(json) {
        v["issues"].as_array().map(|a| a.len()).unwrap_or(0)
    } else {
        0
    }
}
