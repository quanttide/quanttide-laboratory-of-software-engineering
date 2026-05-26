/// 假阳性过滤实验：在真实项目上统计 LLM dismiss 率
use std::path::Path;
use std::io::Write;

#[tokio::main]
async fn main() {
    let api_key = match qtcloud_code_cli::llm::get_api_key_from_vault().await {
        Ok(k) => k,
        Err(e) => { eprintln!("Vault 失败: {}", e); std::process::exit(1); }
    };

    // 1. 在真实项目上跑 review
    let findings = match qtcloud_code_cli::lab::run_review(
        "/home/iguo/repos/quanttide/domains/quanttide-code/apps/qtcloud-devops/src/cli",
        &["long-function", "long-parameter-list", "unused-variable", "missing-tests"],
    ) {
        Ok(f) => f,
        Err(e) => { eprintln!("review 失败: {}", e); std::process::exit(1); }
    };

    println!("total findings: {}", findings.len());

    // 2. 每个 finding 发给 LLM 判断
    let mut results = Vec::new();
    for (i, f) in findings.iter().enumerate() {
        let file = f["file"].as_str().unwrap_or("");
        let line = f["line"].as_u64().unwrap_or(0) as usize;
        let msg = f["message"].as_str().unwrap_or("");
        let rule = f["rule_id"].as_str().unwrap_or("");

        // 读代码上下文
        let ctx = read_context(file, line);
        let prompt = format!(
            "以下是一个代码审查 finding。请判断它是否为真问题：\n\n\
             文件: {}:{}\n规则: {}\n消息: {}\n\n\
             代码上下文:\n```\n{}\n```\n\n\
             请返回 JSON:\n{{\"verdict\": \"confirm/dismiss\", \"reason\": \"...\"}}",
            file, line, rule, msg, ctx
        );

        match qtcloud_code_cli::llm::enhance_finding("", &prompt, &api_key).await {
            Ok(r) => {
                let is_dismiss = r.confidence == "dismiss" || r.explanation.contains("建议忽略");
                let verdict = if is_dismiss { "DISMISS" } else { "CONFIRM" };
                println!("  [{}/{}] {}:{} {} → {}", i+1, findings.len(), file, line, verdict, r.explanation.lines().next().unwrap_or(""));
                results.push((file.to_string(), line, rule.to_string(), verdict.to_string(), r.explanation));
            }
            Err(e) => {
                eprintln!("  [{}/{}] LLM 错误: {}", i+1, findings.len(), e);
            }
        }
    }

    // 3. 统计
    let total = results.len();
    let dismiss = results.iter().filter(|r| r.3 == "DISMISS").count();
    let confirm = results.iter().filter(|r| r.3 == "CONFIRM").count();
    let dismiss_rate = if total > 0 { dismiss as f64 / total as f64 * 100.0 } else { 0.0 };

    // 4. 写报告
    let report = format!(
        r#"# 假阳性过滤实验报告

## 统计

| 指标 | 值 |
|------|---|
| 总 finding 数 | {} |
| LLM 确认 (CONFIRM) | {} |
| LLM 驳回 (DISMISS) | {} |
| 驳回率 | {:.1}% |

## 驳回详情

{}
"#,
        total, confirm, dismiss, dismiss_rate,
        results.iter().filter(|r| r.3 == "DISMISS").map(|r| {
            format!("- `{}:{}` [{}] {}", r.0, r.1, r.2, r.4.lines().next().unwrap_or(""))
        }).collect::<Vec<_>>().join("\n")
    );

    let report_path = "/home/iguo/repos/quanttide/domains/quanttide-code/examples/default/docs/false-positive-report.md";
    let mut f = std::fs::File::create(report_path).unwrap();
    f.write_all(report.as_bytes()).unwrap();
    println!("\n报告已写入: {}", report_path);
    println!("驳回率: {:.1}% ({}/{})", dismiss_rate, dismiss, total);
}

fn read_context(file: &str, line: usize) -> String {
    let content = std::fs::read_to_string(file).unwrap_or_default();
    let lines: Vec<&str> = content.lines().collect();
    let start = if line > 5 { line - 5 } else { 1 };
    let end = (line + 5).min(lines.len());
    (start..=end).map(|i| {
        let marker = if i == line { " →" } else { "  " };
        format!("{}{}: {}", marker, i, lines.get(i - 1).unwrap_or(&""))
    }).collect::<Vec<_>>().join("\n")
}
