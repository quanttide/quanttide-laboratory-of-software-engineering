/// 实验：用 DeepSeek 增强代码审查 finding

#[tokio::main]
async fn main() {
    let api_key = std::env::var("DEEPSEEK_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        eprintln!("请设置 DEEPSEEK_API_KEY 环境变量");
        std::process::exit(1);
    }

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

    let finding = "L8: 函数 helper 被调用但未在当前文件中定义";

    println!("=== 发送到 DeepSeek ===");
    match qtcloud_code_cli::llm::enhance_finding(code, finding, &api_key).await {
        Ok(enhancement) => {
            println!("优先级: {}", enhancement.priority);
            println!("解释: {}", enhancement.explanation);
            println!("置信度: {}", enhancement.confidence);
        }
        Err(e) => {
            eprintln!("错误: {}", e);
        }
    }
}
